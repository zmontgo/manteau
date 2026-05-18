use crate::templating::attributes::measurements::Measurement;

/// All options for the padding of an element. Can be cleanly constructed,
/// tailwind-style.
#[derive(Debug, Clone, Copy)]
pub struct PaddingOptions {
  top:    Option<Measurement>,
  bottom: Option<Measurement>,
  left:   Option<Measurement>,
  right:  Option<Measurement>,
}

impl Default for PaddingOptions {
  fn default() -> Self {
    PaddingOptions {
      top:    None,
      bottom: None,
      left:   None,
      right:  None,
    }
  }
}

impl PaddingOptions {
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
}
