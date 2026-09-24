//! Strongly-typed attribute primitives for MJML element fields.

pub mod colors;
pub mod fonts;
pub mod measurements;
pub mod urls;
// Spacings contains options for Padding and Margin, as opposed to
// `measurements`, which contains the actual units of length that might be
// applied to a spacing.
pub mod spacings;

pub mod prelude {
  pub use crate::templating::attributes::{
    colors::*, fonts::*, measurements::*, spacings::*, urls::*,
  };
}

#[cfg(test)]
mod tests {
  use crate::templating::attributes::prelude::*;

  #[test]
  fn color_hex_24bit() {
    assert_eq!(Color::hex(0xff0000).to_string(), "#ff0000");
    assert_eq!(Color::hex(0x00ff00).to_string(), "#00ff00");
    assert_eq!(Color::hex(0x000001).to_string(), "#000001");
  }

  #[test]
  fn color_hex_masks_high_bits() {
    // Bits above 24 are stripped, not error.
    assert_eq!(Color::hex(0xff_ff_00_00).to_string(), "#ff0000");
  }

  #[test]
  fn color_try_parse_accepts_hex_and_named() {
    assert!(Color::try_parse("#fff").is_ok());
    assert!(Color::try_parse("#ffffff").is_ok());
    assert!(Color::try_parse("#ffffffff").is_ok());
    assert!(Color::try_parse("red").is_ok());
    assert!(Color::try_parse("cornflowerblue").is_ok());
  }

  #[test]
  fn color_try_parse_rejects_garbage() {
    assert!(Color::try_parse("nope nope").is_err());
    assert!(Color::try_parse("#zz").is_err());
    assert!(Color::try_parse("#12345").is_err()); // not a valid hex length
    assert!(Color::try_parse("").is_err());
    // Hyphens are not part of any real CSS named color.
    assert!(Color::try_parse("blue-grey").is_err());
  }

  #[test]
  fn url_try_from_str_and_string() {
    let from_str: Url = "https://example.com".try_into().unwrap();
    let from_string: Url =
      String::from("https://example.com").try_into().unwrap();
    assert_eq!(from_str, from_string);
  }

  #[test]
  fn pixels_display() {
    assert_eq!(Pixels::new(14).to_string(), "14px");
    assert_eq!(Pixels::from(0u32).to_string(), "0px");
  }

  #[test]
  fn percentage_bounds_enforced() {
    assert_eq!(Percentage::new(50).unwrap().to_string(), "50%");
    assert_eq!(Percentage::new(0).unwrap().to_string(), "0%");
    assert_eq!(Percentage::new(100).unwrap().to_string(), "100%");
    assert!(Percentage::new(101).is_err());
    assert!(Percentage::new(200).is_err());
  }

  #[test]
  fn url_validation() {
    assert!(Url::try_parse("https://example.com").is_ok());
    assert!(Url::try_parse("https://example.com/path?q=1").is_ok());
    assert!(Url::try_parse("mailto:hello@example.com").is_ok());
    assert!(Url::try_parse("not a url").is_err());
    assert!(Url::try_parse("").is_err());
  }

  #[test]
  fn alignment_display() {
    assert_eq!(Alignment::Left.to_string(), "left");
    assert_eq!(Alignment::Center.to_string(), "center");
    assert_eq!(Alignment::Right.to_string(), "right");
    assert_eq!(Alignment::Justify.to_string(), "justify");
  }

  #[test]
  fn font_family_passthrough() {
    let ff = FontFamily::new("Helvetica, Arial, sans-serif");
    assert_eq!(ff.to_string(), "Helvetica, Arial, sans-serif");
    let from: FontFamily = "Georgia".into();
    assert_eq!(from.to_string(), "Georgia");
  }
}
