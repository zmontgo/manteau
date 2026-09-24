//! The bounded physical response.

use std::time::Duration;

use super::HttpError;

/// Bounded response provided by an HTTP implementation. The driver must apply
/// request limits before constructing it; only a request or its response guard
/// can construct this value.
pub struct HttpResponse {
  status:      u16,
  retry_after: Option<Duration>,
  body:        Vec<u8>,
}

impl HttpResponse {
  pub(crate) fn new(
    status: u16,
    retry_after: Option<Duration>,
    body: Vec<u8>,
  ) -> Self {
    Self {
      status,
      retry_after,
      body,
    }
  }

  /// Physical HTTP status, independent of provider application-level status.
  pub fn status(&self) -> u16 {
    self.status
  }

  /// Provider-requested delay, not evidence that a retry is safe.
  pub fn retry_after(&self) -> Option<Duration> {
    self.retry_after
  }

  /// Decode into a provider-owned response type without intermediate JSON
  /// cloning.
  pub fn decode<T: serde::de::DeserializeOwned>(&self) -> Result<T, HttpError> {
    serde_json::from_slice(&self.body).map_err(HttpError::Decode)
  }
}
