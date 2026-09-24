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
/// use manteau_core::prelude::*;
///
/// let card = Wrapper::new()
///   .background_color(Color::hex(0xffffff).unwrap())
///   .border_radius(Measurement::Pixels(Pixels::new(8)))
///   .push(Section::new().push(Column::new().push(Text::new("Hello"))));
/// let _body = Body::new().push(card);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Wrapper {
  sections:         Vec<Section>,
  background_color: Option<Color>,
  border_radius:    Option<Measurement>,
  padding:          PaddingOptions,
}

impl Wrapper {
  pub(crate) fn push_section(mut self, section: Section) -> Self {
    self.sections.push(section);
    self
  }

  /// Direct sections in rendering order.
  pub fn section_items(&self) -> &[Section] {
    &self.sections
  }

  /// Configured wrapper background color.
  pub fn configured_background_color(&self) -> Option<&Color> {
    self.background_color.as_ref()
  }

  /// Construct an empty element ready for typed children and styling.
  pub fn new() -> Self {
    Self::default()
  }

  /// Replace the wrapper sections with the supplied ordered list.
  pub fn sections(mut self, sections: Vec<Section>) -> Self {
    self.sections = sections;
    self
  }

  /// Set the checked background color.
  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
    self
  }

  /// Round the element corners by this dimension.
  pub fn border_radius(mut self, radius: impl Into<Measurement>) -> Self {
    self.border_radius = Some(radius.into());
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

impl Element for Wrapper {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open(crate::render::ElementName::builtin("mj-wrapper"))
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
        for section in &self.sections {
          section.write_mjml(w);
        }
      });
  }
}
