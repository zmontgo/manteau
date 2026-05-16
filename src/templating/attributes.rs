//! Strongly-typed attribute primitives for MJML element fields.
//!
//! Every attribute value is a newtype with validating construction — invalid
//! values cannot exist past the boundary. Each type implements `Display` to
//! emit its MJML attribute form (`#ff0000`, `14px`, `50%`, `center`, ...).

use std::str::FromStr;

use validator::ValidateUrl;

// ---------- Color ----------

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
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl Color {
    /// Build a color from a 24-bit RGB value. `0xff0000` becomes `#ff0000`.
    /// Bits above the low 24 are ignored.
    pub fn hex(rgb: u32) -> Self {
        Self(format!("#{:06x}", rgb & 0x00ff_ffff))
    }

    /// Parse a hex (`#rgb`, `#rrggbb`, `#rgba`, `#rrggbbaa`) or named color.
    pub fn try_parse(s: &str) -> Result<Self, ColorError> {
        let s = s.trim();
        if is_valid_hex(s) || is_valid_name(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(ColorError { input: s.to_string() })
        }
    }

    /// Wrap a CSS named color (`red`, `cornflowerblue`, ...). Validation here
    /// is shape-only — alphabetic characters, no claim about whether the
    /// browser knows the name.
    pub fn named(name: &str) -> Result<Self, ColorError> {
        Self::try_parse(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_hex(s: &str) -> bool {
    let Some(rest) = s.strip_prefix('#') else { return false };
    matches!(rest.len(), 3 | 4 | 6 | 8) && rest.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_valid_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
}

impl FromStr for Color {
    type Err = ColorError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Color {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---------- Pixels ----------

/// Pixel dimension — rendered as `Npx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pixels(u32);

impl Pixels {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

impl From<u32> for Pixels {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for Pixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}px", self.0)
    }
}

// ---------- Percentage ----------

/// Percentage dimension (0-100) — rendered as `N%`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Percentage(u8);

#[derive(Debug, thiserror::Error)]
#[error("percentage must be 0..=100: got {value}")]
pub struct PercentageError {
    value: u32,
}

impl PercentageError {
    pub fn value(&self) -> u32 {
        self.value
    }
}

impl Percentage {
    pub fn new(value: u8) -> Result<Self, PercentageError> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(PercentageError { value: u32::from(value) })
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

// ---------- Url ----------

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
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl Url {
    pub fn try_parse(s: &str) -> Result<Self, UrlError> {
        let s = s.trim();
        if s.validate_url() {
            Ok(Self(s.to_string()))
        } else {
            Err(UrlError { input: s.to_string() })
        }
    }

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

// ---------- FontFamily ----------

/// Font stack — open wrapper around the caller's CSS font-family string
/// (e.g. `"Helvetica, Arial, sans-serif"`). No validation; intent only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontFamily(String);

impl FontFamily {
    pub fn new(stack: impl Into<String>) -> Self {
        Self(stack.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FontFamily {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for FontFamily {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

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
