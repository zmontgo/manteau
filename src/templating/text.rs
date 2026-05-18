use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element},
};

/// `mj-text` — paragraph or run of styled text.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Text {
  pub content:        String,
  pub color:          Option<Color>,
  pub font_size:      Option<Pixels>,
  pub font_family:    Option<FontFamily>,
  pub align:          Option<Alignment>,
  pub font_weight:    Option<FontWeight>,
  pub line_height:    Option<LineHeight>,
  pub letter_spacing: Option<Measurement>,
  pub text_transform: Option<TextTransform>,
  pub padding:        PaddingOptions,
}

impl Text {
  pub fn new(content: impl Into<String>) -> Self {
    Self {
      content:        content.into(),
      color:          None,
      font_size:      None,
      font_family:    None,
      align:          None,
      font_weight:    None,
      line_height:    None,
      letter_spacing: None,
      text_transform: None,
      padding:        PaddingOptions::default(),
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

  pub fn font_weight(mut self, weight: impl Into<FontWeight>) -> Self {
    self.font_weight = Some(weight.into());
    self
  }

  pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
    self.line_height = Some(line_height.into());
    self
  }

  pub fn letter_spacing(mut self, spacing: impl Into<Measurement>) -> Self {
    self.letter_spacing = Some(spacing.into());
    self
  }

  pub fn text_transform(mut self, transform: TextTransform) -> Self {
    self.text_transform = Some(transform);
    self
  }

  pub fn padding(mut self, padding: PaddingOptions) -> Self {
    self.padding = padding;
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
      .attr("font-weight", self.font_weight.as_ref())
      .attr("line-height", self.line_height.as_ref())
      .attr("letter-spacing", self.letter_spacing.as_ref())
      .attr("text-transform", self.text_transform.as_ref())
      .attr("padding-top", self.padding.top())
      .attr("padding-right", self.padding.right())
      .attr("padding-bottom", self.padding.bottom())
      .attr("padding-left", self.padding.left())
      .text(&self.content);
  }
}
