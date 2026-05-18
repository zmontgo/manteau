use crate::templating::attributes::measurements::Measurement;

/// Per-side padding for elements that accept one. Builders are tailwind-style
/// (`x`/`y` for axes, `t`/`r`/`b`/`l` for individual sides); accessors return
/// the per-side value or `None` when unset.
///
/// Elements emit padding as four separate MJML attributes (`padding-top`,
/// `padding-right`, `padding-bottom`, `padding-left`) — only the sides that
/// were set produce an attribute. Unset sides inherit the element's MJML
/// default, never `0`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PaddingOptions {
  top:    Option<Measurement>,
  bottom: Option<Measurement>,
  left:   Option<Measurement>,
  right:  Option<Measurement>,
}

impl PaddingOptions {
  /// New empty padding — every side unset. Equivalent to `Default::default()`.
  pub fn new() -> Self { Self::default() }

  pub fn x(mut self, padding: Measurement) -> Self {
    self.left = Some(padding);
    self.right = Some(padding);
    self
  }

  pub fn y(mut self, padding: Measurement) -> Self {
    self.top = Some(padding);
    self.bottom = Some(padding);
    self
  }

  pub fn t(mut self, padding: Measurement) -> Self {
    self.top = Some(padding);
    self
  }

  pub fn l(mut self, padding: Measurement) -> Self {
    self.left = Some(padding);
    self
  }

  pub fn r(mut self, padding: Measurement) -> Self {
    self.right = Some(padding);
    self
  }

  pub fn b(mut self, padding: Measurement) -> Self {
    self.bottom = Some(padding);
    self
  }

  pub fn top(&self) -> Option<&Measurement> { self.top.as_ref() }
  pub fn bottom(&self) -> Option<&Measurement> { self.bottom.as_ref() }
  pub fn left(&self) -> Option<&Measurement> { self.left.as_ref() }
  pub fn right(&self) -> Option<&Measurement> { self.right.as_ref() }

  /// `true` when no side has been set. Elements use this to decide whether
  /// to emit any padding attribute at all.
  pub fn is_empty(&self) -> bool {
    self.top.is_none()
      && self.bottom.is_none()
      && self.left.is_none()
      && self.right.is_none()
  }
}
