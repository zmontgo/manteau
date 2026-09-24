//! Provider-enforced deduplication of immutable, rendered email submissions.
use std::time::Duration;

use async_trait::async_trait;

use crate::{Address, Message, RenderError, Transport};

/// A caller-owned submission identity, retained unchanged across retries.
/// Never use a new key to recover an ambiguous submission.
#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
pub struct IdempotencyKey(String);

/// The key must contain 1–256 visible ASCII characters.
#[derive(Debug, thiserror::Error)]
#[error("idempotency key must contain 1–256 visible ASCII characters")]
pub struct InvalidIdempotencyKey;

impl TryFrom<String> for IdempotencyKey {
  type Error = InvalidIdempotencyKey;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.is_empty()
      || value.len() > 256
      || !value.bytes().all(|b| b.is_ascii_graphic())
    {
      return Err(InvalidIdempotencyKey);
    }
    Ok(Self(value))
  }
}
impl IdempotencyKey {
  /// The exact value sent to the provider.
  pub fn as_str(&self) -> &str { &self.0 }
}

/// Rendered content to persist with its key before the first attempt.
/// Reusing this value avoids template changes altering a retried request.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Envelope")]
pub struct PreparedMessage(Envelope);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
  from:    String,
  to:      Vec<String>,
  cc:      Vec<String>,
  bcc:     Vec<String>,
  subject: String,
  html:    String,
  text:    String,
}

/// Invalid envelope headers, recipients, or empty content.
#[derive(Debug, thiserror::Error)]
#[error("invalid prepared email")]
pub struct InvalidMessage;

impl TryFrom<Envelope> for PreparedMessage {
  type Error = InvalidMessage;

  fn try_from(value: Envelope) -> Result<Self, Self::Error> {
    if value.from.is_empty()
      || value.to.is_empty()
      || value.subject.is_empty()
      || (value.html.is_empty() && value.text.is_empty())
      || std::iter::once(&value.from)
        .chain(value.to.iter())
        .chain(value.cc.iter())
        .chain(value.bcc.iter())
        .chain(std::iter::once(&value.subject))
        .any(|s| s.is_empty() || s.chars().any(char::is_control))
    {
      return Err(InvalidMessage);
    }
    // Stored envelopes may include display names. Each mailbox was typed at
    // construction; deserialization also validates the final mailbox syntax.
    for address in std::iter::once(&value.from)
      .chain(value.to.iter())
      .chain(value.cc.iter())
      .chain(value.bcc.iter())
    {
      let mailbox = if let Some((_, tail)) = address.rsplit_once(" <") {
        tail.strip_suffix('>').ok_or(InvalidMessage)?
      } else {
        address.as_str()
      };
      mailbox
        .parse::<crate::EmailAddress>()
        .map_err(|_| InvalidMessage)?;
    }
    Ok(Self(value))
  }
}
impl PreparedMessage {
  /// Freeze already-rendered content and a typed envelope without rerendering.
  pub fn new(
    from: Address,
    to: Vec<Address>,
    subject: String,
    html: String,
    text: String,
  ) -> Result<Self, InvalidMessage> {
    Self::try_from(Envelope {
      from: from.mailbox(),
      to: to.iter().map(Address::mailbox).collect(),
      cc: Vec::new(),
      bcc: Vec::new(),
      subject,
      html,
      text,
    })
  }

  /// Freeze all recipients and render a template exactly once.
  pub fn render(message: &Message) -> Result<Self, PreparationError> {
    let rendered = message.render()?;
    Ok(Self::try_from(Envelope {
      from:    message.from.mailbox(),
      to:      message.to.iter().map(Address::mailbox).collect(),
      cc:      message.cc.iter().map(Address::mailbox).collect(),
      bcc:     message.bcc.iter().map(Address::mailbox).collect(),
      subject: message.subject.clone(),
      html:    rendered.html,
      text:    rendered.text,
    })?)
  }
}
impl Address {
  fn mailbox(&self) -> String {
    match &self.name {
      Some(name) => format!(
        "\"{}\" <{}>",
        name.replace('\\', "\\\\").replace('"', "\\\""),
        self.email
      ),
      None => self.email.to_string(),
    }
  }
}
/// A template could not be rendered or its envelope is invalid.
#[derive(Debug, thiserror::Error)]
pub enum PreparationError {
  /// Template rendering failed before submission.
  #[error(transparent)]
  Render(#[from] RenderError),
  /// Envelope validation failed before submission.
  #[error(transparent)]
  Envelope(#[from] InvalidMessage),
}

/// A transport that deduplicates submissions at the provider, not in memory.
///
/// Implementors must replay the original acceptance for an identical key and
/// payload and reject key reuse with different content. The guarantee applies
/// within `retention()` and the same configured account and region. Callers
/// must persist the key, prepared message and first-attempt time together and
/// stop retries before that window expires. Acceptance does not prove delivery.
#[async_trait]
pub trait IdempotentTransport: Transport {
  /// Minimum provider retention from the first submission; never sliding.
  fn retention(&self) -> Duration;
  /// Stable, non-secret identity of this provider account and routing target.
  /// A changed scope must prevent replay against a different deduplication
  /// store.
  fn scope(&self) -> &str;
  /// Maximum duration of one request, reserved before the retention deadline.
  fn request_timeout(&self) -> Duration;
  /// Submit once or recover the original receipt using the same key and body.
  async fn send_idempotent(
    &self,
    key: &IdempotencyKey,
    message: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error>;
}

impl serde::Serialize for IdempotencyKey {
  fn serialize<S: serde::Serializer>(
    &self,
    serializer: S,
  ) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.0)
  }
}
