//! Rendering owned by templates and rendered bodies.
use crate::{
  render::{MjmlWriter, RenderError, RenderErrorKind},
  templating::{Element, Template},
};

/// HTML and plaintext alternatives, with at least one nonempty body.
/// Construction and deserialization establish the same invariant. Debug redacts
/// content; getters and serialization intentionally expose the email bodies.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Bodies")]
pub struct Rendered {
  html: String,
  text: String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Bodies {
  html: String,
  text: String,
}
/// Both email body alternatives were empty.
#[derive(Debug, thiserror::Error)]
#[error("at least one nonempty email body is required")]
pub struct EmptyBody;
impl TryFrom<Bodies> for Rendered {
  type Error = EmptyBody;

  fn try_from(value: Bodies) -> Result<Self, EmptyBody> {
    Self::new(value.html, value.text)
  }
}
impl Rendered {
  /// Accept externally rendered content. HTML is trusted markup, not sanitized.
  /// Use typed templates when interpolating untrusted text.
  pub fn new(
    html: impl Into<String>,
    text: impl Into<String>,
  ) -> Result<Self, EmptyBody> {
    let html = html.into();
    let text = text.into();
    if html.is_empty() && text.is_empty() {
      return Err(EmptyBody);
    }
    Ok(Self { html, text })
  }

  /// Exact HTML alternative; may be empty for plaintext-only mail.
  pub fn html(&self) -> &str { &self.html }

  /// Exact plaintext alternative; may be empty for HTML-only mail.
  pub fn text(&self) -> &str { &self.text }
}
impl std::fmt::Debug for Rendered {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Rendered([redacted])")
  }
}
impl Template {
  /// Render the complete document to HTML and a plaintext alternative wrapped
  /// at 80 columns. Rendering performs no network requests.
  pub fn render(&self) -> Result<Rendered, RenderError> {
    self.render_with_text(None)
  }

  #[tracing::instrument(skip_all)]
  pub(crate) fn render_with_text(
    &self,
    text: Option<&str>,
  ) -> Result<Rendered, RenderError> {
    let mut writer = MjmlWriter::new();
    self.write_mjml(&mut writer);
    let parsed = mrml::parse(writer.as_str())
      .map_err(|e| RenderErrorKind::Parse.err(e))?;
    let html = parsed
      .element
      .render(&Default::default())
      .map_err(|e| RenderErrorKind::Render.err(e))?;
    let text = match text {
      Some(text) => text.to_owned(),
      None => html2text::from_read(html.as_bytes(), 80)
        .map_err(|e| RenderErrorKind::Plaintext.err(e))?,
    };
    Ok(Rendered { html, text })
  }
}
