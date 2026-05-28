use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element, section::Section},
};

/// `mj-wrapper` — a section-of-sections container. Wraps multiple
/// [`Section`]s with shared styling (background, padding, border-radius) so
/// a group of sections renders as one outer styled rectangle.
///
/// The structural reason this primitive exists: [`Section`] is MJML's unit
/// of styled rectangular containment but does not nest. When a layout needs
/// several sections to live inside one shared visual container (a card with
/// rounded corners and a unified background, say), `mj-wrapper` is the
/// MJML-native container that provides it. Without it the alternatives are
/// raw HTML inside `mj-text` or multiple sections with matching backgrounds
/// — both lose something the wrapper preserves.
///
/// ```
/// use manteau::prelude::*;
///
/// let card = Wrapper::new()
///   .background_color(Color::hex(0xffffff))
///   .border_radius(Measurement::Pixels(Pixels::new(8)))
///   .push(Section::new().push(Column::new().push(Text::new("Hello"))));
/// let _body = Body::new().push(card);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Wrapper {
  pub sections:         Vec<Section>,
  pub background_color: Option<Color>,
  pub border_radius:    Option<Measurement>,
  pub padding:          PaddingOptions,
}

impl Wrapper {
  pub fn new() -> Self { Self::default() }

  pub fn sections(mut self, sections: Vec<Section>) -> Self {
    self.sections = sections;
    self
  }

  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
    self
  }

  pub fn border_radius(mut self, radius: impl Into<Measurement>) -> Self {
    self.border_radius = Some(radius.into());
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

impl Element for Wrapper {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-wrapper")
      .attr("background-color", self.background_color.as_ref())
      .attr("border-radius", self.border_radius.as_ref())
      .attr("padding-top", self.padding.top())
      .attr("padding-right", self.padding.right())
      .attr("padding-bottom", self.padding.bottom())
      .attr("padding-left", self.padding.left())
      .children(|w| {
        for section in &self.sections {
          section.write_mjml(w);
        }
      });
  }
}
