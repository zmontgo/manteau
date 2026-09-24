//! Contracts implemented by independent delivery adapters.
use std::time::Duration;

use async_trait::async_trait;

use crate::{MessageId, PreparedMessage};

/// Evidence returned by a completed adapter call. A receipt does not
/// universally prove delivery: inspect provider-specific recipient outcomes
/// when available.
pub trait Receipt: std::fmt::Debug {
  /// Genuine provider-assigned IDs only. An empty slice means the adapter has
  /// no provider ID (for example a local capture). Never synthesize IDs from
  /// addresses.
  fn ids(&self) -> &[MessageId];
}

/// What an unsuccessful call establishes about remote acceptance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acceptance {
  /// No recipient was accepted by this invocation.
  NotAccepted,
  /// Some or all recipients may have been accepted. An unkeyed retry can
  /// duplicate mail.
  Unknown,
  /// At least one recipient was accepted, while another failed. Inspect the
  /// adapter's recipient report; never resend the entire envelope blindly.
  Partial,
}

/// Provider-independent failure facts. Temporary failure is separate from
/// acceptance evidence: `is_transient()` never grants permission to retry.
/// Implementations must conservatively report unknown acceptance when evidence
/// is incomplete. Error formatting should omit credentials and private mail.
pub trait TransportFailure: std::error::Error + Send + Sync + 'static {
  /// Whether the underlying failure may resolve without changing the message.
  fn is_transient(&self) -> bool;
  /// Whether credentials or permissions need attention.
  fn is_auth(&self) -> bool;
  /// Whether the provider explicitly rejected the message content or envelope.
  fn is_message_rejected(&self) -> bool;
  /// Acceptance evidence retained even when the request or response failed.
  fn acceptance(&self) -> Acceptance;
  /// Provider-requested delay. This is a lower bound, not evidence of retry
  /// safety.
  fn retry_after(&self) -> Option<Duration> { None }
}

/// A configured delivery mechanism consuming already prepared mail.
/// Implementors must not rerender content, retry ambiguous unkeyed submissions,
/// or claim acceptance from a malformed response. Providers own their receipt
/// and error types; applications own persistence, scheduling, and recovery.
#[async_trait]
pub trait Transport: Send + Sync {
  /// Provider evidence, not necessarily proof of final delivery.
  type Receipt: Receipt;
  /// Failure retaining classification and acceptance uncertainty.
  type Error: TransportFailure;
  /// Submit these exact bodies and envelope once. Reuse configured transports
  /// to retain connection pools. Cancellation after dispatch can leave the
  /// remote outcome unknown even if no Rust error is returned to the caller.
  async fn send(
    &self,
    message: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error>;
}

impl TransportFailure for std::convert::Infallible {
  fn is_transient(&self) -> bool { match *self {} }

  fn is_auth(&self) -> bool { match *self {} }

  fn is_message_rejected(&self) -> bool { match *self {} }

  fn acceptance(&self) -> Acceptance { match *self {} }
}
