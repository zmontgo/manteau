/// Most types of measurements.
#[derive(Debug, Clone, Copy)]
pub enum Measurement {
  Pixels(Pixels),
  Percentage(Percentage),
  Em(Em),
  Rem(Rem),
}

impl std::fmt::Display for Measurement {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let inner: &dyn std::fmt::Display = match self {
      Measurement::Percentage(pc) => pc,
      Measurement::Pixels(px) => px,
      Measurement::Em(em) => em,
      Measurement::Rem(rm) => rm,
    };
    inner.fmt(f)
  }
}

/// Pixel dimension — rendered as `Npx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pixels(u32);

impl Pixels {
  /// ```
  /// # use manteau::templating::attributes::measurements::Pixels;
  /// assert_eq!(Pixels::new(14).to_string(), "14px");
  /// ```
  pub fn new(value: u32) -> Self { Self(value) }

  pub fn value(self) -> u32 { self.0 }
}

impl From<u32> for Pixels {
  fn from(value: u32) -> Self { Self::new(value) }
}

impl std::fmt::Display for Pixels {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}px", self.0)
  }
}

/// Em dimension — rendered as `Nem`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Em(f32);

impl Em {
  /// ```
  /// # use manteau::templating::attributes::prelude::*;
  /// assert_eq!(Em::new(1.3).to_string(), "1.3em");
  /// ```
  pub fn new(value: f32) -> Self { Self(value) }

  pub fn value(self) -> f32 { self.0 }
}

impl From<f32> for Em {
  fn from(value: f32) -> Self { Self::new(value) }
}

impl std::fmt::Display for Em {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}em", self.0)
  }
}

/// Rem dimension — rendered as `Nrem` (hopefully you're sleeping well too.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rem(f32);

impl Rem {
  /// ```
  /// # use manteau::templating::attributes::prelude::*;
  /// assert_eq!(Rem::new(1.3).to_string(), "1.3rem");
  /// ```
  pub fn new(value: f32) -> Self { Self(value) }

  pub fn value(self) -> f32 { self.0 }
}

impl From<f32> for Rem {
  fn from(value: f32) -> Self { Self::new(value) }
}

impl std::fmt::Display for Rem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}rem", self.0)
  }
}

// ---------- Percentage ----------

/// Percentage dimension (0-100) — rendered as `N%`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Percentage(u8);

#[derive(Debug, thiserror::Error)]
#[error("percentage must be 0..=100: got {value}")]
pub struct PercentageError {
  value: u32,
}

impl PercentageError {
  pub fn value(&self) -> u32 { self.value }
}

impl Percentage {
  /// Build a percentage, enforcing the 0..=100 invariant.
  ///
  /// ```
  /// # use manteau::templating::attributes::measurements::Percentage;
  /// assert_eq!(Percentage::new(50).unwrap().to_string(), "50%");
  /// assert!(Percentage::new(101).is_err());
  /// ```
  pub fn new(value: u8) -> Result<Self, PercentageError> {
    if value <= 100 {
      Ok(Self(value))
    } else {
      Err(PercentageError {
        value: u32::from(value),
      })
    }
  }

  pub fn value(self) -> u8 { self.0 }
}

impl std::fmt::Display for Percentage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}%", self.0)
  }
}

/// Line height options. The preferred way (according to MDN) is unitless.
#[derive(Debug, Clone, Copy)]
pub enum LineHeight {
  Unitless(f32),
  Measurement(Measurement),
}

impl std::fmt::Display for LineHeight {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let inner: &dyn std::fmt::Display = match self {
      // f32 implements Display, and since we don't need units, this works
      // perfectly!
      LineHeight::Unitless(ul) => ul,
      LineHeight::Measurement(ms) => ms,
    };
    inner.fmt(f)
  }
}
