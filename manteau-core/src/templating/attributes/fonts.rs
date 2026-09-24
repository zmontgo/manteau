/// A comma-separated font stack without CSS declaration syntax. Family names
/// may contain letters, digits, spaces, hyphens, and underscores.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontFamily(String);

impl FontFamily {
  /// Validate a conservative font stack before it reaches inline CSS.
  pub fn new(stack: &str) -> Result<Self, FontFamilyError> {
    if stack.split(',').any(|name| {
      let name = name.trim();
      name.is_empty()
        || !name
          .chars()
          .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_'))
    }) {
      return Err(FontFamilyError);
    }

    Ok(Self(
      stack
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(", "),
    ))
  }

  /// Canonical font stack after validation.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// The font stack contains unsupported CSS or an empty family name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid font family stack")]
pub struct FontFamilyError;

impl std::fmt::Display for FontFamily {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for FontFamily {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Font weight, 100-900. Renders as the numeric value (`"700"`) rather than
/// the CSS keyword (`"bold"`) — every email client handles the numeric form,
/// while the keyword subset varies by client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontWeight {
  /// 100
  Thin,
  /// 200
  ExtraLight,
  /// 300
  Light,
  /// 400
  Normal,
  /// 500
  Medium,
  /// 600
  SemiBold,
  /// 700
  Bold,
  /// 800
  ExtraBold,
  /// 900
  Black,
}

impl FontWeight {
  /// The CSS numeric value (100 through 900).
  pub fn weight(self) -> u16 {
    match self {
      Self::Thin => 100,
      Self::ExtraLight => 200,
      Self::Light => 300,
      Self::Normal => 400,
      Self::Medium => 500,
      Self::SemiBold => 600,
      Self::Bold => 700,
      Self::ExtraBold => 800,
      Self::Black => 900,
    }
  }
}

impl std::fmt::Display for FontWeight {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.weight())
  }
}

/// CSS `text-transform` — controls case of rendered text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextTransform {
  /// Leave text case unchanged.
  None,
  /// Render text in uppercase.
  Uppercase,
  /// Render text in lowercase.
  Lowercase,
  /// Capitalize words in rendered text.
  Capitalize,
}

impl std::fmt::Display for TextTransform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Self::None => "none",
      Self::Uppercase => "uppercase",
      Self::Lowercase => "lowercase",
      Self::Capitalize => "capitalize",
    })
  }
}
