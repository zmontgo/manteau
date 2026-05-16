//! In-memory transport that captures sent messages for test inspection.
//!
//! Always available — no feature gate, so the `lib.rs` doctest can rely on
//! it in the default feature set.

use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
  message::Message,
  models::MessageId,
  render::RenderError,
  transport::{Receipt, Transport},
};

/// A no-op transport that captures every message it is asked to send.
///
/// Tests inspect captured messages via [`MockTransport::sent`] (snapshot,
/// non-destructive) or [`MockTransport::take_sent`] (drain). Render still
/// runs on send so that rendering failures surface in test as they would in
/// production.
#[derive(Debug, Default)]
pub struct MockTransport {
  sent: Mutex<Vec<Message>>,
}

/// Acknowledgement returned by [`MockTransport::send`]. Always carries a
/// single placeholder ID (`"mock"`) — the receipt exists to satisfy the
/// [`Transport`] contract. Assertions in tests should usually inspect
/// captured messages via [`MockTransport::sent`] /
/// [`MockTransport::take_sent`], not the receipt.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct MockReceipt {
  pub ids: Vec<MessageId>,
}

impl Receipt for MockReceipt {
  fn ids(&self) -> &[MessageId] { &self.ids }
}

impl MockTransport {
  /// Create an empty transport. Inspect captured messages with [`sent`]
  /// (non-destructive snapshot) or [`take_sent`] (drain).
  ///
  /// [`sent`]: MockTransport::sent
  /// [`take_sent`]: MockTransport::take_sent
  pub fn new() -> Self { Self::default() }

  /// Non-destructive snapshot of messages captured so far. The mock
  /// retains them — successive calls return the same set.
  ///
  /// Pair with [`take_sent`](MockTransport::take_sent) for drain
  /// semantics. The two differ:
  ///
  /// - `sent()` clones the internal Vec; state unchanged.
  /// - `take_sent()` moves the internal Vec out; state reset to empty.
  pub fn sent(&self) -> Vec<Message> {
    self
      .sent
      .lock()
      .expect("MockTransport mutex poisoned")
      .clone()
  }

  /// Drain captured messages — return them and clear internal state.
  /// See [`sent`](MockTransport::sent) for the snapshot variant.
  pub fn take_sent(&self) -> Vec<Message> {
    std::mem::take(
      &mut *self.sent.lock().expect("MockTransport mutex poisoned"),
    )
  }
}

#[async_trait]
impl Transport for MockTransport {
  type Error = RenderError;
  type Receipt = MockReceipt;

  #[tracing::instrument(skip_all, fields(subject = %message.subject))]
  async fn send(
    &self,
    message: &Message,
  ) -> Result<Self::Receipt, Self::Error> {
    // Render to surface the same failure modes a real transport would.
    message.render()?;
    self
      .sent
      .lock()
      .expect("MockTransport mutex poisoned")
      .push(message.clone());
    Ok(MockReceipt {
      ids: vec![MessageId::new("mock")],
    })
  }
}
