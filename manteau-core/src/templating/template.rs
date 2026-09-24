use crate::{
  render::MjmlWriter,
  templating::{body::Body, element::Element},
};

/// A typed MJML document with a required body and optional head content.
/// Its fields are private so document construction stays under this type's
/// contract as rendering features are added.
///
/// ```compile_fail
/// use manteau_core::templating::{Body, Template};
/// let mut template = Template::new(Body::new());
/// template.body = Body::new();
/// ```
#[derive(Debug, Clone)]
pub struct Template {
  body:         Body,
  preview_text: Option<String>,
  title:        Option<String>,
}

impl Template {
  /// Wrap an already structured MJML body in a complete document.
  pub fn new(body: Body) -> Self {
    Self {
      body,
      preview_text: None,
      title: None,
    }
  }

  /// Set the inbox preview text. Renders into `<mj-head><mj-preview>` —
  /// the head section is emitted automatically when preview_text or title
  /// is set.
  pub fn preview_text(mut self, text: impl Into<String>) -> Self {
    self.preview_text = Some(text.into());
    self
  }

  /// Set the document title. Renders into `<mj-head><mj-title>` — the
  /// head section is emitted automatically when title or preview_text is
  /// set.
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }
}

impl Element for Template {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mjml"))
      .children(|w| {
        if self.title.is_some() || self.preview_text.is_some() {
          w.open(crate::render::ElementName::builtin("mj-head"))
            .children(|w| {
              if let Some(t) = &self.title {
                w.open(crate::render::ElementName::builtin("mj-title"))
                  .text(t);
              }
              if let Some(p) = &self.preview_text {
                w.open(crate::render::ElementName::builtin("mj-preview"))
                  .text(p);
              }
            });
        }
        self.body.write_mjml(w);
      });
  }
}
