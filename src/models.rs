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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

/// Failure produced by [`EmailAddress`] parsing.
///
/// The failing input is preserved for developer-side debugging via
/// [`EmailAddressError::input`]. Email addresses are PII — do not surface
/// this error's `Display` text directly to end users or to public logs.
#[derive(Debug, thiserror::Error)]
#[error("invalid email address: {input}")]
pub struct EmailAddressError {
  input: String,
}

impl EmailAddressError {
  pub fn input(&self) -> &str { &self.input }
}

impl EmailAddress {
  pub fn as_str(&self) -> &str { &self.0 }

  /// The local part — everything before `@`.
  ///
  /// ```
  /// # use manteau::EmailAddress;
  /// let e: EmailAddress = "User@Example.COM".parse().unwrap();
  /// assert_eq!(e.local_part(), "User");
  /// assert_eq!(e.domain(), "example.com");
  /// ```
  pub fn local_part(&self) -> &str {
    // `validate_email` enforces exactly one `@`, so split_once cannot return
    // None for a successfully constructed EmailAddress.
    self.0.split_once('@').map(|(l, _)| l).unwrap_or(&self.0)
  }

  /// The domain — everything after `@`, lower-cased on construction.
  pub fn domain(&self) -> &str {
    self.0.split_once('@').map(|(_, d)| d).unwrap_or("")
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
    let normalized = match s.split_once('@') {
      Some((local, domain)) => format!("{}@{}", local, domain.to_lowercase()),
      None => s.to_string(), // unreachable: validate_email enforces '@'
    };
    Ok(Self(normalized))
  }
}

impl TryFrom<&str> for EmailAddress {
  type Error = EmailAddressError;

  fn try_from(s: &str) -> Result<Self, Self::Error> { s.parse() }
}

impl TryFrom<String> for EmailAddress {
  type Error = EmailAddressError;

  fn try_from(s: String) -> Result<Self, Self::Error> { s.parse() }
}

impl std::fmt::Display for EmailAddress {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for EmailAddress {
  fn as_ref(&self) -> &str { &self.0 }
}

/// A `From`/`To`/`Cc`/`Bcc` participant — a validated email plus an optional
/// display name.
///
/// ```
/// # use manteau::Address;
/// let a = Address::new("hello@example.com".parse()?).name("Hello");
/// # Ok::<(), manteau::EmailAddressError>(())
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Address {
  pub email: EmailAddress,
  pub name:  Option<String>,
}

impl Address {
  pub fn new(email: EmailAddress) -> Self { Self { email, name: None } }

  pub fn name(mut self, name: impl Into<String>) -> Self {
    self.name = Some(name.into());
    self
  }
}

/// Identifier returned by a transport for a successfully accepted message.
///
/// Provider-specific in meaning — Mailjet returns a numeric string, stdout
/// returns a counter, mock returns a fixed marker. The type carries intent
/// (this string is a message id) without making claims about its format.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(String);

impl MessageId {
  pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }

  pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for MessageId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for MessageId {
  fn as_ref(&self) -> &str { &self.0 }
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
    let a = Address::new("test@example.com".parse().unwrap()).name("Test");
    assert_eq!(a.email.as_str(), "test@example.com");
    assert_eq!(a.name.as_deref(), Some("Test"));
  }

  #[test]
  fn message_id_display() {
    let id = MessageId::new("abc123");
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
