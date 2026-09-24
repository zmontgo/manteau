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
  pub fn as_str(&self) -> &str { &self.0 }
}

/// A transport that deduplicates submissions at the provider, not in memory.
///
/// Implementors must replay the original acceptance for an identical key and
/// payload and reject key reuse with different content. The guarantee applies
/// within `retention()` and the same configured account and region. Callers
/// must persist the key, prepared message and first-attempt time together and
/// stop retries before that window expires. Acceptance does not prove delivery.
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
  /// Submit once or recover the original receipt using the same key and body.
  async fn send_idempotent(
    &self,
    key: &IdempotencyKey,
    message: &PreparedMessage,
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
