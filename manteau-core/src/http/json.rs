//! Serialized JSON admitted for one HTTP submission.

use super::HttpError;

/// A complete JSON value encoded from a provider's admitted payload. Raw
/// bytes cannot be wrapped by a transport implementation.
pub struct JsonBody(Vec<u8>);

impl JsonBody {
  pub(crate) fn encode(
    value: &impl serde::Serialize,
  ) -> Result<Self, HttpError> {
    serde_json::to_vec(value)
      .map(Self)
      .map_err(HttpError::Encode)
  }

  /// Exact bytes sent to the provider; these may contain private mail.
  pub fn as_bytes(&self) -> &[u8] {
    &self.0
  }

  /// Transfer bytes to the physical HTTP client without another allocation.
  pub fn into_bytes(self) -> Vec<u8> {
    self.0
  }
}

impl std::fmt::Debug for JsonBody {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("JsonBody([redacted])")
  }
}
