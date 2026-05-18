//! Compile-time parsing of attribute value strings into typed Rust
//! expressions.
//!
//! The macro accepts attribute values as string literals: `font-size="20px"`,
//! `color="#f00"`, `width="50%"`, `src="https://..."`. This module recognizes
//! the syntax inside the quotes and produces the right typed constructor —
//! `Pixels::new(20)`, `Color::hex(0xff0000)`, `Url::try_parse("...")...`, etc.
//!
//! Anything the recognizers don't match falls through as the original
//! string literal — setters that accept `impl Into<String>` (e.g.
//! `font_family`) consume it directly.

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{LitStr, Result};

/// Parse the contents of an attribute string literal and produce a typed
/// Rust expression. The span of the resulting expression is anchored on
/// the literal so errors point back at the source.
///
/// Dispatch order matters: URL detection runs before unit detection so
/// `"3px"` is not mistaken for a URL fragment, and color before URL so
/// `"#f00"` doesn't drift into the URL branch via a stray `://` substring.
pub fn parse_value(lit: &LitStr) -> Result<TokenStream> {
  let s = lit.value();
  let span = lit.span();

  if let Some(ts) = try_parse_color(&s, span)? {
    return Ok(ts);
  }
  if let Some(ts) = try_parse_unit(&s, span)? {
    return Ok(ts);
  }
  if let Some(ts) = try_parse_url(&s, span)? {
    return Ok(ts);
  }

  // Fallback: pass the string through. Setter must accept it.
  Ok(quote_spanned! { span => #lit })
}

/// Recognizes `#rgb` and `#rrggbb` color hex strings. Returns `None` if the
/// string doesn't start with `#`. Returns `Err` if it starts with `#` but
/// is malformed.
fn try_parse_color(s: &str, span: Span) -> Result<Option<TokenStream>> {
  let Some(hex) = s.strip_prefix('#') else {
    return Ok(None);
  };

  let expanded = match hex.len() {
    3 => {
      // #rgb → #rrggbb by doubling each digit
      let mut out = String::with_capacity(6);
      for c in hex.chars() {
        if !c.is_ascii_hexdigit() {
          return Err(syn::Error::new(
            span,
            format!("invalid color `{}`: expected hex digit, got `{}`", s, c),
          ));
        }
        out.push(c);
        out.push(c);
      }
      out
    }
    6 => {
      if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(syn::Error::new(
          span,
          format!("invalid color `{}`: contains non-hex character", s),
        ));
      }
      hex.to_string()
    }
    _ => {
      return Err(syn::Error::new(
        span,
        format!(
          "invalid color `{}`: expected `#rgb` or `#rrggbb`, got {} hex \
           digits",
          s,
          hex.len()
        ),
      ));
    }
  };

  // Parse as u32 at expand time so we emit a clean integer literal.
  let rgb = u32::from_str_radix(&expanded, 16).map_err(|e| {
    syn::Error::new(span, format!("invalid color `{}`: {}", s, e))
  })?;

  let lit = syn::LitInt::new(&format!("0x{:06x}u32", rgb), span);
  Ok(Some(quote_spanned! { span =>
    ::manteau::prelude::Color::hex(#lit)
  }))
}

/// Recognizes unit-suffixed numeric literals: `20px`, `1.5em`, `50%`, etc.
/// Returns `None` if the string doesn't match a numeric-with-unit pattern.
fn try_parse_unit(s: &str, span: Span) -> Result<Option<TokenStream>> {
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
    "px" => emit_unit_int(num_part, "Pixels", span)?,
    "em" => emit_unit_float(num_part, "Em", span)?,
    "rem" => emit_unit_float(num_part, "Rem", span)?,
    "%" => emit_percentage(num_part, span)?,
    _ => return Ok(None),
  };

  Ok(Some(ctor))
}

/// Recognizes URL strings with known schemes. Validates syntactic
/// correctness at expansion time via `url::Url::parse` so the
/// `Url::try_parse(...).expect(...)` we emit is statically safe.
///
/// Returns `None` if the string doesn't start with a recognized scheme
/// prefix. Returns `Err` if it does but doesn't parse.
fn try_parse_url(s: &str, span: Span) -> Result<Option<TokenStream>> {
  const SCHEME_PREFIXES: &[&str] =
    &["http://", "https://", "mailto:", "tel:", "data:"];

  if !SCHEME_PREFIXES.iter().any(|p| s.starts_with(p)) {
    return Ok(None);
  }

  if let Err(e) = url::Url::parse(s) {
    return Err(syn::Error::new(
      span,
      format!("invalid URL `{}`: {}", s, e),
    ));
  }

  let lit = syn::LitStr::new(s, span);
  Ok(Some(quote_spanned! { span =>
    ::manteau::prelude::Url::try_parse(#lit)
      .expect("manteau::mjml!: URL validated at expansion time")
  }))
}

fn emit_unit_int(
  num: &str,
  type_name: &str,
  span: Span,
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
    ::manteau::prelude::#ty::new(#lit)
  })
}

fn emit_unit_float(
  num: &str,
  type_name: &str,
  span: Span,
) -> Result<TokenStream> {
  let parsed: f32 = num.parse().map_err(|_| {
    syn::Error::new(
      span,
      format!("expected number for `{}` value, got `{}`", type_name, num),
    )
  })?;
  let lit = syn::LitFloat::new(&format!("{}f32", parsed), span);
  let ty = syn::Ident::new(type_name, span);
  Ok(quote_spanned! { span =>
    ::manteau::prelude::#ty::new(#lit)
  })
}

/// Percentages get their own emission path because `Percentage::new` takes
/// a `u8` (not `u32`) and is fallible (returns `Result<Self, _>`). We
/// validate the range at expansion time, then emit a `.expect(...)` that
/// is statically unreachable.
fn emit_percentage(num: &str, span: Span) -> Result<TokenStream> {
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
    ::manteau::prelude::Percentage::new(#lit)
      .expect("manteau::mjml!: percentage validated at expansion time")
  })
}
