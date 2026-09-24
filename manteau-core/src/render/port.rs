//! Values exchanged with a rendering implementation.

/// A complete MJML document emitted by Manteau's typed template model.
/// Only core can construct this value; renderers can inspect it but cannot
/// mistake arbitrary caller text for a prepared document.
pub struct MjmlDocument(String);

impl MjmlDocument {
  pub(crate) fn new(mjml: String) -> Self {
    Self(mjml)
  }

  /// Generated MJML, for the rendering implementation to parse.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Debug for MjmlDocument {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("MjmlDocument([redacted])")
  }
}

/// Nonempty renderer-produced HTML without forbidden control characters.
/// This is an email body contract, not a claim that the markup is sanitized
/// or safe to interpolate into another HTML document.
pub struct HtmlBody(String);

/// A renderer returned empty HTML or forbidden control characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidHtmlBody {
  /// No HTML was produced.
  #[error("rendered HTML is empty")]
  Empty,
  /// HTML contains a control character other than tab or a line break.
  #[error("rendered HTML contains a forbidden control character")]
  Control,
}

impl HtmlBody {
  /// Check the minimal HTML body contract at the adapter boundary.
  pub fn new(html: impl Into<String>) -> Result<Self, InvalidHtmlBody> {
    let html = html.into();
    if html.is_empty() {
      return Err(InvalidHtmlBody::Empty);
    }
    if html.chars().any(|ch| ch.is_control() && !matches!(ch, '\t' | '\n' | '\r')) {
      return Err(InvalidHtmlBody::Control);
    }

    Ok(Self(html))
  }

  /// Exact rendered HTML. This is trusted email markup, not sanitized markup.
  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub(crate) fn into_string(self) -> String {
    self.0
  }
}

impl std::fmt::Debug for HtmlBody {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("HtmlBody([redacted])")
  }
}

/// Plaintext without forbidden control characters. Empty text is allowed when
/// the paired HTML body carries the message.
#[derive(Clone)]
pub struct PlaintextBody(String);

/// Plaintext contains a forbidden control character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("plaintext contains a forbidden control character")]
pub struct InvalidPlaintextBody;

impl PlaintextBody {
  /// Check plaintext from a renderer or an author-supplied alternative.
  pub fn new(text: impl Into<String>) -> Result<Self, InvalidPlaintextBody> {
    let text = text.into();
    if text.chars().any(|ch| ch.is_control() && !matches!(ch, '\t' | '\n' | '\r')) {
      return Err(InvalidPlaintextBody);
    }

    Ok(Self(text))
  }

  /// Exact plaintext alternative.
  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub(crate) fn into_string(self) -> String {
    self.0
  }
}

impl std::fmt::Debug for PlaintextBody {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("PlaintextBody([redacted])")
  }
}

/// Converts core-generated MJML to HTML and, when needed, HTML to plaintext.
/// The two operations let core skip plaintext conversion when the caller
/// supplied an alternative. Implementations must not resolve remote resources.
pub trait Renderer: Send + Sync {
  /// Concrete failure with a preserved source chain.
  type Error: std::error::Error + Send + Sync + 'static;

  /// Render a complete core-generated MJML document to checked HTML.
  fn html(&self, mjml: &MjmlDocument) -> Result<HtmlBody, Self::Error>;

  /// Generate checked plaintext from an already checked HTML body.
  fn plaintext(&self, html: &HtmlBody) -> Result<PlaintextBody, Self::Error>;
}
