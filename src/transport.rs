//! Transport contract: the [`Transport`] trait every adapter implements, and
//! the [`TransportFailure`] trait every transport error implements.
//!
//! Cross-transport abstractions (retry policy, error rollup, alerting) live
//! against [`TransportFailure`] — they work uniformly over manteau's own
//! transports and over consumer-defined ones.
//!
//! # Error pattern
//!
//! Every transport's error follows manteau's standard kind + source shape:
//!
//! - A struct holds a private `kind` enum (the taxonomy callers branch on) and
//!   a `Box<dyn Error + Send + Sync>` source (the upstream error, preserved for
//!   downcasting and Display chaining).
//! - `Display` delegates to source — operator logs get the upstream library's
//!   message verbatim.
//! - A `pub fn kind()` accessor surfaces the taxonomy; a `pub(crate)`
//!   constructor on the kind enum pairs a kind with a source.
//! - Variants are named for failure source (`Network`, `Auth`, `Provider {
//!   status }`), not architectural layer.
//!
//! Consumer transports implementing [`Transport`] should follow the same
//! shape so their errors slot into cross-transport tooling cleanly. See
//! [`MailjetError`](crate::MailjetError) for a worked example.

use std::time::Duration;

use async_trait::async_trait;

use crate::{message::Message, models::MessageId, render::RenderError};

/// Universal shape of a transport receipt — exposes the message IDs the
/// provider assigned to the accepted message. Cross-transport observability
/// code (logging, metrics, audit) writes against this trait so it works
/// uniformly over manteau's transports and over consumer-defined ones.
pub trait Receipt {
  /// IDs the provider assigned to the accepted message(s). One transport
  /// may produce multiple IDs (one per recipient, in Mailjet's case);
  /// others may produce a single placeholder. Order matches the provider's
  /// order, which usually mirrors the `to` field.
  fn ids(&self) -> &[MessageId];
}

/// Properties every transport's error must surface so that abstractions
/// written generically over `T: TransportFailure` can reason about the
/// failure without knowing which provider produced it.
pub trait TransportFailure: std::error::Error + Send + Sync + 'static {
  /// Should the caller retry the operation? True for network blips, rate
  /// limits, transient provider failures. False for auth, malformed input,
  /// permanent rejection.
  fn is_transient(&self) -> bool;

  /// Did the failure originate from authentication or authorization?
  /// True means the caller needs to refresh credentials before another
  /// attempt.
  fn is_auth(&self) -> bool;

  /// Did the provider reject the message itself, vs. infrastructure
  /// failing to deliver? True means the message content/shape is the
  /// problem, not the connection.
  fn is_message_rejected(&self) -> bool;

  /// If the provider indicates when a retry is safe (e.g., a `Retry-After`
  /// header on rate-limit responses), surface it. Default: no information.
  fn retry_after(&self) -> Option<Duration> { None }
}

/// The mechanism that ships a [`Message`] to its destination.
///
/// Library-supplied implementations: [`MailjetTransport`], [`MockTransport`],
/// [`StdoutTransport`]. External consumers implement this for non-Mailjet
/// providers (SMTP, SES, Postmark, etc.) — the trait is the consumer
/// extension point. Each implementation owns its own `Receipt` and `Error`
/// types; the trait dictates the contract, not their internal shapes.
///
/// [`MailjetTransport`]: crate::transports::mailjet::MailjetTransport
/// [`MockTransport`]: crate::transports::mock::MockTransport
/// [`StdoutTransport`]: crate::transports::stdout::StdoutTransport
#[async_trait]
pub trait Transport: Send + Sync {
  /// Provider-specific success type. Carries the message IDs and any raw
  /// response data the consumer might want. Must implement [`Receipt`] so
  /// cross-transport observability code can extract IDs uniformly.
  type Receipt: Receipt;

  /// Provider-specific error type. Must implement [`TransportFailure`] so
  /// cross-transport middleware can branch on the universal categories.
  type Error: TransportFailure;

  async fn send(&self, message: &Message)
  -> Result<Self::Receipt, Self::Error>;
}

/// Render failures are not caller-actionable — they indicate a bug in
/// manteau or its rendering dependencies. All predicates return `false` so
/// retry middleware does not loop on them.
impl TransportFailure for RenderError {
  fn is_transient(&self) -> bool { false }

  fn is_auth(&self) -> bool { false }

  fn is_message_rejected(&self) -> bool { false }
}
