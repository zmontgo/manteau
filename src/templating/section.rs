use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, column::Column, element::Element},
};

/// `mj-section` — horizontal row of [`Column`]s in a [`Body`].
///
/// [`Body`]: crate::templating::body::Body
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Section {
  pub columns:          Vec<Column>,
  pub background_color: Option<Color>,
  pub padding:          PaddingOptions,
}

impl Section {
  pub fn new() -> Self { Self::default() }

  pub fn columns(mut self, columns: Vec<Column>) -> Self {
    self.columns = columns;
    self
  }

  /// Append one column.
  ///
  /// ```
  /// # use manteau::templating::{Column, Section};
  /// let section = Section::new()
  ///   .push_column(Column::new())
  ///   .push_column(Column::new());
  /// ```
  pub fn push_column(mut self, column: Column) -> Self {
    self.columns.push(column);
    self
  }

  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
    self
  }

  pub fn padding(mut self, padding: PaddingOptions) -> Self {
    self.padding = padding;
    self
  }

  pub fn padding_top(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.t(m.into());
    self
  }

  pub fn padding_right(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.r(m.into());
    self
  }

  pub fn padding_bottom(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.b(m.into());
    self
  }

  pub fn padding_left(mut self, m: impl Into<Measurement>) -> Self {
    self.padding = self.padding.l(m.into());
    self
  }
}

impl Element for Section {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-section")
      .attr("background-color", self.background_color.as_ref())
      .attr("padding-top", self.padding.top())
      .attr("padding-right", self.padding.right())
      .attr("padding-bottom", self.padding.bottom())
      .attr("padding-left", self.padding.left())
      .children(|w| {
        for column in &self.columns {
          column.write_mjml(w);
        }
      });
  }
}
