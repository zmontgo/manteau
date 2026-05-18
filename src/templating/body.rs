use crate::{
  render::MjmlWriter,
  templating::{attributes::prelude::*, element::Element, section::Section},
};

/// `mj-body` — top-level container of [`Section`]s.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Body {
  pub sections:         Vec<Section>,
  pub background_color: Option<Color>,
  pub width:            Option<Pixels>,
}

impl Body {
  pub fn new() -> Self { Self::default() }

  pub fn sections(mut self, sections: Vec<Section>) -> Self {
    self.sections = sections;
    self
  }

  /// Append one section. The runtime-modification path — used to
  /// conditionally tack a section on (a coupon offer in a welcome email,
  /// say) without rebuilding the whole tree.
  ///
  /// ```
  /// # use manteau::templating::{Body, Section};
  /// # let user_coupon_eligible = true;
  /// # let welcome = Section::new();
  /// # let coupon = Section::new();
  /// let mut body = Body::new().push_section(welcome);
  /// if user_coupon_eligible {
  ///   body = body.push_section(coupon);
  /// }
  /// ```
  pub fn push_section(mut self, section: Section) -> Self {
    self.sections.push(section);
    self
  }

  pub fn background_color(mut self, color: Color) -> Self {
    self.background_color = Some(color);
    self
  }

  pub fn width(mut self, width: impl Into<Pixels>) -> Self {
    self.width = Some(width.into());
    self
  }
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
