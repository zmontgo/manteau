//! In-memory transport that captures sent messages for test inspection.
//!
//! Always available — no feature gate, so the `lib.rs` doctest can rely on
//! it in the default feature set.

use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
  message::Message, models::MessageId, render::RenderError,
  transport::Transport,
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

#[derive(Debug, Clone)]
pub struct MockReceipt {
  pub ids: Vec<MessageId>,
}

impl MockTransport {
  pub fn new() -> Self { Self::default() }

  /// Non-destructive snapshot of messages captured so far.
  pub fn sent(&self) -> Vec<Message> {
    self
      .sent
      .lock()
      .expect("MockTransport mutex poisoned")
      .clone()
  }

  /// Drain captured messages — return them and clear internal state.
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
