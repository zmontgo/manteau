//! Rendering pipeline: [`Template`] -> MJML -> HTML -> plaintext.

use crate::{
  render::{
    error::{RenderError, RenderErrorKind},
    writer::MjmlWriter,
  },
  templating::{Element, Template},
};

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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::templating::{Body, Column, Section, Template, Text};

  fn minimal_template() -> Template {
    Template::builder()
      .body(
        Body::builder()
          .sections(vec![
            Section::builder()
              .columns(vec![
                Column::builder()
                  .children(vec![
                    Text::builder().content("Hello, world!").build().into(),
                  ])
                  .build(),
              ])
              .build(),
          ])
          .build(),
      )
      .build()
  }

  #[test]
  fn render_html_produces_html_document() {
    let html = render_html(&minimal_template()).unwrap();
    assert!(
      html.contains("<!doctype html>") || html.contains("<!DOCTYPE html>")
    );
    assert!(html.contains("Hello, world!"));
  }

  #[test]
  fn render_plaintext_extracts_text() {
    let html = "<p>Hello, <a href=\"https://example.com\">world</a>!</p>";
    let text = render_plaintext(html).unwrap();
    assert!(text.contains("Hello"));
    assert!(text.contains("world"));
    // html2text renders links with the URL in some form
    assert!(text.contains("example.com"));
  }

  #[test]
  fn end_to_end_pipeline() {
    let html = render_html(&minimal_template()).unwrap();
    let text = render_plaintext(&html).unwrap();
    assert!(text.contains("Hello, world!"));
  }
}
