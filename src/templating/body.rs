use typed_builder::TypedBuilder;

use crate::render::MjmlWriter;
use crate::templating::attributes::{Color, Pixels};
use crate::templating::element::Element;
use crate::templating::section::Section;

/// `mj-body` — top-level container of [`Section`]s.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Body {
    #[builder(default)]
    pub sections: Vec<Section>,
    #[builder(default, setter(strip_option))]
    pub background_color: Option<Color>,
    #[builder(default, setter(strip_option, into))]
    pub width: Option<Pixels>,
}

impl Element for Body {
    fn write_mjml(&self, w: &mut MjmlWriter) {
        w.open("mj-body")
            .attr("background-color", self.background_color.as_ref())
            .attr("width", self.width.as_ref())
            .children(|w| {
                for section in &self.sections {
                    section.write_mjml(w);
                }
            });
    }
}
