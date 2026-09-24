use manteau_core::{Acceptance, TransportFailure, http::HttpError};
/// Mailjet-specific response or input failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MailjetErrorKind {
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
#[error("Mailjet protocol failed: {kind:?}")]
pub struct MailjetError {
  pub(crate) kind:          MailjetErrorKind,
  pub(crate) provider_code: Option<String>,
  #[source]
  source:                   Option<HttpError>,
}

impl MailjetError {
  /// Provider-specific failure source.
  pub fn kind(&self) -> MailjetErrorKind {
    self.kind
  }

  /// Standardized Mailjet error code, when the response supplies one.
  pub fn provider_code(&self) -> Option<&str> {
    self.provider_code.as_deref()
  }

  pub(crate) fn decode(source: HttpError) -> Self {
    Self {
      kind:          MailjetErrorKind::Response,
      provider_code: None,
      source:        Some(source),
    }
  }
}

impl MailjetErrorKind {
  pub(crate) fn error(self) -> MailjetError {
    MailjetError {
      kind:          self,
      provider_code: None,
      source:        None,
    }
  }
}

impl std::fmt::Debug for MailjetError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

impl TransportFailure for MailjetError {
  fn is_transient(&self) -> bool {
    false
  }

  fn is_auth(&self) -> bool {
    false
  }

  fn is_message_rejected(&self) -> bool {
    matches!(
      self.kind,
      MailjetErrorKind::Input | MailjetErrorKind::Rejected
    )
  }

  fn acceptance(&self) -> Acceptance {
    if self.kind == MailjetErrorKind::Response {
      Acceptance::Unknown
    } else {
      Acceptance::NotAccepted
    }
  }
}
