//! Credentials bound to a fixed destination.

use sha2::{Digest, Sha256};

use super::HttpError;

/// Validated authentication material. Debug never exposes credentials.
pub struct Credentials(Authentication);

enum Authentication {
  Bearer(String),
  Basic { key: String, secret: String },
}

impl Credentials {
  /// Validate a visible-ASCII bearer token before any request is possible.
  pub fn bearer(token: impl Into<String>) -> Result<Self, HttpError> {
    let token = token.into();
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_graphic()) {
      return Err(HttpError::Configuration);
    }

    Ok(Self(Authentication::Bearer(token)))
  }

  /// Validate a Basic-auth key and secret. The key cannot contain a colon.
  pub fn basic(
    key: impl Into<String>,
    secret: impl Into<String>,
  ) -> Result<Self, HttpError> {
    let key = key.into();
    let secret = secret.into();
    if key.is_empty()
      || secret.is_empty()
      || key.contains(':')
      || !key
        .bytes()
        .chain(secret.bytes())
        .all(|byte| byte.is_ascii_graphic())
    {
      return Err(HttpError::Configuration);
    }

    Ok(Self(Authentication::Basic { key, secret }))
  }

  /// Explicit secret access for HTTP port implementations only. Do not log it.
  pub fn bearer_token(&self) -> Option<&str> {
    match &self.0 {
      Authentication::Bearer(token) => Some(token),
      Authentication::Basic { .. } => None,
    }
  }

  /// Explicit Basic credentials for HTTP port implementations. Do not log them.
  pub fn basic_pair(&self) -> Option<(&str, &str)> {
    match &self.0 {
      Authentication::Basic { key, secret } => Some((key, secret)),
      Authentication::Bearer(_) => None,
    }
  }

  pub(crate) fn scope(&self, endpoint: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(endpoint.as_bytes());
    hash.update([0]);

    match &self.0 {
      Authentication::Bearer(token) => hash.update(token.as_bytes()),
      Authentication::Basic { key, secret } => {
        hash.update(key.as_bytes());
        hash.update([0]);
        hash.update(secret.as_bytes());
      }
    }

    format!("{:x}", hash.finalize())
  }
}

impl std::fmt::Debug for Credentials {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Credentials([redacted])")
  }
}
