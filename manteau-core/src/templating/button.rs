//! `mj-button` element.
//!
//! Constructor pattern: `Button::new(content, href)` — two required
//! arguments, one for the text and one for the destination URL. Does not
//! follow the unified container/text-bodied constructor pattern of the
//! other elements; consumers (including the eventual `mjml!` macro) must
//! handle this element's required attributes explicitly.

use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element},
};

/// `mj-button` — clickable button with required destination URL.
#[derive(Debug, Clone)]
pub struct Button {
  content:          String,
  href:             Url,
  background_color: Option<Color>,
  color:            Option<Color>,
  border_radius:    Option<Measurement>,
  font_size:        Option<Measurement>,
  font_weight:      Option<FontWeight>,
  inner_padding:    PaddingOptions,
  align:            Option<ButtonAlignment>,
}

impl Button {
  /// Button label before MJML escaping.
  pub fn content(&self) -> &str {
    &self.content
  }

  /// Checked destination URL.
  pub fn href(&self) -> &Url {
    &self.href
  }

  /// Create a button with required text and a checked destination.
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

  /// Set the checked background color.
  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
    self
  }

  /// Set the checked foreground color.
  pub fn color(mut self, color: impl Into<Color>) -> Self {
    self.color = Some(color.into());
    self
  }

  /// Round the element corners by this dimension.
  pub fn border_radius(mut self, radius: impl Into<Measurement>) -> Self {
    self.border_radius = Some(radius.into());
    self
  }

  /// Set the rendered font size.
  pub fn font_size(mut self, size: impl Into<Measurement>) -> Self {
    self.font_size = Some(size.into());
    self
  }

  /// Set the numeric font weight.
  pub fn font_weight(mut self, weight: impl Into<FontWeight>) -> Self {
    self.font_weight = Some(weight.into());
    self
  }

  /// Replace the padding inside the button.
  pub fn inner_padding(mut self, inner_padding: PaddingOptions) -> Self {
    self.inner_padding = inner_padding;
    self
  }

  /// Set top padding on the inside of the button.
  pub fn inner_padding_top(mut self, m: impl Into<Measurement>) -> Self {
    self.inner_padding = self.inner_padding.t(m.into());
    self
  }

  /// Set right padding on the inside of the button.
  pub fn inner_padding_right(mut self, m: impl Into<Measurement>) -> Self {
    self.inner_padding = self.inner_padding.r(m.into());
    self
  }

  /// Set bottom padding on the inside of the button.
  pub fn inner_padding_bottom(mut self, m: impl Into<Measurement>) -> Self {
    self.inner_padding = self.inner_padding.b(m.into());
    self
  }

  /// Set left padding on the inside of the button.
  pub fn inner_padding_left(mut self, m: impl Into<Measurement>) -> Self {
    self.inner_padding = self.inner_padding.l(m.into());
    self
  }

  /// Set horizontal alignment.
  pub fn align(mut self, align: impl Into<ButtonAlignment>) -> Self {
    self.align = Some(align.into());
    self
  }
}

impl Element for Button {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mj-button"))
      .attr(
        crate::render::AttributeName::builtin("href"),
        Some(&self.href),
      )
      .attr(
        crate::render::AttributeName::builtin("background-color"),
        self.background_color.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("color"),
        self.color.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("border-radius"),
        self.border_radius.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("font-size"),
        self.font_size.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("font-weight"),
        self.font_weight.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("align"),
        self.align.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("inner-padding-top"),
        self.inner_padding.top(),
      )
      .attr(
        crate::render::AttributeName::builtin("inner-padding-right"),
        self.inner_padding.right(),
      )
      .attr(
        crate::render::AttributeName::builtin("inner-padding-bottom"),
        self.inner_padding.bottom(),
      )
      .attr(
        crate::render::AttributeName::builtin("inner-padding-left"),
        self.inner_padding.left(),
      )
      .text(&self.content);
  }
}
