use std::sync::Arc;

use crate::{
  render::MjmlWriter,
  templating::{button::Button, element::Element, image::Image, text::Text},
};

/// Anything that can live inside a [`Column`] — a leaf MJML element.
///
/// Library-known leaves are first-class variants for ergonomic, compile-time
/// typed construction. Consumer-defined elements plug in via [`Block::Custom`]
/// (any type implementing [`Element`]) — see [`Block::custom`].
///
/// [`Column`]: crate::templating::column::Column
#[derive(Debug, Clone)]
pub enum Block {
  Text(Text),
  Button(Button),
  Image(Image),
  /// A consumer-defined element. Held by [`Arc`] so [`Block`] (and the
  /// containers built on it) can be `Clone` without pulling in a
  /// dyn-cloning crate.
  Custom(Arc<dyn Element>),
}

impl Block {
  /// Wrap any [`Element`] into a [`Block`]. Convenience for the
  /// consumer-extension path:
  ///
  /// ```
  /// use manteau::render::MjmlWriter;
  /// use manteau::templating::{Block, Element};
  ///
  /// #[derive(Debug)]
  /// struct MyDivider;
  ///
  /// impl Element for MyDivider {
  ///     fn write_mjml(&self, w: &mut MjmlWriter) {
  ///         w.open("mj-divider").close_self();
  ///     }
  /// }
  ///
  /// let _block = Block::custom(MyDivider);
  /// ```
  pub fn custom<E: Element + 'static>(element: E) -> Self {
    Self::Custom(Arc::new(element))
  }
}

impl Element for Block {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    match self {
      Self::Text(t) => t.write_mjml(w),
      Self::Button(b) => b.write_mjml(w),
      Self::Image(i) => i.write_mjml(w),
      Self::Custom(c) => c.write_mjml(w),
    }
  }
}

impl From<Text> for Block {
  fn from(t: Text) -> Self { Self::Text(t) }
}

impl From<Button> for Block {
  fn from(b: Button) -> Self { Self::Button(b) }
}

impl From<Image> for Block {
  fn from(i: Image) -> Self { Self::Image(i) }
}
