/// Font stack — open wrapper around the caller's CSS font-family string
/// (e.g. `"Helvetica, Arial, sans-serif"`). No validation; intent only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontFamily(String);

impl FontFamily {
  pub fn new(stack: impl Into<String>) -> Self { Self(stack.into()) }

  pub fn as_str(&self) -> &str { &self.0 }
}

impl From<&str> for FontFamily {
  fn from(s: &str) -> Self { Self::new(s) }
}

impl From<String> for FontFamily {
  fn from(s: String) -> Self { Self::new(s) }
}

impl std::fmt::Display for FontFamily {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for FontFamily {
  fn as_ref(&self) -> &str { &self.0 }
}

/// Font weight, 100-900
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
