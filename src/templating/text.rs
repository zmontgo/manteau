use crate::{
  render::MjmlWriter,
  templating::{
    attributes::{Alignment, Color, FontFamily, Pixels},
    element::Element,
  },
};

/// `mj-text` — paragraph or run of styled text.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Text {
  pub content:     String,
  pub color:       Option<Color>,
  pub font_size:   Option<Pixels>,
  pub font_family: Option<FontFamily>,
  pub align:       Option<Alignment>,
}

impl Text {
  pub fn new(content: impl Into<String>) -> Self {
    Self {
      content:     content.into(),
      color:       None,
      font_size:   None,
      font_family: None,
      align:       None,
    }
  }

  pub fn color(mut self, color: Color) -> Self {
    self.color = Some(color);
    self
  }

  pub fn font_size(mut self, size: impl Into<Pixels>) -> Self {
    self.font_size = Some(size.into());
    self
  }

  pub fn font_family(mut self, family: impl Into<FontFamily>) -> Self {
    self.font_family = Some(family.into());
    self
  }

  pub fn align(mut self, align: Alignment) -> Self {
    self.align = Some(align);
    self
  }
}

impl Element for Text {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-text")
      .attr("color", self.color.as_ref())
      .attr("font-size", self.font_size.as_ref())
      .attr("font-family", self.font_family.as_ref())
      .attr("align", self.align.as_ref())
      .text(&self.content);
  }
}
