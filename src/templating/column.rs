use typed_builder::TypedBuilder;

use crate::{
  render::MjmlWriter,
  templating::{attributes::Percentage, block::Block, element::Element},
};

/// `mj-column` — vertical stack of [`Block`]s inside a [`Section`].
///
/// [`Section`]: crate::templating::section::Section
#[non_exhaustive]
#[derive(Debug, Clone, TypedBuilder)]
pub struct Column {
  #[builder(default)]
  pub children: Vec<Block>,
  #[builder(default, setter(strip_option))]
  pub width:    Option<Percentage>,
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
