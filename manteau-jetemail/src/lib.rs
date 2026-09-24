//! JetEmail protocol and idempotency capabilities. This adapter performs no
//! I/O.
#![deny(missing_docs)]

use std::time::Duration;

use manteau_core::{
  Acceptance, HttpProvider, IdempotentProvider, MessageId, PreparedMessage,
  Receipt, StatusPolicy, TransportFailure,
  http::{Credentials, HttpConfig, HttpError, HttpResponse},
};

/// Configured JetEmail protocol, with fixed credentials and routing.
#[derive(Debug)]
pub struct JetEmail {
  config:      HttpConfig,
  credentials: Credentials,
}

impl JetEmail {
  /// Use the standard transactional send endpoint.
  pub fn new(token: &str) -> Result<Self, HttpError> {
    Self::with_config(token, HttpConfig::new("https://api.jetemail.com/email")?)
  }

  /// Bind a token to a trusted endpoint and bounded request policy.
  pub fn with_config(
    token: &str,
    config: HttpConfig,
  ) -> Result<Self, HttpError> {
    Ok(Self {
      config,
      credentials: Credentials::bearer(token)?,
    })
  }
}

impl HttpProvider for JetEmail {
  type Error = JetEmailError;
  type Payload<'a> = Payload<'a>;
  type Receipt = JetEmailReceipt;

  fn config(&self) -> &HttpConfig {
    &self.config
  }

  fn credentials(&self) -> &Credentials {
    &self.credentials
  }

  fn protocol_scope(&self) -> &'static str {
    concat!("jetemail/", env!("CARGO_PKG_VERSION"))
  }

  fn status_policy(&self) -> StatusPolicy {
    const POLICY: StatusPolicy = StatusPolicy::new(&[201, 409], &[401, 403]);
    POLICY
  }

  fn request<'a>(
    &'a self,
    message: &'a PreparedMessage,
  ) -> Result<Payload<'a>, Self::Error> {
    if message.envelope().recipients().to_addresses().is_empty()
      || message.envelope().subject().is_empty()
    {
      return Err(JetEmailErrorKind::Input.error());
    }
    Ok(Payload::from(message))
  }

  fn receipt(
    &self,
    response: HttpResponse,
    _: &PreparedMessage,
  ) -> Result<JetEmailReceipt, JetEmailError> {
    if response.status() == 409 {
      let conflict: Conflict =
        response.decode().map_err(JetEmailError::decode)?;
      let kind = match conflict.code.as_deref().or(conflict.error.as_deref()) {
        Some("IDEMPOTENCY_IN_FLIGHT") => JetEmailErrorKind::InFlight,
        Some("IDEMPOTENCY_BODY_MISMATCH") => JetEmailErrorKind::Conflict,
        _ => JetEmailErrorKind::Response,
      };
      let mut error = kind.error();
      error.retry_after = response.retry_after();
      return Err(error);
    }
    let accepted: Accepted =
      response.decode().map_err(JetEmailError::decode)?;
    if accepted.id.len() > 998 {
      return Err(JetEmailErrorKind::Response.error());
    }
    Ok(JetEmailReceipt {
      ids: [MessageId::new(accepted.id)
        .map_err(|_| JetEmailErrorKind::Response.error())?],
    })
  }
}

impl IdempotentProvider for JetEmail {
  fn retention(&self) -> Duration {
    Duration::from_secs(24 * 60 * 60)
  }
}

#[derive(serde::Deserialize)]
struct Accepted {
  id: String,
}

#[derive(serde::Deserialize)]
struct Conflict {
  code:  Option<String>,
  error: Option<String>,
}

/// Acceptance into JetEmail's queue, not proof of delivery.
pub struct JetEmailReceipt {
  ids: [MessageId; 1],
}

impl std::fmt::Debug for JetEmailReceipt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("JetEmailReceipt([redacted])")
  }
}

impl Receipt for JetEmailReceipt {
  fn ids(&self) -> &[MessageId] {
    &self.ids
  }
}

/// JetEmail-specific input and response failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JetEmailErrorKind {
  /// JetEmail requires a nonempty To collection and subject.
  Input,
  /// A keyed request is still being processed; no new identity is permitted.
  InFlight,
  /// The key already identifies different content. Do not replace it to retry.
  Conflict,
  /// Acceptance evidence is malformed or unknown.
  Response,
}

/// Sanitized protocol failure retaining the original decoder error when
/// relevant.
#[derive(thiserror::Error)]
#[error("JetEmail protocol failed: {kind:?}")]
pub struct JetEmailError {
  kind:        JetEmailErrorKind,
  retry_after: Option<Duration>,
  #[source]
  source:      Option<HttpError>,
}

impl JetEmailError {
  /// Provider-specific failure source.
  pub fn kind(&self) -> JetEmailErrorKind {
    self.kind
  }

  fn decode(source: HttpError) -> Self {
    Self {
      kind:        JetEmailErrorKind::Response,
      retry_after: None,
      source:      Some(source),
    }
  }
}

impl JetEmailErrorKind {
  fn error(self) -> JetEmailError {
    JetEmailError {
      kind:        self,
      retry_after: None,
      source:      None,
    }
  }
}

impl std::fmt::Debug for JetEmailError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

impl TransportFailure for JetEmailError {
  fn is_transient(&self) -> bool {
    self.kind == JetEmailErrorKind::InFlight
  }

  fn is_auth(&self) -> bool {
    false
  }

  fn is_message_rejected(&self) -> bool {
    matches!(
      self.kind,
      JetEmailErrorKind::Input | JetEmailErrorKind::Conflict
    )
  }

  fn acceptance(&self) -> Acceptance {
    if self.kind == JetEmailErrorKind::Input {
      Acceptance::NotAccepted
    } else {
      Acceptance::Unknown
    }
  }

  fn retry_after(&self) -> Option<Duration> {
    self.retry_after
  }
}

/// Provider payload available only through checked admission.
#[derive(serde::Serialize)]
pub struct Payload<'a> {
  from:    String,
  to:      Vec<String>,
  cc:      Vec<String>,
  bcc:     Vec<String>,
  subject: &'a str,
  html:    &'a str,
  text:    &'a str,
}

impl<'a> From<&'a PreparedMessage> for Payload<'a> {
  fn from(message: &'a PreparedMessage) -> Self {
    let envelope = message.envelope();
    let recipients = envelope.recipients();
    Self {
      from:    envelope.from().mailbox(),
      to:      recipients
        .to_addresses()
        .iter()
        .map(manteau_core::Address::mailbox)
        .collect(),
      cc:      recipients
        .cc_addresses()
        .iter()
        .map(manteau_core::Address::mailbox)
        .collect(),
      bcc:     recipients
        .bcc_addresses()
        .iter()
        .map(manteau_core::Address::mailbox)
        .collect(),
      subject: envelope.subject(),
      html:    message.body().html(),
      text:    message.body().text(),
    }
  }
}
