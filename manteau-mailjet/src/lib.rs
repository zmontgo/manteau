//! Mailjet protocol implementation for Manteau's core sender.
//! This crate performs no I/O. Supply a core Http implementation to Sender.
use manteau_core::{
  Address, EmailAddress, HttpProvider, MessageId, PreparedMessage, Receipt,
  StatusPolicy,
  http::{Credentials, HttpConfig, HttpError, HttpResponse},
};
use serde::{Deserialize, Serialize};
mod error;
pub use error::{MailjetError, MailjetErrorKind};

/// Configured Mailjet protocol. Credentials and endpoint are fixed at
/// construction.
#[derive(Debug)]
pub struct Mailjet {
  config:      HttpConfig,
  credentials: Credentials,
}
impl Mailjet {
  /// Bind Mailjet credentials to its standard Send API endpoint.
  pub fn new(key: &str, secret: &str) -> Result<Self, HttpError> {
    Self::with_config(
      key,
      secret,
      HttpConfig::new("https://api.mailjet.com/v3.1/send")?,
    )
  }

  /// Bind credentials to a trusted custom send endpoint and request bounds.
  pub fn with_config(
    key: &str,
    secret: &str,
    config: HttpConfig,
  ) -> Result<Self, HttpError> {
    Ok(Self {
      config,
      credentials: Credentials::basic(key, secret)?,
    })
  }
}
/// Provider acceptance for all recipients, not proof of delivery.
#[derive(Debug)]
pub struct MailjetReceipt {
  ids:        Vec<MessageId>,
  recipients: Vec<EmailAddress>,
}
impl MailjetReceipt {
  /// Recipients corresponding to `ids()`, ordered To, Cc, then Bcc.
  pub fn recipients(&self) -> &[EmailAddress] {
    &self.recipients
  }
}
impl Receipt for MailjetReceipt {
  fn ids(&self) -> &[MessageId] {
    &self.ids
  }
}
/// Provider request produced only through checked admission.
#[derive(Serialize)]
pub struct Payload<'a> {
  #[serde(rename = "Messages")]
  messages: [Mail<'a>; 1],
}
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Mail<'a> {
  from:    Mailbox<'a>,
  to:      Vec<Mailbox<'a>>,
  cc:      Vec<Mailbox<'a>>,
  bcc:     Vec<Mailbox<'a>>,
  subject: &'a str,
  #[serde(rename = "HTMLPart")]
  html:    &'a str,
  #[serde(rename = "TextPart")]
  text:    &'a str,
}
#[derive(Serialize)]
struct Mailbox<'a> {
  #[serde(rename = "Email")]
  email: &'a str,
  #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
  name:  Option<&'a str>,
}
impl<'a> From<&'a Address> for Mailbox<'a> {
  fn from(address: &'a Address) -> Self {
    Self {
      email: address.email().as_str(),
      name:  address.display_name(),
    }
  }
}
#[derive(Deserialize)]
struct Response {
  #[serde(rename = "Messages")]
  messages: Vec<Outcome>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Outcome {
  status: String,
  #[serde(default)]
  to:     Vec<Accepted>,
  #[serde(default)]
  cc:     Vec<Accepted>,
  #[serde(default)]
  bcc:    Vec<Accepted>,
}
#[derive(Deserialize)]
struct Accepted {
  #[serde(rename = "Email")]
  email: EmailAddress,
  #[serde(rename = "MessageID")]
  id:    ProviderId,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum ProviderId {
  Number(u64),
  Text(String),
}
impl ProviderId {
  fn into_id(self) -> Result<MessageId, MailjetError> {
    let id = match self {
      Self::Number(id) => id.to_string(),
      Self::Text(id) => id,
    };
    if id.is_empty() || id.chars().any(char::is_control) {
      return Err(MailjetErrorKind::Response.error());
    }
    Ok(MessageId::new(id))
  }
}
impl Response {
  fn into_receipt(
    self,
    message: &PreparedMessage,
  ) -> Result<MailjetReceipt, MailjetError> {
    let [outcome]: [Outcome; 1] = self
      .messages
      .try_into()
      .map_err(|_| MailjetErrorKind::Response.error())?;
    if outcome.status != "success" {
      // Mixed or unrecognized response evidence must not grant safe retry.
      if outcome.status == "error"
        && outcome.to.is_empty()
        && outcome.cc.is_empty()
        && outcome.bcc.is_empty()
      {
        return Err(MailjetErrorKind::Rejected.error());
      }
      return Err(MailjetErrorKind::Response.error());
    }
    let recipients = message.envelope().recipients();
    let mut receipt = MailjetReceipt {
      ids:        Vec::new(),
      recipients: Vec::new(),
    };
    for (actual, expected) in [
      (outcome.to, recipients.to_addresses()),
      (outcome.cc, recipients.cc_addresses()),
      (outcome.bcc, recipients.bcc_addresses()),
    ] {
      if actual.len() != expected.len() {
        return Err(MailjetErrorKind::Response.error());
      }
      for (accepted, address) in actual.into_iter().zip(expected) {
        if &accepted.email != address.email() {
          return Err(MailjetErrorKind::Response.error());
        }
        receipt.ids.push(accepted.id.into_id()?);
        receipt.recipients.push(accepted.email);
      }
    }
    Ok(receipt)
  }
}

impl HttpProvider for Mailjet {
  type Error = MailjetError;
  type Payload<'a> = Payload<'a>;
  type Receipt = MailjetReceipt;

  fn config(&self) -> &HttpConfig {
    &self.config
  }

  fn credentials(&self) -> &Credentials {
    &self.credentials
  }

  fn protocol_scope(&self) -> &'static str {
    concat!("mailjet/", env!("CARGO_PKG_VERSION"))
  }

  fn status_policy(&self) -> StatusPolicy {
    StatusPolicy {
      handled:      &[200],
      not_accepted: &[400, 401, 403, 413, 422, 429],
    }
  }

  fn request<'a>(
    &'a self,
    message: &'a PreparedMessage,
  ) -> Result<Payload<'a>, Self::Error> {
    let envelope = message.envelope();
    let recipients = envelope.recipients();
    if recipients.to_addresses().is_empty() {
      return Err(MailjetErrorKind::Input.error());
    }
    Ok(Payload {
      messages: [Mail {
        from:    envelope.from().into(),
        to:      recipients.to_addresses().iter().map(Into::into).collect(),
        cc:      recipients.cc_addresses().iter().map(Into::into).collect(),
        bcc:     recipients.bcc_addresses().iter().map(Into::into).collect(),
        subject: envelope.subject(),
        html:    message.body().html(),
        text:    message.body().text(),
      }],
    })
  }

  fn receipt(
    &self,
    response: HttpResponse,
    message: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error> {
    response
      .decode::<Response>()
      .map_err(MailjetError::decode)?
      .into_receipt(message)
  }
}
