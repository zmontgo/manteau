use std::sync::atomic::{AtomicUsize, Ordering};

use manteau_core::{
  HtmlBody, InvalidHtmlBody, MjmlDocument, PlaintextBody, RenderErrorKind,
  Renderer,
  templating::{Body, Column, Push, Section, Template, Text},
};

struct ProbeRenderer {
  html:            String,
  text:            String,
  html_calls:      AtomicUsize,
  plaintext_calls: AtomicUsize,
}

impl ProbeRenderer {
  fn new(html: &str, text: &str) -> Self {
    Self {
      html:            html.to_owned(),
      text:            text.to_owned(),
      html_calls:      AtomicUsize::new(0),
      plaintext_calls: AtomicUsize::new(0),
    }
  }
}

impl Renderer for ProbeRenderer {
  type Error = InvalidHtmlBody;

  fn html(&self, mjml: &MjmlDocument) -> Result<HtmlBody, Self::Error> {
    assert!(mjml.as_str().contains("<mj-text>hello</mj-text>"));
    self.html_calls.fetch_add(1, Ordering::SeqCst);
    HtmlBody::new(self.html.clone())
  }

  fn plaintext(&self, html: &HtmlBody) -> Result<PlaintextBody, Self::Error> {
    assert_eq!(html.as_str(), self.html);
    self.plaintext_calls.fetch_add(1, Ordering::SeqCst);
    Ok(PlaintextBody::new(self.text.clone()).unwrap())
  }
}

#[test]
fn core_composes_rendering_without_a_concrete_library() {
  let renderer = ProbeRenderer::new("<p>hello</p>", "hello");
  let template = Template::new(
    Body::new()
      .push(Section::new().push(Column::new().push(Text::new("hello")))),
  );
  let rendered = template.render(&renderer).unwrap();

  assert_eq!(rendered.html(), "<p>hello</p>");
  assert_eq!(rendered.text(), "hello");
  assert_eq!(renderer.html_calls.load(Ordering::SeqCst), 1);
  assert_eq!(renderer.plaintext_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn supplied_plaintext_skips_conversion_and_empty_html_is_rejected() {
  let renderer = ProbeRenderer::new("<p>hello</p>", "unused");
  let template = Template::new(
    Body::new()
      .push(Section::new().push(Column::new().push(Text::new("hello")))),
  );
  let supplied = PlaintextBody::new("supplied").unwrap();
  let rendered = template
    .render_with_text(&renderer, Some(&supplied))
    .unwrap();

  assert_eq!(rendered.text(), "supplied");
  assert_eq!(renderer.html_calls.load(Ordering::SeqCst), 1);
  assert_eq!(renderer.plaintext_calls.load(Ordering::SeqCst), 0);

  let empty = ProbeRenderer::new("", "");
  let template = Template::new(
    Body::new()
      .push(Section::new().push(Column::new().push(Text::new("hello")))),
  );
  let error = template.render(&empty).unwrap_err();
  assert_eq!(error.kind(), RenderErrorKind::Html);
}
