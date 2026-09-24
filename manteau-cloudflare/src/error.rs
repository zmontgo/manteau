use manteau_core::{Acceptance, TransportFailure, http::HttpError};
/// Cloudflare-specific response or input failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CloudflareErrorKind {
  /// The prepared message violates this provider's documented input
  /// requirements.
  Input,
  /// The response lacks sufficient or consistent acceptance evidence.
  Response,
  /// The provider explicitly rejected this message.
  Rejected,
}
/// Sanitized provider failure; source details require explicit diagnostic
/// access.
#[derive(thiserror::Error)]
#[error("Cloudflare protocol failed: {kind:?}")]
pub struct CloudflareError {
  pub(crate) kind:          CloudflareErrorKind,
  pub(crate) provider_code: Option<i64>,
  #[source]
  source:                   Option<HttpError>,
}
impl CloudflareError {
  /// Provider-specific failure source.
  pub fn kind(&self) -> CloudflareErrorKind {
    self.kind
  }

  /// Application error code, when the provider supplied one.
  pub fn provider_code(&self) -> Option<i64> {
    self.provider_code
  }

  pub(crate) fn decode(source: HttpError) -> Self {
    Self {
      kind:          CloudflareErrorKind::Response,
      provider_code: None,
      source:        Some(source),
    }
  }
}
impl CloudflareErrorKind {
  pub(crate) fn error(self) -> CloudflareError {
    CloudflareError {
      kind:          self,
      provider_code: None,
      source:        None,
    }
  }
}
impl std::fmt::Debug for CloudflareError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}
impl TransportFailure for CloudflareError {
  fn is_transient(&self) -> bool {
    false
  }

  fn is_auth(&self) -> bool {
    false
  }

  fn is_message_rejected(&self) -> bool {
    matches!(
      self.kind,
      CloudflareErrorKind::Input | CloudflareErrorKind::Rejected
    )
  }

  fn acceptance(&self) -> Acceptance {
    if self.kind == CloudflareErrorKind::Response {
      Acceptance::Unknown
    } else {
      Acceptance::NotAccepted
    }
  }
}
