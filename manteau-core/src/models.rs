//! Shared data types that cross multiple module boundaries.
//!
//! Lives at the crate root so neither the `templating` model layer nor the
//! `transports` adapters has to reach across each other to share primitives.

use std::str::FromStr;

use validator::ValidateEmail;

/// A syntactically valid email address, validated at construction.
///
/// Validation follows the HTML5 spec (what `<input type="email">` accepts).
///
/// Input is trimmed of surrounding whitespace before validation. The domain
/// component is lower-cased on construction so that addresses differing only
/// in domain case (`User@Example.com` vs `User@example.com`) compare and hash
/// equal. The local part is preserved as-is (it is technically case-sensitive
/// per RFC, even though most providers treat it as case-insensitive).
#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct EmailAddress(String);

/// Failure produced by [`EmailAddress`] parsing.
///
/// The failing input is preserved for developer-side debugging via
/// [`EmailAddressError::input`]. Email addresses are PII — do not surface
/// this error's `Display` text directly to end users or to public logs.
#[derive(thiserror::Error)]
#[error("invalid email address")]
pub struct EmailAddressError {
  input: String,
}

impl EmailAddressError {
  /// Return the rejected input; it may contain private data and must not be
  /// logged.
  pub fn input(&self) -> &str {
    &self.input
  }
}

impl EmailAddress {
  /// Borrow the validated value as text.
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// The local part — everything before `@`.
  ///
  /// ```
  /// # use manteau_core::EmailAddress;
  /// let e: EmailAddress = "User@Example.COM".parse().unwrap();
  /// assert_eq!(e.local_part(), "User");
  /// assert_eq!(e.domain(), "example.com");
  /// ```
  pub fn local_part(&self) -> &str {
    // `validate_email` enforces exactly one `@`, so split_once cannot return
    // None for a successfully constructed EmailAddress.
    self.0.split_once('@').expect("validated email").0
  }

  /// The domain — everything after `@`, lower-cased on construction.
  pub fn domain(&self) -> &str {
    self.0.split_once('@').expect("validated email").1
  }
}

impl FromStr for EmailAddress {
  type Err = EmailAddressError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let s = s.trim();
    if !s.validate_email() {
      return Err(EmailAddressError {
        input: s.to_string(),
      });
    }
    // Domain part lower-cased; local part preserved.
    let (local, domain) = s.split_once('@').expect("validated email");
    let normalized = format!("{local}@{}", domain.to_ascii_lowercase());
    Ok(Self(normalized))
  }
}

impl TryFrom<&str> for EmailAddress {
  type Error = EmailAddressError;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    s.parse()
  }
}

impl TryFrom<String> for EmailAddress {
  type Error = EmailAddressError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    s.parse()
  }
}

impl std::fmt::Display for EmailAddress {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for EmailAddress {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// A validated single-line email header value. Empty values are permitted.
/// Controls (including CR, LF, and tabs) are rejected before serialization.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct HeaderText(String);

/// A header contains a control character. The input is deliberately omitted.
#[derive(Debug, thiserror::Error)]
#[error("email headers must not contain control characters")]
pub struct InvalidHeader;
impl HeaderText {
  /// Validate a subject or display name without modifying its text.
  pub fn new(value: impl Into<String>) -> Result<Self, InvalidHeader> {
    let value = value.into();
    if value.chars().any(char::is_control) {
      return Err(InvalidHeader);
    }
    Ok(Self(value))
  }

  /// Explicit access to potentially private header content.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for HeaderText {
  type Error = InvalidHeader;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

/// A mailbox with an optional validated display name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
  email: EmailAddress,
  name:  Option<HeaderText>,
}

impl Address {
  /// Construct an unnamed mailbox from an already validated address.
  pub fn new(email: EmailAddress) -> Self {
    Self { email, name: None }
  }

  /// Attach a single-line display name.
  pub fn name(mut self, name: HeaderText) -> Self {
    self.name = Some(name);
    self
  }

  /// The validated mailbox address.
  pub fn email(&self) -> &EmailAddress {
    &self.email
  }

  /// Optional display name; accessing it may expose personal information.
  pub fn display_name(&self) -> Option<&str> {
    self.name.as_ref().map(HeaderText::as_str)
  }

  /// Format a quoted mailbox for providers accepting RFC-style mailbox strings.
  pub fn mailbox(&self) -> String {
    match self.display_name() {
      Some(name) => format!(
        "\"{}\" <{}>",
        name.replace('\\', "\\\\").replace('"', "\\\""),
        self.email
      ),
      None => self.email.to_string(),
    }
  }
}

/// Identifier returned by a transport for a successfully accepted message.
///
/// Provider-specific in meaning — Mailjet returns a numeric string, stdout
/// returns a counter, mock returns a fixed marker. The type carries intent
/// (this string is a message id) without making claims about its format.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MessageId(String);

/// A provider identifier was empty or contained a control character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid provider message identifier")]
pub struct InvalidMessageId;

impl MessageId {
  /// Admit a provider-assigned identifier without assuming its format.
  pub fn new(s: impl Into<String>) -> Result<Self, InvalidMessageId> {
    let value = s.into();
    if value.is_empty() || value.chars().any(char::is_control) {
      return Err(InvalidMessageId);
    }
    Ok(Self(value))
  }

  /// Borrow the validated value as text.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Display for MessageId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for MessageId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Debug for MessageId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("MessageId([redacted])")
  }
}

impl std::fmt::Debug for EmailAddress {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("EmailAddress([redacted])")
  }
}

impl std::fmt::Debug for EmailAddressError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("EmailAddressError([redacted])")
  }
}

impl std::fmt::Debug for HeaderText {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("HeaderText([redacted])")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn email_accepts_simple() {
    assert!("hello@example.com".parse::<EmailAddress>().is_ok());
    assert!("a+tag@sub.example.co.uk".parse::<EmailAddress>().is_ok());
  }

  #[test]
  fn email_rejects_invalid() {
    assert!("not an email".parse::<EmailAddress>().is_err());
    assert!("missing@".parse::<EmailAddress>().is_err());
    assert!("@missing.local".parse::<EmailAddress>().is_err());
    assert!("".parse::<EmailAddress>().is_err());
  }

  #[test]
  fn email_error_carries_input() {
    let err = "bad".parse::<EmailAddress>().unwrap_err();
    assert_eq!(err.input(), "bad");
  }

  #[test]
  fn address_constructor_and_setter() {
    let a = Address::new("test@example.com".parse().unwrap())
      .name(HeaderText::new("Test").unwrap());
    assert_eq!(a.email.as_str(), "test@example.com");
    assert_eq!(a.display_name(), Some("Test"));
  }

  #[test]
  fn message_id_display() {
    let id = MessageId::new("abc123").unwrap();
    assert_eq!(id.to_string(), "abc123");
    assert_eq!(id.as_str(), "abc123");
  }

  #[test]
  fn email_trims_whitespace() {
    let e: EmailAddress = "  hello@example.com  ".parse().unwrap();
    assert_eq!(e.as_str(), "hello@example.com");
  }

  #[test]
  fn email_lowercases_domain() {
    let e: EmailAddress = "Hello@Example.COM".parse().unwrap();
    assert_eq!(e.as_str(), "Hello@example.com");
    // Local part preserved.
    assert_eq!(e.local_part(), "Hello");
    assert_eq!(e.domain(), "example.com");
  }

  #[test]
  fn email_case_differing_domains_are_equal() {
    let a: EmailAddress = "a@Example.com".parse().unwrap();
    let b: EmailAddress = "a@example.COM".parse().unwrap();
    assert_eq!(a, b);
  }
}
