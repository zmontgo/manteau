use typed_builder::TypedBuilder;

use crate::render::MjmlWriter;
use crate::templating::attributes::Percentage;
use crate::templating::block::Block;
use crate::templating::element::Element;

/// `mj-column` — vertical stack of [`Block`]s inside a [`Section`].
///
/// [`Section`]: crate::templating::section::Section
#[derive(Debug, TypedBuilder)]
pub struct Column {
    #[builder(default)]
    pub children: Vec<Block>,
    #[builder(default, setter(strip_option))]
    pub width: Option<Percentage>,
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
