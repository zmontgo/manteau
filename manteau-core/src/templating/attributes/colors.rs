use std::str::FromStr;

/// A CSS color value — hex (`#rgb`, `#rrggbb`, `#rgba`, `#rrggbbaa`) or a
/// named color.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Color(String);

#[derive(Debug, thiserror::Error)]
#[error("invalid color: {input}")]
pub struct ColorError {
  input: String,
}

impl ColorError {
  pub fn input(&self) -> &str { &self.input }
}

impl Color {
  /// Build a color from a 24-bit RGB value. Reject bits outside that range.
  ///
  /// ```
  /// # use manteau_core::templating::attributes::colors::Color;
  /// assert_eq!(Color::hex(0xff0000).unwrap().to_string(), "#ff0000");
  /// ```
  pub fn hex(rgb: u32) -> Result<Self, ColorError> {
    if rgb > 0x00ff_ffff {
      return Err(ColorError { input: format!("0x{rgb:x}") });
    }
    Ok(Self(format!("#{rgb:06x}")))
  }

  /// Parse a hex (`#rgb`, `#rrggbb`, `#rgba`, `#rrggbbaa`) or named color.
  ///
  /// ```
  /// # use manteau_core::templating::attributes::colors::Color;
  /// assert!(Color::try_parse("#ff0000").is_ok());
  /// assert!(Color::try_parse("red").is_ok());
  /// assert!(Color::try_parse("not a color").is_err());
  /// ```
  pub fn try_parse(s: &str) -> Result<Self, ColorError> {
    let s = s.trim();
    if Self::is_valid_hex(s) || Self::is_valid_name(s) {
      Ok(Self(s.to_string()))
    } else {
      Err(ColorError {
        input: s.to_string(),
      })
    }
  }

  /// Accept a CSS named color only when the CSS color parser recognizes it.
  pub fn named(name: &str) -> Result<Self, ColorError> { Self::try_parse(name) }

  pub fn as_str(&self) -> &str { &self.0 }

  fn is_valid_hex(s: &str) -> bool {
    let Some(rest) = s.strip_prefix('#') else {
      return false;
    };
    matches!(rest.len(), 3 | 4 | 6 | 8)
      && rest.chars().all(|c| c.is_ascii_hexdigit())
  }

  fn is_valid_name(s: &str) -> bool {
    !s.is_empty()
      && s.chars().all(|c| c.is_ascii_alphabetic())
      && csscolorparser::parse(s).is_ok()
  }
}

impl FromStr for Color {
  type Err = ColorError;

  fn from_str(s: &str) -> Result<Self, Self::Err> { Self::try_parse(s) }
}

impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for Color {
  fn as_ref(&self) -> &str { &self.0 }
}
