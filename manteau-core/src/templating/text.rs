use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element},
};

/// `mj-text` — paragraph or run of styled text.
#[derive(Debug, Clone)]
pub struct Text {
  content:        String,
  color:          Option<Color>,
  font_size:      Option<Pixels>,
  font_family:    Option<FontFamily>,
  align:          Option<Alignment>,
  font_weight:    Option<FontWeight>,
  line_height:    Option<LineHeight>,
  letter_spacing: Option<Measurement>,
  text_transform: Option<TextTransform>,
  padding:        PaddingOptions,
}

impl Text {
  /// Text content before MJML escaping.
  pub fn content(&self) -> &str { &self.content }

  /// Configured text color.
  pub fn configured_color(&self) -> Option<&Color> { self.color.as_ref() }

  /// Configured text size.
  pub fn configured_font_size(&self) -> Option<Pixels> { self.font_size }

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

  pub fn color(mut self, color: impl Into<Color>) -> Self {
    self.color = Some(color.into());
    self
  }

  pub fn font_size(mut self, size: impl Into<Pixels>) -> Self {
    self.font_size = Some(size.into());
    self
  }

  pub fn font_family(mut self, family: FontFamily) -> Self {
    self.font_family = Some(family);
    self
  }

  pub fn align(mut self, align: impl Into<Alignment>) -> Self {
    self.align = Some(align.into());
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

  pub fn text_transform(mut self, transform: impl Into<TextTransform>) -> Self {
    self.text_transform = Some(transform.into());
    self
  }

  pub fn padding(mut self, padding: PaddingOptions) -> Self {
    self.padding = padding;
    self
  }

  pub fn padding_top(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.t(m.into());
    self
  }

  pub fn padding_right(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.r(m.into());
    self
  }

  pub fn padding_bottom(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.b(m.into());
    self
  }

  pub fn padding_left(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.l(m.into());
    self
  }
}

impl Element for Text {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mj-text"))
      .attr(crate::render::AttributeName::builtin("color"), self.color.as_ref())
      .attr(crate::render::AttributeName::builtin("font-size"), self.font_size.as_ref())
      .attr(crate::render::AttributeName::builtin("font-family"), self.font_family.as_ref())
      .attr(crate::render::AttributeName::builtin("align"), self.align.as_ref())
      .attr(crate::render::AttributeName::builtin("font-weight"), self.font_weight.as_ref())
      .attr(crate::render::AttributeName::builtin("line-height"), self.line_height.as_ref())
      .attr(crate::render::AttributeName::builtin("letter-spacing"), self.letter_spacing.as_ref())
      .attr(crate::render::AttributeName::builtin("text-transform"), self.text_transform.as_ref())
      .attr(crate::render::AttributeName::builtin("padding-top"), self.padding.top())
      .attr(crate::render::AttributeName::builtin("padding-right"), self.padding.right())
      .attr(crate::render::AttributeName::builtin("padding-bottom"), self.padding.bottom())
      .attr(crate::render::AttributeName::builtin("padding-left"), self.padding.left())
      .text(&self.content);
  }
}
