//! Shared data types that cross multiple module boundaries.
//!
//! Lives at the crate root so neither the `templating` model layer nor the
//! `transports` adapters has to reach across each other to share primitives.

use std::str::FromStr;

use typed_builder::TypedBuilder;
use validator::ValidateEmail;

/// A syntactically valid email address, validated at construction.
///
/// Validation follows the HTML5 spec (what `<input type="email">` accepts) via
/// the `validator` crate — chosen over strict RFC 5322 because email providers
/// in practice accept the HTML5 set, not the RFC superset. Quoted local parts
/// and IP-literal hosts are rejected here even though they are RFC-valid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

#[derive(Debug, thiserror::Error)]
#[error("invalid email address: {input}")]
pub struct EmailAddressError {
    input: String,
}

impl EmailAddressError {
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl EmailAddress {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for EmailAddress {
    type Err = EmailAddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.validate_email() {
            Ok(Self(s.to_string()))
        } else {
            Err(EmailAddressError { input: s.to_string() })
        }
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

/// A `From`/`To`/`Cc`/`Bcc` participant — a validated email plus an optional
/// display name.
///
/// `email` is taken as a constructed `EmailAddress`. Validation lives on the
/// newtype; the builder just accepts an already-valid value:
///
/// ```
/// # use manteau::Address;
/// let a = Address::builder()
///     .email("hello@example.com".parse()?)
///     .name("Hello")
///     .build();
/// # Ok::<(), manteau::EmailAddressError>(())
/// ```
#[derive(Debug, Clone, TypedBuilder)]
pub struct Address {
    pub email: EmailAddress,
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,
}

/// Identifier returned by a transport for a successfully accepted message.
///
/// Provider-specific in meaning — Mailjet returns a numeric string, stdout
/// returns a counter, mock returns a fixed marker. The type carries intent
/// (this string is a message id) without making claims about its format.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(String);

impl MessageId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

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
