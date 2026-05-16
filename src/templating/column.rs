use crate::{
  render::MjmlWriter,
  templating::{attributes::Percentage, block::Block, element::Element},
};

/// `mj-column` — vertical stack of [`Block`]s inside a [`Section`].
///
/// [`Section`]: crate::templating::section::Section
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Column {
  pub children: Vec<Block>,
  pub width:    Option<Percentage>,
}

impl Column {
  pub fn new() -> Self { Self::default() }

  /// Replace the children with a whole new vec — for when you have the
  /// list upfront.
  pub fn children(mut self, children: Vec<Block>) -> Self {
    self.children = children;
    self
  }

  /// Append one child. The runtime-modification path:
  ///
  /// ```
  /// # use manteau::templating::{Column, Text};
  /// let mut col = Column::new().push(Text::new("Welcome!"));
  /// # let coupon_eligible = true;
  /// if coupon_eligible {
  ///   col = col.push(Text::new("Here's a coupon"));
  /// }
  /// ```
  pub fn push(mut self, child: impl Into<Block>) -> Self {
    self.children.push(child.into());
    self
  }

  pub fn width(mut self, width: Percentage) -> Self {
    self.width = Some(width);
    self
  }
}

impl Element for Column {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-column")
      .attr("width", self.width.as_ref())
      .children(|w| {
        for child in &self.children {
          child.write_mjml(w);
        }
      });
  }
}
