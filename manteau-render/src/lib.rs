//! MJML and plaintext rendering for Manteau's core rendering port.
//! The parser uses its no-op include loader, so rendering does not fetch
//! remote content or resolve local files.

use manteau_core::Renderer;

/// Configured MJML renderer. The default plaintext line width is 80 columns.
#[derive(Debug, Clone, Copy)]
pub struct MrmlRenderer {
  text_width: usize,
}

impl MrmlRenderer {
  /// Use the default 80-column plaintext alternative.
  pub fn new() -> Self {
    Self { text_width: 80 }
  }

  /// Set plaintext width. `html2text` requires at least three columns.
  pub fn text_width(mut self, width: usize) -> Result<Self, InvalidTextWidth> {
    if width < 3 {
      return Err(InvalidTextWidth);
    }

    self.text_width = width;
    Ok(self)
  }
}

impl Default for MrmlRenderer {
  fn default() -> Self {
    Self::new()
  }
}

/// The requested plaintext width is below the renderer's minimum.
#[derive(Debug, thiserror::Error)]
#[error("plaintext width must be at least three columns")]
pub struct InvalidTextWidth;

/// Concrete parser, HTML renderer, or plaintext conversion failure.
/// Normal formatting omits template content; inspect the source explicitly
/// when diagnosing trusted input.
#[derive(thiserror::Error)]
pub enum MrmlError {
  /// The generated MJML could not be parsed.
  #[error("could not parse generated MJML")]
  Parse(#[source] mrml::prelude::parser::Error),

  /// The parsed MJML could not be rendered to HTML.
  #[error("could not render parsed MJML")]
  Html(#[source] mrml::prelude::render::Error),

  /// HTML could not be converted to plaintext.
  #[error("could not convert HTML to plaintext")]
  Plaintext(#[source] html2text::Error),
}

impl std::fmt::Debug for MrmlError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

impl Renderer for MrmlRenderer {
  type Error = MrmlError;

  fn html(&self, mjml: &str) -> Result<String, Self::Error> {
    let options = mrml::prelude::parser::ParserOptions::default();
    let parsed =
      mrml::parse_with_options(mjml, &options).map_err(MrmlError::Parse)?;
    parsed
      .element
      .render(&Default::default())
      .map_err(MrmlError::Html)
  }

  fn plaintext(&self, html: &str) -> Result<String, Self::Error> {
    html2text::from_read(html.as_bytes(), self.text_width)
      .map_err(MrmlError::Plaintext)
  }
}
