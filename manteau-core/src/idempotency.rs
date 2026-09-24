//! Provider-enforced deduplication of immutable, rendered email submissions.
use std::time::Duration;

use async_trait::async_trait;

use crate::{PreparedMessage, Transport};

/// A caller-owned submission identity, retained unchanged across retries.
/// Never use a new key to recover an ambiguous submission.
#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
pub struct IdempotencyKey(String);

/// The key must contain 1–256 visible ASCII characters.
#[derive(Debug, thiserror::Error)]
#[error("idempotency key must contain 1–256 visible ASCII characters")]
pub struct InvalidIdempotencyKey;

impl TryFrom<String> for IdempotencyKey {
  type Error = InvalidIdempotencyKey;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.is_empty()
      || value.len() > 256
      || !value.bytes().all(|b| b.is_ascii_graphic())
    {
      return Err(InvalidIdempotencyKey);
    }
    Ok(Self(value))
  }
}

impl IdempotencyKey {
  /// The exact value sent to the provider.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// A transport that deduplicates submissions at the provider, not in memory.
///
/// Implementors must replay the original acceptance for an identical key and
/// payload and reject key reuse with different content. The guarantee applies
/// within `retention()` and the same configured account and region. Callers
/// must persist the key, prepared message and first-attempt time together and
/// stop retries before that window expires. Core's HTTP sender also binds a
/// provider protocol version: an upgrade prevents replay if serialization might
/// change. Routing through the same URL must keep the same deduplication
/// region; a URL or credential hash alone cannot prove where the provider
/// routed it. Acceptance does not prove delivery.
#[async_trait]
pub trait IdempotentTransport: Transport {
  /// Minimum provider retention from the first submission; never sliding.
  fn retention(&self) -> Duration;
  /// Stable, non-secret identity of this provider account and routing target.
  /// A changed scope must prevent replay against a different deduplication
  /// store.
  fn scope(&self) -> &str;
  /// Maximum duration of one request, reserved before the retention deadline.
  fn request_timeout(&self) -> Duration;
  /// Submit or replay the bound identity. Check scope and remaining retention
  /// immediately before dispatch; never replace the key or rerender content.
  async fn send_idempotent(
    &self,
    submission: &Submission,
  ) -> Result<Self::Receipt, Self::Error>;
}

impl serde::Serialize for IdempotencyKey {
  fn serialize<S: serde::Serializer>(
    &self,
    serializer: S,
  ) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.0)
  }
}

/// One durable submission identity, exact message, provider scope, and start
/// time. Persist this whole value before its first request and restore it only
/// from trusted storage. A replacement key represents a different operation.
/// This is intent, not proof that a provider accepted or delivered anything.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
  key:     IdempotencyKey,
  message: PreparedMessage,
  scope:   String,
  started: std::time::SystemTime,
}

/// A replay would exceed the guarantee of the bound provider.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
  /// Credentials or routing identify a different deduplication store.
  #[error("submission belongs to a different provider scope")]
  Scope,
  /// Retention cannot cover another complete request, or the clock moved
  /// backward.
  #[error("submission is outside its safe replay window")]
  Expired,
}

impl Submission {
  /// Bind a fresh identity and immutable message to a configured provider.
  /// The retention budget starts now, conservatively before the first request.
  pub fn new(
    key: IdempotencyKey,
    message: PreparedMessage,
    transport: &impl IdempotentTransport,
  ) -> Self {
    Self {
      key,
      message,
      scope: transport.scope().to_owned(),
      started: std::time::SystemTime::now(),
    }
  }

  /// Original key, unchanged for all attempts.
  pub fn key(&self) -> &IdempotencyKey {
    &self.key
  }

  /// Exact prepared message bound to the key.
  pub fn message(&self) -> &PreparedMessage {
    &self.message
  }

  /// Check scope and reserve a full request before the retention deadline.
  /// Adapters must call this immediately before dispatching a keyed request.
  pub fn check(
    &self,
    transport: &impl IdempotentTransport,
  ) -> Result<(), ReplayError> {
    if self.scope != transport.scope() {
      return Err(ReplayError::Scope);
    }
    let elapsed = self.started.elapsed().map_err(|_| ReplayError::Expired)?;
    match elapsed.checked_add(transport.request_timeout()) {
      Some(total) if total < transport.retention() => Ok(()),
      _ => Err(ReplayError::Expired),
    }
  }
}

impl std::fmt::Debug for Submission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Submission([redacted])")
  }
}
