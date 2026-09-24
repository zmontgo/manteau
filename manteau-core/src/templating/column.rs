use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, block::Block, element::Element},
};

/// `mj-column` — vertical stack of [`Block`]s inside a [`Section`].
///
/// [`Section`]: crate::templating::section::Section
#[derive(Debug, Clone, Default)]
pub struct Column {
  children:         Vec<Block>,
  width:            Option<Percentage>,
  background_color: Option<Color>,
  border_radius:    Option<Measurement>,
  padding:          PaddingOptions,
}

impl Column {
  pub(crate) fn push_block(mut self, block: Block) -> Self {
    self.children.push(block);
    self
  }

  /// Direct leaf blocks in rendering order.
  pub fn blocks(&self) -> &[Block] {
    &self.children
  }

  /// Configured width, if any.
  pub fn configured_width(&self) -> Option<Percentage> {
    self.width
  }

  /// Construct an empty element ready for typed children and styling.
  pub fn new() -> Self {
    Self::default()
  }

  /// Replace the children with a whole new vec — for when you have the
  /// list upfront.
  pub fn children(mut self, children: Vec<Block>) -> Self {
    self.children = children;
    self
  }

  /// Set the width in the dimension accepted by this element.
  pub fn width(mut self, width: impl Into<Percentage>) -> Self {
    self.width = Some(width.into());
    self
  }

  /// Set the checked background color.
  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
    self
  }

  /// Round the element corners by this dimension.
  pub fn border_radius(
    mut self,
    border_radius: impl Into<Measurement>,
  ) -> Self {
    self.border_radius = Some(border_radius.into());
    self
  }

  /// Replace all configured padding sides.
  pub fn padding(mut self, padding: PaddingOptions) -> Self {
    self.padding = padding;
    self
  }

  /// Set top padding on the element.
  pub fn padding_top(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.t(m.into());
    self
  }

  /// Set right padding on the element.
  pub fn padding_right(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.r(m.into());
    self
  }

  /// Set bottom padding on the element.
  pub fn padding_bottom(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.b(m.into());
    self
  }

  /// Set left padding on the element.
  pub fn padding_left(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.l(m.into());
    self
  }
}

impl Element for Column {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mj-column"))
      .attr(
        crate::render::AttributeName::builtin("width"),
        self.width.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("background-color"),
        self.background_color.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("border-radius"),
        self.border_radius.as_ref(),
      )
      .attr(
        crate::render::AttributeName::builtin("padding-top"),
        self.padding.top(),
      )
      .attr(
        crate::render::AttributeName::builtin("padding-right"),
        self.padding.right(),
      )
      .attr(
        crate::render::AttributeName::builtin("padding-bottom"),
        self.padding.bottom(),
      )
      .attr(
        crate::render::AttributeName::builtin("padding-left"),
        self.padding.left(),
      )
      .children(|w| {
        for child in &self.children {
          child.write_mjml(w);
        }
      });
  }
}
