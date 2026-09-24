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
///
/// ```compile_fail
/// use manteau_core::templating::{Image, attributes::urls::Url};
/// let link = Url::try_parse("mailto:person@example.com").unwrap();
/// let image = Image::new(link);
/// ```
#[derive(Debug, Clone)]
pub struct Image {
  src:   ImageUrl,
  alt:   Option<String>,
  href:  Option<Url>,
  width: Option<Pixels>,
}

impl Image {
  /// Checked image source URL.
  pub fn src(&self) -> &ImageUrl {
    &self.src
  }

  /// Create an image from an admitted HTTP(S) source.
  pub fn new(src: ImageUrl) -> Self {
    Self {
      src,
      alt: None,
      href: None,
      width: None,
    }
  }

  /// Set alternative text for clients that do not display the image.
  pub fn alt(mut self, alt: impl Into<String>) -> Self {
    self.alt = Some(alt.into());
    self
  }

  /// Set the checked link destination.
  pub fn href(mut self, href: impl Into<Url>) -> Self {
    self.href = Some(href.into());
    self
  }

  /// Set the width in the dimension accepted by this element.
  pub fn width(mut self, width: impl Into<Pixels>) -> Self {
    self.width = Some(width.into());
    self
  }
}

impl Element for Image {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mj-image"))
      .attr(
        crate::render::AttributeName::builtin("src"),
        Some(&self.src),
      )
      .attr(
        crate::render::AttributeName::builtin("alt"),
        self.alt.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("href"),
        self.href.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("width"),
        self.width.as_ref(),
      )
      .close_self();
  }
}
