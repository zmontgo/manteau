//! Rendering pipeline: [`Template`] -> MJML -> HTML -> plaintext.

use crate::render::error::{RenderError, RenderErrorKind};
use crate::render::writer::MjmlWriter;
use crate::templating::{Element, Template};

/// Default plaintext wrap width (columns). Email plaintext convention is
/// 78-80 columns; we pick 80 as the standard plaintext width.
const DEFAULT_WRAP_WIDTH: usize = 80;

/// The product of rendering a [`Template`]: the HTML body and a plaintext
/// alternative for the `multipart/alternative` payload most providers expect.
#[derive(Debug, Clone)]
pub struct Rendered {
    pub html: String,
    pub text: String,
}

/// Render a [`Template`] to HTML via MJML.
///
/// Walks the element tree to produce an MJML string, then runs it through
/// `mrml` to produce the final HTML.
pub fn render_html(template: &Template) -> Result<String, RenderError> {
    let mut writer = MjmlWriter::new();
    template.write_mjml(&mut writer);
    let mjml = writer.into_string();

    let parsed = mrml::parse(&mjml)
        .map_err(|e| RenderErrorKind::Parse.err(format!("{e:?}")))?;

    let opts = mrml::prelude::render::RenderOptions::default();
    parsed
        .element
        .render(&opts)
        .map_err(|e| RenderErrorKind::Render.err(format!("{e:?}")))
}

/// Render HTML to plaintext via `html2text` at the default wrap width.
pub fn render_plaintext(html: &str) -> Result<String, RenderError> {
    html2text::from_read(html.as_bytes(), DEFAULT_WRAP_WIDTH)
        .map_err(|e| RenderErrorKind::Plaintext.err(format!("{e:?}")))
}
