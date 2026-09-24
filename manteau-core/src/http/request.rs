//! The admitted, serialized HTTP request.

use super::{Credentials, HttpConfig, HttpError};
use crate::IdempotencyKey;

/// A serialized request admitted by the core sender. The HTTP adapter supplies
/// mechanics only: send once, do not redirect, enforce timeout and response
/// size.
pub struct HttpRequest<'a> {
  config:      &'a HttpConfig,
  credentials: &'a Credentials,
  key:         Option<&'a IdempotencyKey>,
  body:        Vec<u8>,
}

impl<'a> HttpRequest<'a> {
  pub(crate) fn new(
    config: &'a HttpConfig,
    credentials: &'a Credentials,
    key: Option<&'a IdempotencyKey>,
    payload: &impl serde::Serialize,
  ) -> Result<Self, HttpError> {
    let body = serde_json::to_vec(payload).map_err(HttpError::Encode)?;
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
    &self.body
  }

  /// Transfer the admitted request to an HTTP implementation without copying
  /// private message content. Keep these parts together when dispatching.
  pub fn into_parts(
    self,
  ) -> (
    &'a HttpConfig,
    &'a Credentials,
    Option<&'a IdempotencyKey>,
    Vec<u8>,
  ) {
    (self.config, self.credentials, self.key, self.body)
  }
}
