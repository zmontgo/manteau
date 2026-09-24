//! JetEmail HTTP sending with provider-enforced idempotency.
use std::time::Duration;

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use manteau_core::{
  IdempotencyKey, IdempotentTransport, Message, MessageId, PreparedMessage,
  Receipt, Transport, TransportFailure,
};

/// Connection policy. Keep the endpoint and account fixed across retries;
/// JetEmail's key store is region-scoped. Redirects and automatic retries are
/// disabled so a submission cannot silently move to another endpoint.
pub struct JetEmailConfig {
  /// Send endpoint, including `/email`.
  pub endpoint: String,
  /// Upper bound on one physical HTTP request.
  pub timeout:  Duration,
}
impl Default for JetEmailConfig {
  fn default() -> Self {
    Self {
      endpoint: "https://api.jetemail.com/email".into(),
      timeout:  Duration::from_secs(30),
    }
  }
}
/// Authenticated client. Credentials and message contents are never Debug.
pub struct JetEmailTransport {
  scope:         String,
  timeout:       Duration,
  client:        reqwest::Client,
  endpoint:      reqwest::Url,
  authorization: reqwest::header::HeaderValue,
}
impl JetEmailTransport {
  /// Construct a fixed-endpoint client with a transactional API token.
  pub fn new(
    token: &str,
    config: JetEmailConfig,
  ) -> Result<Self, JetEmailError> {
    let endpoint = reqwest::Url::parse(&config.endpoint)
      .map_err(|_| JetEmailErrorKind::Configuration.error())?;
    if token.is_empty()
      || !token.bytes().all(|b| b.is_ascii_graphic())
      || config.timeout.is_zero()
      || !endpoint.username().is_empty()
      || endpoint.password().is_some()
      || endpoint.query().is_some()
      || endpoint.fragment().is_some()
      || !(endpoint.scheme() == "https"
        || (endpoint.scheme() == "http"
          && matches!(
            endpoint.host_str(),
            Some("127.0.0.1" | "[::1]" | "localhost")
          )))
    {
      return Err(JetEmailErrorKind::Configuration.error());
    }
    let mut authorization =
      reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| JetEmailErrorKind::Configuration.error())?;
    authorization.set_sensitive(true);
    let client = reqwest::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .retry(reqwest::retry::never())
      .timeout(config.timeout)
      .build()
      .map_err(|_| JetEmailErrorKind::Configuration.error())?;
    let mut fingerprint = Sha256::new();
    fingerprint.update(endpoint.as_str().as_bytes());
    fingerprint.update([0]);
    fingerprint.update(token.as_bytes());
    let scope = format!("jetemail:{:x}", fingerprint.finalize());
    Ok(Self {
      client,
      endpoint,
      authorization,
      scope,
      timeout: config.timeout,
    })
  }

  async fn submit(
    &self,
    key: Option<&IdempotencyKey>,
    message: &PreparedMessage,
  ) -> Result<JetEmailReceipt, JetEmailError> {
    let mut request = self
      .client
      .post(self.endpoint.clone())
      .header(reqwest::header::AUTHORIZATION, self.authorization.clone())
      .json(message);
    if let Some(key) = key {
      request = request.header("Idempotency-Key", key.as_str());
    }
    let response = request
      .send()
      .await
      .map_err(|_| JetEmailErrorKind::Uncertain.error())?;
    let status = response.status().as_u16();
    let retry_after = response
      .headers()
      .get("Retry-After")
      .and_then(|h| h.to_str().ok())
      .and_then(|v| v.parse::<u64>().ok())
      .map(Duration::from_secs);
    if status != 201 {
      let kind = match status {
        401 | 403 => JetEmailErrorKind::Authentication,
        429 => JetEmailErrorKind::RateLimited,
        409 => {
          let body: serde_json::Value = response
            .json()
            .await
            .map_err(|_| JetEmailErrorKind::Uncertain.error())?;
          match body
            .get("code")
            .and_then(|v| v.as_str())
            .or_else(|| body.get("error").and_then(|v| v.as_str()))
          {
            Some("IDEMPOTENCY_IN_FLIGHT") => JetEmailErrorKind::InFlight,
            Some("IDEMPOTENCY_BODY_MISMATCH") => JetEmailErrorKind::Conflict,
            _ => JetEmailErrorKind::Uncertain,
          }
        }
        400 | 413 | 422 => JetEmailErrorKind::Rejected,
        _ => JetEmailErrorKind::Uncertain,
      };
      return Err(JetEmailError { kind, retry_after });
    }
    let accepted: Accepted = response
      .json()
      .await
      .map_err(|_| JetEmailErrorKind::Uncertain.error())?;
    if accepted.id.is_empty()
      || accepted.id.len() > 998
      || accepted.id.chars().any(char::is_control)
    {
      return Err(JetEmailErrorKind::Uncertain.error());
    }
    Ok(JetEmailReceipt {
      ids: [MessageId::new(accepted.id)],
    })
  }
}
#[derive(serde::Deserialize)]
struct Accepted {
  id: String,
}
/// Acceptance into JetEmail's queue, not proof of delivery.
#[derive(Debug)]
pub struct JetEmailReceipt {
  ids: [MessageId; 1],
}
impl Receipt for JetEmailReceipt {
  fn ids(&self) -> &[MessageId] { &self.ids }
}

/// Sanitized failure classification; provider bodies may contain private mail.
#[derive(Debug, PartialEq, Eq)]
pub enum JetEmailErrorKind {
  /// Invalid local client configuration.
  Configuration,
  /// Invalid rendered envelope or template.
  Preparation,
  /// Credentials or account permissions were rejected.
  Authentication,
  /// Provider rate limit; retry only within the idempotency window.
  RateLimited,
  /// An identical submission is still being processed.
  InFlight,
  /// The key was already used with different content.
  Conflict,
  /// The provider rejected the request.
  Rejected,
  /// Acceptance cannot be ruled out; never retry an unkeyed send.
  Uncertain,
}
impl JetEmailErrorKind {
  fn error(self) -> JetEmailError {
    JetEmailError {
      kind:        self,
      retry_after: None,
    }
  }
}
/// Failure without credentials, recipient addresses, or provider response text.
#[derive(Debug, thiserror::Error)]
#[error("JetEmail submission failed: {kind:?}")]
pub struct JetEmailError {
  kind:        JetEmailErrorKind,
  retry_after: Option<Duration>,
}
impl JetEmailError {
  /// Classification used for retry and operator decisions.
  pub fn kind(&self) -> &JetEmailErrorKind { &self.kind }
}
impl TransportFailure for JetEmailError {
  // Unknown acceptance is deliberately not a generic transient failure.
  fn is_transient(&self) -> bool {
    matches!(
      self.kind,
      JetEmailErrorKind::RateLimited | JetEmailErrorKind::InFlight
    )
  }

  fn is_auth(&self) -> bool { self.kind == JetEmailErrorKind::Authentication }

  fn is_message_rejected(&self) -> bool {
    matches!(
      self.kind,
      JetEmailErrorKind::Rejected
        | JetEmailErrorKind::Conflict
        | JetEmailErrorKind::Preparation
    )
  }

  fn retry_after(&self) -> Option<Duration> { self.retry_after }
}
#[async_trait]
impl Transport for JetEmailTransport {
  type Error = JetEmailError;
  type Receipt = JetEmailReceipt;

  async fn send(
    &self,
    message: &Message,
  ) -> Result<Self::Receipt, Self::Error> {
    let prepared = PreparedMessage::render(message)
      .map_err(|_| JetEmailErrorKind::Preparation.error())?;
    self.submit(None, &prepared).await
  }
}
#[async_trait]
impl IdempotentTransport for JetEmailTransport {
  fn scope(&self) -> &str { &self.scope }

  fn request_timeout(&self) -> Duration { self.timeout }

  fn retention(&self) -> Duration { Duration::from_secs(24 * 60 * 60) }

  async fn send_idempotent(
    &self,
    key: &IdempotencyKey,
    message: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error> {
    self.submit(Some(key), message).await
  }
}
