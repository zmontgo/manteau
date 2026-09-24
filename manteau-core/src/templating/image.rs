//! `mj-image` element.
//!
//! Constructor pattern: `Image::new(src)` — one required argument for the
//! source URL. Does not follow the unified container/text-bodied
//! constructor pattern of the other elements; consumers (including the
//! eventual `mjml!` macro) must handle this element's required `src`
//! attribute explicitly.

use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element},
};

/// `mj-image` — embedded image with required source URL.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Image {
  pub src:   Url,
  pub alt:   Option<String>,
  pub href:  Option<Url>,
  pub width: Option<Pixels>,
}

impl Image {
  pub fn new(src: Url) -> Self {
    Self {
      src,
      alt: None,
      href: None,
      width: None,
    }
  }

  pub fn alt(mut self, alt: impl Into<String>) -> Self {
    self.alt = Some(alt.into());
    self
  }

  pub fn href(mut self, href: impl Into<Url>) -> Self {
    self.href = Some(href.into());
    self
  }

  pub fn width(mut self, width: impl Into<Pixels>) -> Self {
    self.width = Some(width.into());
    self
  }
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
