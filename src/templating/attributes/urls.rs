use std::str::FromStr;

use validator::ValidateUrl;

/// A URL validated at construction. Backed by the `validator` crate's
/// `ValidateUrl`, which itself delegates to the `url` crate's parser.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(String);

#[derive(Debug, thiserror::Error)]
#[error("invalid url: {input}")]
pub struct UrlError {
  input: String,
}

impl UrlError {
  pub fn input(&self) -> &str { &self.input }
}

impl Url {
  /// Parse and validate a URL string. Accepts any absolute URL the
  /// `validator` crate's HTML5-spec check considers well-formed —
  /// `https://`, `http://`, `mailto:`, `tel:` schemes all pass.
  ///
  /// ```
  /// # use manteau::templating::attributes::Url;
  /// assert!(Url::try_parse("https://example.com").is_ok());
  /// assert!(Url::try_parse("mailto:hello@example.com").is_ok());
  /// assert!(Url::try_parse("not a url").is_err());
  /// ```
  pub fn try_parse(s: &str) -> Result<Self, UrlError> {
    let s = s.trim();
    if s.validate_url() {
      Ok(Self(s.to_string()))
    } else {
      Err(UrlError {
        input: s.to_string(),
      })
    }
  }

  pub fn as_str(&self) -> &str { &self.0 }
}

impl FromStr for Url {
  type Err = UrlError;

  fn from_str(s: &str) -> Result<Self, Self::Err> { Self::try_parse(s) }
}

impl TryFrom<&str> for Url {
  type Error = UrlError;

  fn try_from(s: &str) -> Result<Self, Self::Error> { s.parse() }
}

impl TryFrom<String> for Url {
  type Error = UrlError;

  fn try_from(s: String) -> Result<Self, Self::Error> { s.parse() }
}

impl std::fmt::Display for Url {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for Url {
  fn as_ref(&self) -> &str { &self.0 }
}

// ---------- Alignment ----------

/// Horizontal alignment — closed set, mapped to MJML's `align` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
  Left,
  Center,
  Right,
  Justify,
}

impl std::fmt::Display for Alignment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Self::Left => "left",
      Self::Center => "center",
      Self::Right => "right",
      Self::Justify => "justify",
    })
  }
}

/// Identical to the `Alignment` enum, except lacking `Justify`, since this is
/// not a valid option for a button. Truly unfortunate that the creators of HTML
/// did not consider our crate's elegance in their design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonAlignment {
  Left,
  Center,
  Right,
}

impl std::fmt::Display for ButtonAlignment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Self::Left => "left",
      Self::Center => "center",
      Self::Right => "right",
    })
  }
}
