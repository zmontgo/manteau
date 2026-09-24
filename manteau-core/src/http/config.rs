//! Validated HTTP destination and request bounds.

use std::{num::NonZeroUsize, time::Duration};

use url::Url;

use super::HttpError;

/// Validated request controls. The endpoint is fixed before credentials bind.
#[derive(Clone)]
pub struct HttpConfig {
  endpoint:       Url,
  timeout:        Duration,
  response_limit: NonZeroUsize,
}

impl HttpConfig {
  /// Accept HTTPS without userinfo, query, or fragment; allow HTTP only for
  /// literal loopback IPs. Pass the complete send endpoint including its path.
  pub fn new(endpoint: &str) -> Result<Self, HttpError> {
    let endpoint =
      Url::parse(endpoint).map_err(|_| HttpError::Configuration)?;
    let loopback = match endpoint.host() {
      Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
      Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
      _ => false,
    };

    if !endpoint.username().is_empty()
      || endpoint.password().is_some()
      || endpoint.query().is_some()
      || endpoint.fragment().is_some()
      || endpoint.host().is_none()
      || !(endpoint.scheme() == "https"
        || (endpoint.scheme() == "http" && loopback))
    {
      return Err(HttpError::Configuration);
    }

    Ok(Self {
      endpoint,
      timeout: Duration::from_secs(30),
      response_limit: NonZeroUsize::new(1024 * 1024)
        .expect("positive response limit"),
    })
  }

  /// Bound an entire request, including response-body reads. Zero is rejected.
  pub fn timeout(mut self, timeout: Duration) -> Result<Self, HttpError> {
    if timeout.is_zero() {
      return Err(HttpError::Configuration);
    }

    self.timeout = timeout;
    Ok(self)
  }

  /// Bound memory used by a provider response; default is one MiB.
  pub fn response_limit(mut self, bytes: NonZeroUsize) -> Self {
    self.response_limit = bytes;
    self
  }

  /// Fixed destination. Treat custom endpoints as trusted credential
  /// recipients.
  pub fn endpoint(&self) -> &str {
    self.endpoint.as_str()
  }

  /// Maximum request duration, including body decoding input.
  pub fn request_timeout(&self) -> Duration {
    self.timeout
  }

  /// Maximum permitted response bytes.
  pub fn max_response_bytes(&self) -> usize {
    self.response_limit.get()
  }
}

impl std::fmt::Debug for HttpConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("HttpConfig")
      .field("timeout", &self.timeout)
      .field("response_limit", &self.response_limit)
      .finish_non_exhaustive()
  }
}
