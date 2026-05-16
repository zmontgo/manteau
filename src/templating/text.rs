use typed_builder::TypedBuilder;

use crate::{
  render::MjmlWriter,
  templating::{
    attributes::{Alignment, Color, FontFamily, Pixels},
    element::Element,
  },
};

/// `mj-text` — paragraph or run of styled text.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Text {
  #[builder(setter(into))]
  pub content:     String,
  #[builder(default, setter(strip_option))]
  pub color:       Option<Color>,
  #[builder(default, setter(strip_option, into))]
  pub font_size:   Option<Pixels>,
  #[builder(default, setter(strip_option, into))]
  pub font_family: Option<FontFamily>,
  #[builder(default, setter(strip_option))]
  pub align:       Option<Alignment>,
}

impl Element for Text {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-text")
      .attr("color", self.color.as_ref())
      .attr("font-size", self.font_size.as_ref())
      .attr("font-family", self.font_family.as_ref())
      .attr("align", self.align.as_ref())
      .text(&self.content);
  }
}
