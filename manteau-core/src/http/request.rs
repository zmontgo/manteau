//! The admitted, serialized HTTP request.

use super::{Credentials, HttpConfig, HttpError, HttpResponse, JsonBody};
use std::time::Duration;
use crate::IdempotencyKey;

/// A serialized request admitted by the core sender. The HTTP adapter supplies
/// mechanics only: send once, do not redirect, enforce timeout and response
/// size.
pub struct HttpRequest<'a> {
  config:      &'a HttpConfig,
  credentials: &'a Credentials,
  key:         Option<&'a IdempotencyKey>,
  body:        JsonBody,
}

/// Response-admission capability tied to the exact dispatched request bounds.
/// It survives transfer of the request body into an HTTP client.
pub struct ResponseGuard<'a> {
  config: &'a HttpConfig,
}

impl ResponseGuard<'_> {
  /// Check a physical response before exposing it to core or a provider.
  pub fn response(
    self,
    status: u16,
    retry_after: Option<Duration>,
    body: Vec<u8>,
  ) -> Result<HttpResponse, HttpError> {
    if !(100..=599).contains(&status) {
      return Err(HttpError::InvalidResponseStatus);
    }
    if body.len() > self.config.max_response_bytes() {
      return Err(HttpError::ResponseLimit);
    }

    Ok(HttpResponse::new(status, retry_after, body))
  }
}

impl<'a> HttpRequest<'a> {
  pub(crate) fn new(
    config: &'a HttpConfig,
    credentials: &'a Credentials,
    key: Option<&'a IdempotencyKey>,
    payload: &impl serde::Serialize,
  ) -> Result<Self, HttpError> {
    let body = JsonBody::encode(payload)?;
    Ok(Self {
      config,
      credentials,
      key,
      body,
    })
  }

  /// Validated endpoint and request bounds.
  pub fn config(&self) -> &HttpConfig {
    self.config
  }

  /// Credentials bound to this request. Do not send to another endpoint.
  pub fn credentials(&self) -> &Credentials {
    self.credentials
  }

  /// Optional fixed identity for provider-enforced deduplication.
  pub fn key(&self) -> Option<&IdempotencyKey> {
    self.key
  }

  /// Exact serialized JSON; contains private mail.
  pub fn body(&self) -> &[u8] {
    self.body.as_bytes()
  }

  /// Admit a physical response under this request's byte limit. The HTTP
  /// implementation must still stop reading once that limit is reached.
  pub fn response(
    self,
    status: u16,
    retry_after: Option<Duration>,
    body: Vec<u8>,
  ) -> Result<HttpResponse, HttpError> {
    ResponseGuard { config: self.config }.response(status, retry_after, body)
  }

  /// Transfer the admitted request to an HTTP implementation without copying
  /// private message content. Keep these parts together when dispatching.
  pub fn into_parts(
    self,
  ) -> (
    &'a HttpConfig,
    &'a Credentials,
    Option<&'a IdempotencyKey>,
    JsonBody,
    ResponseGuard<'a>,
  ) {
    (self.config, self.credentials, self.key, self.body, ResponseGuard { config: self.config })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn response_is_bounded_by_its_dispatched_request() {
    let config = HttpConfig::new("https://example.com/send")
      .unwrap()
      .response_limit(std::num::NonZeroUsize::new(4).unwrap());
    let credentials = Credentials::bearer("private-token").unwrap();
    let request = HttpRequest::new(&config, &credentials, None, &serde_json::json!({})).unwrap();
    assert!(matches!(request.response(200, None, vec![0; 5]), Err(HttpError::ResponseLimit)));

    let request = HttpRequest::new(&config, &credentials, None, &serde_json::json!({})).unwrap();
    assert!(matches!(request.response(700, None, vec![]), Err(HttpError::InvalidResponseStatus)));
  }
}
