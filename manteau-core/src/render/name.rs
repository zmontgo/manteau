//! Checked names for the custom MJML writer extension boundary.

/// A syntactically valid MJML element name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementName(&'static str);

/// A syntactically valid MJML attribute name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeName(&'static str);

/// An element or attribute name cannot be written as MJML markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid MJML markup name")]
pub struct InvalidMarkupName;

impl ElementName {
  pub(crate) const fn builtin(name: &'static str) -> Self { Self(name) }

  /// Admit a custom `mj-*` tag using lowercase ASCII letters, digits, and
  /// hyphens. This checks syntax; it does not register renderer support.
  pub fn custom(name: &'static str) -> Result<Self, InvalidMarkupName> {
    if !name.starts_with("mj-") || name.len() <= 3 || name.len() > 64 || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
      return Err(InvalidMarkupName);
    }

    Ok(Self(name))
  }

  /// The admitted tag name.
  pub fn as_str(self) -> &'static str { self.0 }
}

impl AttributeName {
  pub(crate) const fn builtin(name: &'static str) -> Self { Self(name) }

  /// Admit a custom attribute name using lowercase ASCII letters, digits,
  /// and hyphens. The first character must be a letter.
  pub fn custom(name: &'static str) -> Result<Self, InvalidMarkupName> {
    if name.len() > 64
      || !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
      || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
      return Err(InvalidMarkupName);
    }

    Ok(Self(name))
  }

  /// The admitted attribute name.
  pub fn as_str(self) -> &'static str { self.0 }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn custom_names_cannot_escape_markup_syntax() {
    assert!(ElementName::custom("mj-divider").is_ok());
    assert!(ElementName::custom("mj-divider onload=alert(1)").is_err());
    assert!(ElementName::custom("script").is_err());
    assert!(AttributeName::custom("data-role").is_ok());
    assert!(AttributeName::custom("role=\"admin\"").is_err());
    assert!(AttributeName::custom("9role").is_err());
  }
}
