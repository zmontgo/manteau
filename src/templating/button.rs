use typed_builder::TypedBuilder;

use crate::{
  render::MjmlWriter,
  templating::{
    attributes::{Color, Url},
    element::Element,
  },
};

/// `mj-button` — clickable button with required destination URL.
#[non_exhaustive]
#[derive(Debug, Clone, TypedBuilder)]
pub struct Button {
  #[builder(setter(into))]
  pub content:          String,
  pub href:             Url,
  #[builder(default, setter(strip_option))]
  pub background_color: Option<Color>,
  #[builder(default, setter(strip_option))]
  pub color:            Option<Color>,
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
