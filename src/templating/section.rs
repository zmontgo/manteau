use typed_builder::TypedBuilder;

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
#[derive(Debug, Clone, TypedBuilder)]
pub struct Section {
  #[builder(default)]
  pub columns:          Vec<Column>,
  #[builder(default, setter(strip_option))]
  pub background_color: Option<Color>,
  #[builder(default, setter(strip_option, into))]
  pub padding:          Option<Pixels>,
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
