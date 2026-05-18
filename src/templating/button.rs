use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element},
};

/// `mj-button` — clickable button with required destination URL.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Button {
  pub content:          String,
  pub href:             Url,
  pub background_color: Option<Color>,
  pub color:            Option<Color>,
  pub border_radius:    Option<Measurement>,
  pub font_size:        Option<Measurement>,
  pub font_weight:      Option<FontWeight>,
  pub inner_padding:    PaddingOptions,
  pub align:            Option<ButtonAlignment>,
}

impl Button {
  pub fn new(content: impl Into<String>, href: Url) -> Self {
    Self {
      content: content.into(),
      href,
      background_color: None,
      color: None,
      border_radius: None,
      font_size: None,
      font_weight: None,
      inner_padding: PaddingOptions::default(),
      align: None,
    }
  }

  pub fn background_color(mut self, color: Color) -> Self {
    self.background_color = Some(color);
    self
  }

  pub fn color(mut self, color: Color) -> Self {
    self.color = Some(color);
    self
  }

  pub fn border_radius(mut self, radius: impl Into<Measurement>) -> Self {
    self.border_radius = Some(radius.into());
    self
  }

  pub fn font_size(mut self, size: impl Into<Measurement>) -> Self {
    self.font_size = Some(size.into());
    self
  }

  pub fn font_weight(mut self, weight: impl Into<FontWeight>) -> Self {
    self.font_weight = Some(weight.into());
    self
  }

  pub fn inner_padding(mut self, inner_padding: PaddingOptions) -> Self {
    self.inner_padding = inner_padding;
    self
  }

  pub fn align(mut self, align: ButtonAlignment) -> Self {
    self.align = Some(align);
    self
  }
}

impl Element for Button {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-button")
      .attr("href", Some(&self.href))
      .attr("background-color", self.background_color.as_ref())
      .attr("color", self.color.as_ref())
      .attr("border-radius", self.border_radius.as_ref())
      .attr("font-size", self.font_size.as_ref())
      .attr("font-weight", self.font_weight.as_ref())
      .attr("align", self.align.as_ref())
      .attr("inner-padding-top", self.inner_padding.top())
      .attr("inner-padding-right", self.inner_padding.right())
      .attr("inner-padding-bottom", self.inner_padding.bottom())
      .attr("inner-padding-left", self.inner_padding.left())
      .text(&self.content);
  }
}
