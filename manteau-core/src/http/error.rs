//! Errors at the physical HTTP boundary.

/// Shared physical-request failure classification. Formatting is sanitized;
/// source details are accessible explicitly for diagnostics.
#[derive(thiserror::Error)]
pub enum HttpError {
  /// Invalid local endpoint, credentials, timeout, or client configuration.
  #[error("invalid HTTP submission configuration")]
  Configuration,
  /// Request encoding failed before dispatch.
  #[error("could not encode provider request")]
  Encode(#[source] serde_json::Error),
  /// Remote acceptance cannot be excluded after transport failure.
  #[error("HTTP submission outcome is unknown")]
  Network(#[source] Box<dyn std::error::Error + Send + Sync>),
  /// Response exceeded the configured byte limit; acceptance remains unknown.
  #[error("HTTP response exceeded its size limit")]
  ResponseLimit,
  /// Driver returned a status outside the HTTP response status range.
  #[error("invalid HTTP response status")]
  InvalidResponseStatus,
  /// Response did not match the documented representation.
  #[error("invalid provider response")]
  Decode(#[source] serde_json::Error),
}

impl std::fmt::Debug for HttpError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}
