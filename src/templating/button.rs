use crate::{
  render::MjmlWriter,
  templating::{
    attributes::{Color, Url},
    element::Element,
  },
};

/// `mj-button` — clickable button with required destination URL.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Button {
  pub content:          String,
  pub href:             Url,
  pub background_color: Option<Color>,
  pub color:            Option<Color>,
}

impl Button {
  pub fn new(content: impl Into<String>, href: Url) -> Self {
    Self {
      content:          content.into(),
      href,
      background_color: None,
      color:            None,
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
}

impl Element for Button {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-button")
      .attr("href", Some(&self.href))
      .attr("background-color", self.background_color.as_ref())
      .attr("color", self.color.as_ref())
      .text(&self.content);
  }
}
