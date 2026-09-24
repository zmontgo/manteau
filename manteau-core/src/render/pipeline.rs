//! Shared preparation of MJML and validated rendered bodies.

use crate::{
  render::{HtmlBody, MjmlDocument, MjmlWriter, PlaintextBody, RenderError, Renderer},
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
  pub(crate) fn from_renderer(html: HtmlBody, text: PlaintextBody) -> Self {
    Self { html: html.into_string(), text: text.into_string() }
  }

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
  pub fn html(&self) -> &str {
    &self.html
  }

  /// Exact plaintext alternative; may be empty for HTML-only mail.
  pub fn text(&self) -> &str {
    &self.text
  }
}

impl std::fmt::Debug for Rendered {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Rendered([redacted])")
  }
}

impl Template {
  /// Render the complete document using an explicitly chosen rendering
  /// capability. The renderer controls HTML conversion and plaintext width.
  pub fn render(
    &self,
    renderer: &impl Renderer,
  ) -> Result<Rendered, RenderError> {
    self.render_with_text(renderer, None)
  }

  /// Render HTML and use the supplied plaintext alternative when present.
  /// This skips plaintext conversion while retaining body validation.
  pub fn render_with_text(
    &self,
    renderer: &impl Renderer,
    explicit_text: Option<&str>,
  ) -> Result<Rendered, RenderError> {
    let mut writer = MjmlWriter::new();
    self.write_mjml(&mut writer);

    let document = MjmlDocument::new(writer.into_string());
    let html = renderer.html(&document).map_err(RenderError::html)?;
    let text = match explicit_text {
      Some(text) => PlaintextBody::new(text).map_err(RenderError::plaintext)?,
      None => renderer.plaintext(&html).map_err(RenderError::plaintext)?,
    };

    Ok(Rendered::from_renderer(html, text))
  }
}
