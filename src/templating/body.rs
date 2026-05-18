use crate::{
  render::MjmlWriter,
  templating::{
    attributes::prelude::*, element::Element, section::Section,
    wrapper::Wrapper,
  },
};

/// One direct child of [`Body`] — either a bare [`Section`] or a
/// [`Wrapper`] containing further sections. [`Body`] holds a `Vec` of
/// these, the only enum in the templating tree that isn't a leaf-block
/// (the others are [`crate::templating::Block`]).
#[derive(Debug, Clone)]
pub enum BodyChild {
  Section(Section),
  Wrapper(Wrapper),
}

impl From<Section> for BodyChild {
  fn from(s: Section) -> Self { Self::Section(s) }
}

impl From<Wrapper> for BodyChild {
  fn from(w: Wrapper) -> Self { Self::Wrapper(w) }
}

impl Element for BodyChild {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    match self {
      Self::Section(s) => s.write_mjml(w),
      Self::Wrapper(wr) => wr.write_mjml(w),
    }
  }
}

/// `mj-body` — top-level container of [`Section`]s and/or [`Wrapper`]s.
/// Children render in declared order.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Body {
  pub children:         Vec<BodyChild>,
  pub background_color: Option<Color>,
  pub width:            Option<Pixels>,
}

impl Body {
  pub fn new() -> Self { Self::default() }

  /// Replace the children with a fresh `Vec` of sections. For mixed
  /// section/wrapper trees use [`push_section`] and [`push_wrapper`].
  ///
  /// [`push_section`]: Self::push_section
  /// [`push_wrapper`]: Self::push_wrapper
  pub fn sections(mut self, sections: Vec<Section>) -> Self {
    self.children = sections.into_iter().map(BodyChild::Section).collect();
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
    self.children.push(BodyChild::Section(section));
    self
  }

  /// Append one wrapper — a section-of-sections container that applies
  /// shared styling (background, padding, border-radius) to the group of
  /// sections inside it. See [`Wrapper`] for the canonical "card around
  /// several sections" use case.
  pub fn push_wrapper(mut self, wrapper: Wrapper) -> Self {
    self.children.push(BodyChild::Wrapper(wrapper));
    self
  }

  pub fn background_color(mut self, color: impl Into<Color>) -> Self {
    self.background_color = Some(color.into());
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
        for child in &self.children {
          child.write_mjml(w);
        }
      });
  }
}
