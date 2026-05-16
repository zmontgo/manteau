use typed_builder::TypedBuilder;

use crate::{
  render::MjmlWriter,
  templating::{
    attributes::{Pixels, Url},
    element::Element,
  },
};

/// `mj-image` — embedded image with required source URL.
#[non_exhaustive]
#[derive(Debug, Clone, TypedBuilder)]
pub struct Image {
  pub src:   Url,
  #[builder(default, setter(strip_option, into))]
  pub alt:   Option<String>,
  #[builder(default, setter(strip_option))]
  pub href:  Option<Url>,
  #[builder(default, setter(strip_option, into))]
  pub width: Option<Pixels>,
}

impl Element for Image {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-image")
      .attr("src", Some(&self.src))
      .attr("alt", self.alt.as_ref())
      .attr("href", self.href.as_ref())
      .attr("width", self.width.as_ref())
      .close_self();
  }
}
