//! Cloudflare protocol implementation for Manteau's core sender.
//! This crate performs no I/O. Supply a core Http implementation to Sender.
use manteau_core::{
  Address, EmailAddress, HttpProvider, MessageId, PreparedMessage, Receipt,
  StatusPolicy,
  http::{Credentials, HttpConfig, HttpError, HttpResponse},
};
use serde::{Deserialize, Serialize};
mod error;
pub use error::{CloudflareError, CloudflareErrorKind};
mod header;
use header::Header;

/// Configured Cloudflare protocol. Credentials and endpoint are fixed at
/// construction.
#[derive(Debug)]
pub struct Cloudflare {
  config:      HttpConfig,
  credentials: Credentials,
}
impl Cloudflare {
  /// Bind a token to a Cloudflare account. Path delimiters are not valid IDs.
  pub fn new(account: &str, token: &str) -> Result<Self, HttpError> {
    if account.is_empty()
      || !account
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
      return Err(HttpError::Configuration);
    }
    Self::with_config(token, HttpConfig::new(&format!("https://api.cloudflare.com/client/v4/accounts/{account}/email/sending/send"))?)
  }

  /// Bind a token to a complete trusted account-scoped endpoint.
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
/// Complete recipient report from Cloudflare. Success of the HTTP operation
/// does not imply success for every recipient. IDs are genuine provider
/// identifiers.
pub struct CloudflareReceipt {
  ids:               Vec<MessageId>,
  delivered:         Vec<EmailAddress>,
  queued:            Vec<EmailAddress>,
  permanent_bounces: Vec<EmailAddress>,
  suppressed:        Vec<EmailAddress>,
}

impl std::fmt::Debug for CloudflareReceipt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CloudflareReceipt")
      .field("delivered", &self.delivered.len())
      .field("queued", &self.queued.len())
      .field("permanent_bounces", &self.permanent_bounces.len())
      .field("suppressed", &self.suppressed.len())
      .finish_non_exhaustive()
  }
}
impl CloudflareReceipt {
  /// Recipients the provider reports as immediately delivered.
  pub fn delivered(&self) -> &[EmailAddress] {
    &self.delivered
  }

  /// Recipients accepted into the provider's queue; delivery remains pending.
  pub fn queued(&self) -> &[EmailAddress] {
    &self.queued
  }

  /// Recipients rejected permanently; other recipients may have succeeded.
  pub fn permanent_bounces(&self) -> &[EmailAddress] {
    &self.permanent_bounces
  }

  /// Recipients skipped by provider suppression policy.
  pub fn suppressed(&self) -> &[EmailAddress] {
    &self.suppressed
  }
}
impl Receipt for CloudflareReceipt {
  fn ids(&self) -> &[MessageId] {
    &self.ids
  }
}
/// Provider request produced only through checked admission.
#[derive(Serialize)]
pub struct Payload<'a> {
  from:    Mailbox<'a>,
  to:      Vec<Mailbox<'a>>,
  cc:      Vec<Mailbox<'a>>,
  bcc:     Vec<Mailbox<'a>>,
  subject: std::borrow::Cow<'a, str>,
  html:    &'a str,
  text:    &'a str,
}
#[derive(Serialize)]
struct Mailbox<'a> {
  address: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  name:    Option<std::borrow::Cow<'a, str>>,
}

impl<'a> From<&'a Address> for Mailbox<'a> {
  fn from(address: &'a Address) -> Self {
    Self {
      address: address.email().as_str(),
      name:    address.display_name().map(|name| Header(name).encode()),
    }
  }
}
#[derive(Deserialize)]
struct Response {
  success: bool,
  result:  Option<Outcomes>,
  #[serde(default)]
  errors:  Vec<ApiError>,
}
#[derive(Deserialize)]
struct ApiError {
  code: i64,
}
#[derive(Deserialize)]
struct Outcomes {
  message_id:            String,
  delivered:             Vec<EmailAddress>,
  queued:                Vec<EmailAddress>,
  permanent_bounces:     Vec<EmailAddress>,
  #[serde(default)]
  suppressed_recipients: Vec<EmailAddress>,
}
impl Response {
  fn into_receipt(
    self,
    message: &PreparedMessage,
  ) -> Result<CloudflareReceipt, CloudflareError> {
    if !self.success {
      if self.result.is_some() {
        return Err(CloudflareErrorKind::Response.error());
      }

      let mut error = CloudflareErrorKind::Rejected.error();
      error.provider_code = self.errors.first().map(|e| e.code);
      return Err(error);
    }
    let result = self
      .result
      .ok_or_else(|| CloudflareErrorKind::Response.error())?;
    let expected: std::collections::HashSet<_> = message
      .envelope()
      .recipients()
      .iter()
      .map(|a| a.email())
      .collect();
    let mut observed = std::collections::HashSet::new();
    for address in result
      .delivered
      .iter()
      .chain(&result.queued)
      .chain(&result.permanent_bounces)
      .chain(&result.suppressed_recipients)
    {
      if !expected.contains(address) || !observed.insert(address) {
        return Err(CloudflareErrorKind::Response.error());
      }
    }
    if observed != expected || result.message_id.chars().any(char::is_control) {
      return Err(CloudflareErrorKind::Response.error());
    }
    let mut ids = Vec::new();
    if !result.message_id.is_empty() {
      ids.push(MessageId::new(result.message_id));
    } else if !result.delivered.is_empty() || !result.queued.is_empty() {
      return Err(CloudflareErrorKind::Response.error());
    }
    Ok(CloudflareReceipt {
      ids,
      delivered: result.delivered,
      queued: result.queued,
      permanent_bounces: result.permanent_bounces,
      suppressed: result.suppressed_recipients,
    })
  }
}

impl HttpProvider for Cloudflare {
  type Error = CloudflareError;
  type Payload<'a> = Payload<'a>;
  type Receipt = CloudflareReceipt;

  fn config(&self) -> &HttpConfig {
    &self.config
  }

  fn credentials(&self) -> &Credentials {
    &self.credentials
  }

  fn protocol_scope(&self) -> &'static str {
    concat!("cloudflare/", env!("CARGO_PKG_VERSION"))
  }

  fn status_policy(&self) -> StatusPolicy {
    StatusPolicy {
      handled:      &[200],
      not_accepted: &[401, 403],
    }
  }

  fn request<'a>(
    &'a self,
    message: &'a PreparedMessage,
  ) -> Result<Payload<'a>, Self::Error> {
    let envelope = message.envelope();
    let recipients = envelope.recipients();
    let mut unique = std::collections::HashSet::new();
    let count = recipients.iter().count();
    if envelope.subject().is_empty()
      || count > 50
      || !recipients
        .iter()
        .all(|address| unique.insert(address.email()))
    {
      return Err(CloudflareErrorKind::Input.error());
    }

    Ok(Payload {
      from:    envelope.from().into(),
      to:      recipients.to_addresses().iter().map(Into::into).collect(),
      cc:      recipients.cc_addresses().iter().map(Into::into).collect(),
      bcc:     recipients.bcc_addresses().iter().map(Into::into).collect(),
      subject: Header(envelope.subject()).encode(),
      html:    message.body().html(),
      text:    message.body().text(),
    })
  }

  fn receipt(
    &self,
    response: HttpResponse,
    message: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error> {
    response
      .decode::<Response>()
      .map_err(CloudflareError::decode)?
      .into_receipt(message)
  }
}
