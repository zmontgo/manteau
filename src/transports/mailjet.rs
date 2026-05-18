//! Mailjet API transport.
//!
//! Posts to Mailjet's v3.1 `/send` endpoint with basic auth. Errors flow
//! through [`MailjetError`] — manteau's standard struct-over-kind shape —
//! and implement [`TransportFailure`] so retry middleware classifies failures
//! by category (transient, auth, message-rejected).

use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;

use crate::{
  message::Message,
  models::{Address, MessageId},
  templating::attributes::urls::Url,
  transport::{Receipt, Transport, TransportFailure},
};

/// Tunable defaults for [`MailjetTransport`]. Construct via
/// [`MailjetConfig::new`] and chain setters to override individual fields.
///
/// ```
/// # use std::time::Duration;
/// # use manteau::MailjetConfig;
/// let config = MailjetConfig::new().timeout(Duration::from_secs(60));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct MailjetConfig {
  /// Per-request total timeout (DNS + connect + TLS + send + receive).
  pub timeout: Duration,
}

impl MailjetConfig {
  pub fn new() -> Self {
    Self {
      timeout: Duration::from_secs(30),
    }
  }

  /// Override the per-request timeout. Default 30 seconds.
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }
}

impl Default for MailjetConfig {
  fn default() -> Self { Self::new() }
}

/// Mailjet API transport — posts to v3.1 `/send` with HTTP Basic auth.
///
/// Requires a key/secret pair from a Mailjet account. Defaults to
/// `https://api.mailjet.com`; override via [`base_url`] for sandbox or
/// proxy environments. Uses [`MailjetConfig::default`] (30s timeout) for
/// per-request budgets; override via [`config`] for tighter or looser.
///
/// One transport per process is the lazy choice — the internal
/// `reqwest::Client` pools connections, so reusing a single transport
/// across many sends is significantly cheaper than constructing one per
/// request.
///
/// ```no_run
/// # use std::time::Duration;
/// # use manteau::{MailjetConfig, MailjetTransport};
/// let transport = MailjetTransport::new("api-key", "api-secret")
///   .config(MailjetConfig::new().timeout(Duration::from_secs(60)));
/// ```
///
/// [`base_url`]: MailjetTransport::base_url
/// [`config`]: MailjetTransport::config
#[derive(Debug, Clone)]
pub struct MailjetTransport {
  api_key:    String,
  api_secret: String,
  base_url:   Url,
  config:     MailjetConfig,
  client:     reqwest::Client,
}

impl MailjetTransport {
  /// Build with the required Mailjet credentials. Defaults applied:
  ///
  /// - `base_url`: `https://api.mailjet.com`
  /// - `config`: [`MailjetConfig::default`] (30s timeout)
  /// - `client`: a fresh [`reqwest::Client`]
  ///
  /// Override any of these by chaining the corresponding setter.
  pub fn new(
    api_key: impl Into<String>,
    api_secret: impl Into<String>,
  ) -> Self {
    Self {
      api_key:    api_key.into(),
      api_secret: api_secret.into(),
      base_url:   Url::try_parse("https://api.mailjet.com")
        .expect("hardcoded URL is valid"),
      config:     MailjetConfig::default(),
      client:     reqwest::Client::default(),
    }
  }

  /// Override the API host. Useful for sandbox environments, proxies, or
  /// fakes (wiremock in integration tests).
  pub fn base_url(mut self, base_url: Url) -> Self {
    self.base_url = base_url;
    self
  }

  /// Override the tunable defaults (timeout, etc.). See [`MailjetConfig`].
  pub fn config(mut self, config: MailjetConfig) -> Self {
    self.config = config;
    self
  }

  /// Override the reqwest client (for shared connection pools across many
  /// transports, custom middleware, etc.).
  pub fn client(mut self, client: reqwest::Client) -> Self {
    self.client = client;
    self
  }
}

/// Acknowledgement returned by [`MailjetTransport::send`] for a successful
/// post.
///
/// `ids` contains one [`MessageId`] per recipient in `to` order — Mailjet
/// assigns separate IDs even when the message body is identical.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct MailjetReceipt {
  pub ids: Vec<MessageId>,
  /// Full JSON response from Mailjet. Exposed as an escape hatch for
  /// fields not surfaced in `ids`; treat its schema as unstable and
  /// provider-controlled — pin to specific JSON paths at your own risk.
  pub raw: serde_json::Value,
}

impl Receipt for MailjetReceipt {
  fn ids(&self) -> &[MessageId] { &self.ids }
}

// ---------- Error machinery ----------

/// Error returned by [`MailjetTransport::send`].
///
/// Follows manteau's standard kind + source pattern (see the
/// [`crate::transport`] module docs). Branch on [`MailjetError::kind`] for
/// programmatic decisions; `Display` carries the upstream's message to
/// operator logs; the original `reqwest::Error` / `serde_json::Error` is
/// preserved in the `#[source]` chain for downcasting.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct MailjetError {
  kind:   MailjetErrorKind,
  #[source]
  source: Box<dyn std::error::Error + Send + Sync>,
}

impl MailjetError {
  /// The category of failure. Callers branch on this for retry policy;
  /// the human-readable message comes from the source chain.
  ///
  /// ```
  /// # use manteau::{MailjetError, MailjetErrorKind};
  /// # fn classify(err: &MailjetError) {
  /// match err.kind() {
  ///   MailjetErrorKind::Network => { /* retry with backoff */ }
  ///   MailjetErrorKind::Auth => { /* refresh credentials */ }
  ///   MailjetErrorKind::Provider { status: 429 } => { /* honor retry-after */ }
  ///   MailjetErrorKind::Provider { status: 500..=599 } => { /* retry */ }
  ///   _ => { /* surface to caller */ }
  /// }
  /// # }
  /// ```
  pub fn kind(&self) -> &MailjetErrorKind { &self.kind }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MailjetErrorKind {
  /// Network-level failure (DNS, connection refused, TLS, timeout).
  /// `is_transient` returns true; retry middleware should back off.
  #[error("network error")]
  Network,
  /// Mailjet rejected the credentials (401/403).
  #[error("authentication rejected")]
  Auth,
  /// Mailjet returned a non-success status. Status code carried on the
  /// variant so retry classification (5xx + 429 transient, other 4xx
  /// message-rejected) does not need to downcast the source.
  #[error("provider rejected request ({status})")]
  Provider { status: u16 },
  /// The 2xx response body did not parse as the expected JSON shape.
  #[error("failed to decode provider response")]
  Decode,
  /// The upstream message could not be rendered. Source chain holds the
  /// underlying [`RenderError`](crate::render::RenderError).
  #[error("failed to render message")]
  Render,
}

impl MailjetErrorKind {
  /// Internal constructor — paired with a source error.
  /// `pub(crate)` so consumers cannot fabricate manteau errors.
  pub(crate) fn err(
    self,
    source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
  ) -> MailjetError {
    MailjetError {
      kind:   self,
      source: source.into(),
    }
  }
}

impl TransportFailure for MailjetError {
  fn is_transient(&self) -> bool {
    match &self.kind {
      MailjetErrorKind::Network => true,
      MailjetErrorKind::Provider { status } => {
        *status == 429 || (500..=599).contains(status)
      }
      _ => false,
    }
  }

  fn is_auth(&self) -> bool { matches!(self.kind, MailjetErrorKind::Auth) }

  fn is_message_rejected(&self) -> bool {
    match &self.kind {
      MailjetErrorKind::Provider { status } => {
        (400..=499).contains(status) && *status != 429
      }
      _ => false,
    }
  }
}

// Source error capturing both the HTTP status and the response body — held
// behind MailjetError's `source: Box<dyn Error>` so the body survives in the
// Display chain without bloating the kind variant.
#[derive(Debug, thiserror::Error)]
#[error("status {status}: {body}")]
struct ProviderRejection {
  status: u16,
  body:   String,
}

// ---------- Serialization wire shape (private to this module) ----------

#[derive(Serialize)]
struct Payload<'a> {
  #[serde(rename = "Messages")]
  messages: Vec<MailjetMessage<'a>>,
}

#[derive(Serialize)]
struct MailjetMessage<'a> {
  #[serde(rename = "From")]
  from:      AddrRef<'a>,
  #[serde(rename = "To")]
  to:        Vec<AddrRef<'a>>,
  #[serde(rename = "Cc", skip_serializing_if = "Vec::is_empty")]
  cc:        Vec<AddrRef<'a>>,
  #[serde(rename = "Bcc", skip_serializing_if = "Vec::is_empty")]
  bcc:       Vec<AddrRef<'a>>,
  #[serde(rename = "Subject")]
  subject:   &'a str,
  #[serde(rename = "TextPart")]
  text_part: &'a str,
  #[serde(rename = "HTMLPart")]
  html_part: &'a str,
}

#[derive(Serialize)]
struct AddrRef<'a> {
  #[serde(rename = "Email")]
  email: &'a str,
  #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
  name:  Option<&'a str>,
}

impl<'a> From<&'a Address> for AddrRef<'a> {
  fn from(a: &'a Address) -> Self {
    AddrRef {
      email: a.email.as_str(),
      name:  a.name.as_deref(),
    }
  }
}

// ---------- Implementation ----------

#[async_trait]
impl Transport for MailjetTransport {
  type Error = MailjetError;
  type Receipt = MailjetReceipt;

  #[tracing::instrument(skip_all, fields(base_url = %self.base_url.as_str()))]
  async fn send(
    &self,
    message: &Message,
  ) -> Result<MailjetReceipt, MailjetError> {
    let rendered = message
      .render()
      .map_err(|e| MailjetErrorKind::Render.err(e))?;

    let payload = Payload {
      messages: vec![MailjetMessage {
        from:      (&message.from).into(),
        to:        message.to.iter().map(Into::into).collect(),
        cc:        message.cc.iter().map(Into::into).collect(),
        bcc:       message.bcc.iter().map(Into::into).collect(),
        subject:   &message.subject,
        text_part: &rendered.text,
        html_part: &rendered.html,
      }],
    };

    let url = format!("{}/v3.1/send", self.base_url.as_str());
    let resp = self
      .client
      .post(&url)
      .timeout(self.config.timeout)
      .basic_auth(&self.api_key, Some(&self.api_secret))
      .json(&payload)
      .send()
      .await
      .map_err(|e| MailjetErrorKind::Network.err(e))?;

    let status = resp.status();
    if !status.is_success() {
      let status_u16 = status.as_u16();
      let body = resp.text().await.unwrap_or_default();
      let rejection = ProviderRejection {
        status: status_u16,
        body,
      };
      let kind = match status_u16 {
        401 | 403 => MailjetErrorKind::Auth,
        _ => MailjetErrorKind::Provider { status: status_u16 },
      };
      tracing::error!(kind = ?kind, "mailjet send failed");
      return Err(kind.err(rejection));
    }

    let raw: serde_json::Value = resp
      .json()
      .await
      .map_err(|e| MailjetErrorKind::Decode.err(e))?;

    let ids = raw
      .get("Messages")
      .and_then(|m| m.as_array())
      .map(|arr| {
        arr
          .iter()
          .filter_map(|m| m.get("To"))
          .filter_map(|to| to.as_array())
          .flatten()
          .filter_map(|t| t.get("MessageID"))
          .map(|id| match id {
            serde_json::Value::String(s) => MessageId::new(s.clone()),
            other => MessageId::new(other.to_string()),
          })
          .collect()
      })
      .unwrap_or_default();

    Ok(MailjetReceipt { ids, raw })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn err(kind: MailjetErrorKind) -> MailjetError {
    MailjetError {
      kind,
      source: "test".into(),
    }
  }

  #[test]
  fn network_is_transient() {
    let e = err(MailjetErrorKind::Network);
    assert!(e.is_transient());
    assert!(!e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn auth_is_only_auth() {
    let e = err(MailjetErrorKind::Auth);
    assert!(!e.is_transient());
    assert!(e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn provider_5xx_is_transient() {
    for status in [500u16, 502, 503, 504, 599] {
      let e = err(MailjetErrorKind::Provider { status });
      assert!(e.is_transient(), "status {status} should be transient");
      assert!(!e.is_message_rejected(), "status {status}");
    }
  }

  #[test]
  fn provider_429_is_transient_not_rejected() {
    let e = err(MailjetErrorKind::Provider { status: 429 });
    assert!(e.is_transient());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn provider_4xx_non_429_is_rejected() {
    for status in [400u16, 404, 422] {
      let e = err(MailjetErrorKind::Provider { status });
      assert!(!e.is_transient(), "status {status}");
      assert!(
        e.is_message_rejected(),
        "status {status} should be rejected"
      );
    }
  }

  #[test]
  fn render_failure_is_neither() {
    let e = err(MailjetErrorKind::Render);
    assert!(!e.is_transient());
    assert!(!e.is_auth());
    assert!(!e.is_message_rejected());
  }
}
