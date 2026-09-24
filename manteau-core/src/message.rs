//! Validated envelopes and immutable transport input.
use crate::{Address, HeaderText, Rendered};

/// Nonempty recipient lists, preserving To, Cc, and Bcc roles.
/// No mutation can remove the last recipient. Provider-specific limits are
/// enforced by adapters, not by this provider-independent type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RecipientLists")]
pub struct Recipients {
  to:  Vec<Address>,
  cc:  Vec<Address>,
  bcc: Vec<Address>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipientLists {
  to:  Vec<Address>,
  cc:  Vec<Address>,
  bcc: Vec<Address>,
}

/// An envelope must contain at least one recipient across To, Cc, and Bcc.
#[derive(Debug, thiserror::Error)]
#[error("at least one recipient is required")]
pub struct EmptyRecipients;
impl TryFrom<RecipientLists> for Recipients {
  type Error = EmptyRecipients;

  fn try_from(lists: RecipientLists) -> Result<Self, Self::Error> {
    if lists.to.is_empty() && lists.cc.is_empty() && lists.bcc.is_empty() {
      return Err(EmptyRecipients);
    }
    Ok(Self {
      to:  lists.to,
      cc:  lists.cc,
      bcc: lists.bcc,
    })
  }
}

impl Recipients {
  /// Start with a visible primary recipient.
  pub fn to(address: Address) -> Self {
    Self {
      to:  vec![address],
      cc:  vec![],
      bcc: vec![],
    }
  }

  /// Start with a carbon-copy recipient and no To header.
  pub fn cc(address: Address) -> Self {
    Self {
      to:  vec![],
      cc:  vec![address],
      bcc: vec![],
    }
  }

  /// Start with a blind-copy recipient and no visible recipients.
  pub fn bcc(address: Address) -> Self {
    Self {
      to:  vec![],
      cc:  vec![],
      bcc: vec![address],
    }
  }

  /// Append a primary recipient.
  pub fn push_to(mut self, address: Address) -> Self {
    self.to.push(address);
    self
  }

  /// Append a carbon-copy recipient.
  pub fn push_cc(mut self, address: Address) -> Self {
    self.cc.push(address);
    self
  }

  /// Append a blind-copy recipient.
  pub fn push_bcc(mut self, address: Address) -> Self {
    self.bcc.push(address);
    self
  }

  /// Primary recipients, in insertion order.
  pub fn to_addresses(&self) -> &[Address] {
    &self.to
  }

  /// Carbon-copy recipients, in insertion order.
  pub fn cc_addresses(&self) -> &[Address] {
    &self.cc
  }

  /// Blind-copy recipients. Do not include these in visible email headers.
  pub fn bcc_addresses(&self) -> &[Address] {
    &self.bcc
  }

  /// Every recipient, ordered by To, then Cc, then Bcc.
  pub fn iter(&self) -> impl Iterator<Item = &Address> {
    self.to.iter().chain(&self.cc).chain(&self.bcc)
  }
}

/// Validated routing and header data independent of any provider or template.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
  from:       Address,
  recipients: Recipients,
  subject:    HeaderText,
}

impl Envelope {
  /// Bind a sender, nonempty recipient collection, and safe single-line
  /// subject.
  pub fn new(
    from: Address,
    recipients: Recipients,
    subject: HeaderText,
  ) -> Self {
    Self {
      from,
      recipients,
      subject,
    }
  }

  /// Sender mailbox; provider authorization is checked remotely.
  pub fn from(&self) -> &Address {
    &self.from
  }

  /// Recipients with their header roles preserved.
  pub fn recipients(&self) -> &Recipients {
    &self.recipients
  }

  /// Explicit access to the potentially private subject.
  pub fn subject(&self) -> &str {
    self.subject.as_str()
  }
}

/// Immutable, provider-independent transport input. Clone is cheap: the
/// envelope and rendered bodies are shared. Serialize only into protected
/// storage; this representation contains the complete private email, including
/// Bcc recipients.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PreparedMessage(std::sync::Arc<PreparedContent>);
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparedContent {
  envelope: Envelope,
  body:     Rendered,
}

impl PreparedMessage {
  /// Freeze an envelope and validated rendered bodies without rendering again.
  pub fn new(envelope: Envelope, body: Rendered) -> Self {
    Self(std::sync::Arc::new(PreparedContent { envelope, body }))
  }

  /// Routing and header values; no mutable access is exposed.
  pub fn envelope(&self) -> &Envelope {
    &self.0.envelope
  }

  /// Exact rendered content, unchanged across clones and serialization round
  /// trips.
  pub fn body(&self) -> &Rendered {
    &self.0.body
  }
}

impl std::fmt::Debug for PreparedMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("PreparedMessage([redacted])")
  }
}
