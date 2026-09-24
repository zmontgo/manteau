//! Compile-time parsing of attribute value strings into typed Rust
//! expressions.
//!
//! Attribute-specific literals use the same checked constructors as builders.
//! Measurements are recognized by their suffix where several attributes share
//! the same syntax. Other literals reach the setter unchanged.

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{LitStr, Result};

/// Parse a color using the same contract as the handwritten builder.
pub fn parse_color(lit: &LitStr, facade: &TokenStream) -> Result<TokenStream> {
  manteau_core::templating::attributes::colors::Color::try_parse(&lit.value())
    .map_err(|_| {
      syn::Error::new(
        lit.span(),
        "color must be a CSS named color or #rgb, #rgba, #rrggbb, or #rrggbbaa",
      )
    })?;

  Ok(quote_spanned! { lit.span() =>
    #facade::prelude::Color::try_parse(#lit)
      .expect("mjml!: color validated at expansion time")
  })
}

/// Parse a link as an admitted email destination.
pub fn parse_url(lit: &LitStr, facade: &TokenStream) -> Result<TokenStream> {
  manteau_core::templating::attributes::urls::Url::try_parse(&lit.value())
    .map_err(|_| {
      syn::Error::new(
        lit.span(),
        "href must be an HTTP(S), mailto, or tel URL without credentials",
      )
    })?;

  Ok(quote_spanned! { lit.span() =>
    #facade::prelude::Url::try_parse(#lit)
      .expect("mjml!: href validated at expansion time")
  })
}

pub fn parse_alignment(
  lit: &LitStr,
  button: bool,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let variant = match lit.value().as_str() {
    "left" => "Left",
    "center" => "Center",
    "right" => "Right",
    "justify" if !button => "Justify",
    _ => {
      return Err(syn::Error::new(
        lit.span(),
        "align must be left, center, or right (Text also supports justify)",
      ));
    }
  };
  let ty = if button {
    "ButtonAlignment"
  } else {
    "Alignment"
  };
  let ty = syn::Ident::new(ty, lit.span());
  let variant = syn::Ident::new(variant, lit.span());
  Ok(quote_spanned! { lit.span() => #facade::prelude::#ty::#variant })
}

pub fn parse_font_weight(
  lit: &LitStr,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let variant = match lit.value().as_str() {
    "100" => "Thin",
    "200" => "ExtraLight",
    "300" => "Light",
    "400" => "Normal",
    "500" => "Medium",
    "600" => "SemiBold",
    "700" => "Bold",
    "800" => "ExtraBold",
    "900" => "Black",
    _ => {
      return Err(syn::Error::new(
        lit.span(),
        "font-weight must be 100, 200, ..., 900",
      ));
    }
  };
  let variant = syn::Ident::new(variant, lit.span());
  Ok(quote_spanned! { lit.span() => #facade::prelude::FontWeight::#variant })
}

pub fn parse_text_transform(
  lit: &LitStr,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let variant = match lit.value().as_str() {
    "none" => "None",
    "uppercase" => "Uppercase",
    "lowercase" => "Lowercase",
    "capitalize" => "Capitalize",
    _ => {
      return Err(syn::Error::new(
        lit.span(),
        "text-transform must be none, uppercase, lowercase, or capitalize",
      ));
    }
  };
  let variant = syn::Ident::new(variant, lit.span());
  Ok(quote_spanned! { lit.span() => #facade::prelude::TextTransform::#variant })
}

pub fn parse_line_height(
  lit: &LitStr,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let value = lit.value();
  if let Ok(ratio) = value.parse::<f32>() {
    manteau_core::templating::attributes::measurements::PositiveFinite::new(
      ratio,
    )
    .map_err(|_| {
      syn::Error::new(
        lit.span(),
        "unitless line-height must be finite and positive",
      )
    })?;
    let ratio = syn::LitFloat::new(&format!("{ratio}f32"), lit.span());
    return Ok(quote_spanned! { lit.span() =>
      #facade::prelude::LineHeight::Unitless(
        #facade::prelude::PositiveFinite::new(#ratio)
          .expect("mjml!: line height validated at expansion time"))
    });
  }

  let measurement =
    try_parse_unit(&value, lit.span(), facade)?.ok_or_else(|| {
      syn::Error::new(
        lit.span(),
        "line-height must be a positive ratio or a CSS measurement",
      )
    })?;
  Ok(quote_spanned! { lit.span() =>
    #facade::prelude::LineHeight::Measurement(#measurement.into())
  })
}

/// Parse an image source under the narrower HTTP(S)-only contract.
pub fn parse_image_url(
  lit: &LitStr,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let value = lit.value();
  manteau_core::templating::attributes::urls::ImageUrl::try_parse(&value)
    .map_err(|_| {
      syn::Error::new(
        lit.span(),
        "image source must be an HTTP(S) URL without credentials",
      )
    })?;

  Ok(quote_spanned! { lit.span() =>
    #facade::prelude::ImageUrl::try_parse(#lit)
      .expect("manteau::mjml!: image URL validated at expansion time")
  })
}

/// Validate a literal font stack against core before emitting its constructor.
pub fn parse_font_family(
  lit: &LitStr,
  facade: &TokenStream,
) -> Result<TokenStream> {
  manteau_core::templating::attributes::fonts::FontFamily::new(&lit.value())
    .map_err(|_| {
      syn::Error::new(
        lit.span(),
        "font-family must be a comma-separated list of simple family names",
      )
    })?;

  Ok(quote_spanned! { lit.span() =>
    #facade::prelude::FontFamily::new(#lit)
      .expect("manteau::mjml!: font family validated at expansion time")
  })
}

/// Parse the contents of an attribute string literal and produce a typed
/// Rust expression. The span of the resulting expression is anchored on
/// the literal so errors point back at the source.
///
/// Attribute roles with stronger contracts are handled before this function.
pub fn parse_value(lit: &LitStr, facade: &TokenStream) -> Result<TokenStream> {
  let s = lit.value();
  let span = lit.span();

  if let Some(ts) = try_parse_unit(&s, span, facade)? {
    return Ok(ts);
  }

  // Fallback: pass the string through. Setter must accept it.
  Ok(quote_spanned! { span => #lit })
}

/// Recognizes unit-suffixed numeric literals: `20px`, `1.5em`, `50%`, etc.
/// Returns `None` if the string doesn't match a numeric-with-unit pattern.
fn try_parse_unit(
  s: &str,
  span: Span,
  facade: &TokenStream,
) -> Result<Option<TokenStream>> {
  // Find where the numeric portion ends.
  let trimmed = s.trim();
  let split = trimmed
    .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
    .unwrap_or(trimmed.len());
  let (num_part, unit_part) = trimmed.split_at(split);

  if num_part.is_empty() {
    // No numeric prefix; not a unit value.
    return Ok(None);
  }

  let unit = unit_part.trim();
  if unit.is_empty() {
    // Just a number with no unit. Not our business — pass through as
    // string.
    return Ok(None);
  }

  let ctor = match unit {
    "px" => emit_unit_int(num_part, "Pixels", span, facade)?,
    "em" => emit_unit_float(num_part, "Em", span, facade)?,
    "rem" => emit_unit_float(num_part, "Rem", span, facade)?,
    "%" => emit_percentage(num_part, span, facade)?,
    _ => return Ok(None),
  };

  Ok(Some(ctor))
}

fn emit_unit_int(
  num: &str,
  type_name: &str,
  span: Span,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let parsed: u32 = num.parse().map_err(|_| {
    syn::Error::new(
      span,
      format!("expected integer for `{}` value, got `{}`", type_name, num),
    )
  })?;
  let lit = syn::LitInt::new(&format!("{}u32", parsed), span);
  let ty = syn::Ident::new(type_name, span);
  Ok(quote_spanned! { span =>
    #facade::prelude::#ty::new(#lit)
  })
}

fn emit_unit_float(
  num: &str,
  type_name: &str,
  span: Span,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let parsed: f32 = num.parse().map_err(|_| {
    syn::Error::new(
      span,
      format!("expected number for `{}` value, got `{}`", type_name, num),
    )
  })?;
  if !parsed.is_finite() || parsed < 0.0 {
    return Err(syn::Error::new(
      span,
      format!("{} must be finite and nonnegative", type_name),
    ));
  }
  let lit = syn::LitFloat::new(&format!("{}f32", parsed), span);
  let ty = syn::Ident::new(type_name, span);
  Ok(quote_spanned! { span =>
    #facade::prelude::#ty::new(#lit)
      .expect("manteau::mjml!: measurement validated at expansion time")
  })
}

/// Percentages get their own emission path because `Percentage::new` takes
/// a `u8` (not `u32`) and is fallible (returns `Result<Self, _>`). We
/// validate the range at expansion time, then emit a `.expect(...)` that
/// is statically unreachable.
fn emit_percentage(
  num: &str,
  span: Span,
  facade: &TokenStream,
) -> Result<TokenStream> {
  let parsed: u8 = num.parse().map_err(|_| {
    syn::Error::new(
      span,
      format!("expected integer 0..=100 for percentage, got `{}`", num),
    )
  })?;
  if parsed > 100 {
    return Err(syn::Error::new(
      span,
      format!("percentage must be 0..=100, got {}", parsed),
    ));
  }
  let lit = syn::LitInt::new(&format!("{}u8", parsed), span);
  Ok(quote_spanned! { span =>
    #facade::prelude::Percentage::new(#lit)
      .expect("manteau::mjml!: percentage validated at expansion time")
  })
}
