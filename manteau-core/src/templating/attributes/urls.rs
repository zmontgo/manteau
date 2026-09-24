use std::str::FromStr;

/// An absolute email link using HTTPS, HTTP, mailto, or tel.
/// Script, data, file, and credential-bearing URLs are rejected.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Url(String);

#[derive(thiserror::Error)]
#[error("invalid email URL")]
/// A rejected email URL; its input may contain private data.
pub struct UrlError {
  input: String,
}

impl UrlError {
  /// Return the rejected input; it may contain private data and must not be
  /// logged.
  pub fn input(&self) -> &str {
    &self.input
  }
}

impl std::fmt::Debug for UrlError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

impl std::fmt::Debug for Url {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Url([redacted])")
  }
}

impl Url {
  /// Parse an absolute URL with a supported email-link scheme.
  ///
  /// ```
  /// # use manteau_core::templating::attributes::urls::Url;
  /// assert!(Url::try_parse("https://example.com").is_ok());
  /// assert!(Url::try_parse("mailto:hello@example.com").is_ok());
  /// assert!(Url::try_parse("not a url").is_err());
  /// ```
  pub fn try_parse(s: &str) -> Result<Self, UrlError> {
    let s = s.trim();
    let invalid = || UrlError {
      input: s.to_string(),
    };
    if s.is_empty() || s.chars().any(char::is_control) {
      return Err(invalid());
    }

    let parsed = url::Url::parse(s).map_err(|_| invalid())?;
    match parsed.scheme() {
      "http" | "https"
        if parsed.host().is_some()
          && parsed.username().is_empty()
          && parsed.password().is_none() => {}
      "mailto" | "tel" if !parsed.path().is_empty() => {}
      _ => return Err(invalid()),
    }

    Ok(Self(parsed.into()))
  }

  /// Borrow the validated value as text.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for Url {
  type Err = UrlError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_parse(s)
  }
}

impl TryFrom<&str> for Url {
  type Error = UrlError;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    s.parse()
  }
}

impl TryFrom<String> for Url {
  type Error = UrlError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    s.parse()
  }
}

impl std::fmt::Display for Url {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for Url {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// A network image source. Email links may use `mailto:` or `tel:`, but an
/// image source must use HTTP or HTTPS.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageUrl(Url);

impl ImageUrl {
  /// Parse and admit an HTTP(S) image source.
  pub fn try_parse(value: &str) -> Result<Self, UrlError> {
    Self::try_from(Url::try_parse(value)?)
  }

  /// Canonical absolute image URL.
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl TryFrom<Url> for ImageUrl {
  type Error = UrlError;

  fn try_from(value: Url) -> Result<Self, Self::Error> {
    if value.0.starts_with("https://") || value.0.starts_with("http://") {
      Ok(Self(value))
    } else {
      Err(UrlError { input: value.0 })
    }
  }
}

impl std::fmt::Display for ImageUrl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_str().fmt(f)
  }
}

// ---------- Alignment ----------

/// Horizontal alignment — closed set, mapped to MJML's `align` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
  /// Align content to the left edge.
  Left,
  /// Center content horizontally.
  Center,
  /// Align content to the right edge.
  Right,
  /// Spread text across the available line width.
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
  /// Align content to the left edge.
  Left,
  /// Center content horizontally.
  Center,
  /// Align content to the right edge.
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
