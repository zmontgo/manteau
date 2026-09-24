use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, column::Column, element::Element},
};

/// `mj-section` — horizontal row of [`Column`]s in a [`Body`].
///
/// [`Body`]: crate::templating::body::Body
#[derive(Debug, Clone, Default)]
pub struct Section {
  columns:          Vec<Column>,
  background_color: Option<Color>,
  padding:          PaddingOptions,
}

impl Section {
  pub(crate) fn push_column(mut self, column: Column) -> Self {
    self.columns.push(column);
    self
  }

  /// Direct columns in rendering order.
  pub fn column_items(&self) -> &[Column] {
    &self.columns
  }

  /// Configured section background color.
  pub fn configured_background_color(&self) -> Option<&Color> {
    self.background_color.as_ref()
  }

  /// Construct an empty element ready for typed children and styling.
  pub fn new() -> Self {
    Self::default()
  }

  /// Replace the section columns with the supplied ordered list.
  pub fn columns(mut self, columns: Vec<Column>) -> Self {
    self.columns = columns;
    self
  }

  /// Set the checked background color.
  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
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

impl Element for Section {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mj-section"))
      .attr(
        crate::render::AttributeName::builtin("background-color"),
        self.background_color.as_ref(),
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
        for column in &self.columns {
          column.write_mjml(w);
        }
      });
  }
}
