//! In-memory capture of prepared mail. Nothing is delivered or rerendered.
use std::{convert::Infallible, sync::Mutex};

use async_trait::async_trait;
use manteau_core::{MessageId, PreparedMessage, Receipt, Transport};
/// Thread-safe local capture. Snapshots share immutable message bodies.
#[derive(Debug, Default)]
pub struct MockTransport {
  sent: Mutex<Vec<PreparedMessage>>,
}
impl MockTransport {
  /// Create an empty capture.
  pub fn new() -> Self { Self::default() }

  /// Snapshot captured submissions without copying their bodies.
  pub fn sent(&self) -> Vec<PreparedMessage> {
    self.sent.lock().expect("capture mutex poisoned").clone()
  }

  /// Move all captured submissions out and clear the capture.
  pub fn take_sent(&self) -> Vec<PreparedMessage> {
    std::mem::take(&mut *self.sent.lock().expect("capture mutex poisoned"))
  }
}
/// Evidence of local capture only. There is no provider-assigned message ID.
#[derive(Debug)]
pub struct MockReceipt;
impl Receipt for MockReceipt {
  fn ids(&self) -> &[MessageId] { &[] }
}
#[async_trait]
impl Transport for MockTransport {
  type Error = Infallible;
  type Receipt = MockReceipt;

  #[tracing::instrument(skip_all)]
  async fn send(
    &self,
    message: &PreparedMessage,
  ) -> Result<MockReceipt, Infallible> {
    self
      .sent
      .lock()
      .expect("capture mutex poisoned")
      .push(message.clone());
    Ok(MockReceipt)
  }
}
