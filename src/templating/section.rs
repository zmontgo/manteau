use crate::{
  render::MjmlWriter,
  templating::{
    attributes::{Color, Pixels},
    column::Column,
    element::Element,
  },
};

/// `mj-section` — horizontal row of [`Column`]s in a [`Body`].
///
/// [`Body`]: crate::templating::body::Body
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Section {
  pub columns:          Vec<Column>,
  pub background_color: Option<Color>,
  pub padding:          Option<Pixels>,
}

impl Section {
  pub fn new() -> Self { Self::default() }

  pub fn columns(mut self, columns: Vec<Column>) -> Self {
    self.columns = columns;
    self
  }

  /// Append one column.
  pub fn push_column(mut self, column: Column) -> Self {
    self.columns.push(column);
    self
  }

  pub fn background_color(mut self, color: Color) -> Self {
    self.background_color = Some(color);
    self
  }

  pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
    self.padding = Some(padding.into());
    self
  }
}

impl Element for Section {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-section")
      .attr("background-color", self.background_color.as_ref())
      .attr("padding", self.padding.as_ref())
      .children(|w| {
        for column in &self.columns {
          column.write_mjml(w);
        }
      });
  }
}
