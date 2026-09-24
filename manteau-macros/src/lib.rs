//! Procedural macros for the `manteau` crate. Not intended for direct use:
//! depend on `manteau` and import its macros from `manteau::prelude` or
//! through `manteau::mjml` directly. The shape here mirrors that
//! re-export, so anything published from this crate eventually surfaces
//! under `manteau::*`.
#![deny(missing_docs)]

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;

mod ast;
mod codegen;
mod parser;
mod values;

/// `mjml!` — typed MJML template DSL.
///
/// Builds a manteau element tree using JSX-style syntax: `<Body>` /
/// `<Section>` / `<Column>` open and close in pairs, attributes become
/// builder calls, and `{expr}` interpolates Rust expressions as children
/// or attribute values. Control flow (`@if`, `@for`, `@while`, `@match`)
/// is supported inside container bodies for runtime-conditional content.
///
/// See `manteau::prelude` for the brought-into-scope element and
/// attribute types. The macro emits fully-qualified paths through
/// the consumer's dependency name, so the caller doesn't need any specific
/// imports for the expansion to compile — but the prelude is the
/// conventional way to make the surrounding code readable.
#[proc_macro]
pub fn mjml(input: TokenStream) -> TokenStream {
  let node = parse_macro_input!(input as ast::Node);
  let facade = match crate_name("manteau") {
    Ok(FoundCrate::Itself) => quote!(::manteau),
    Ok(FoundCrate::Name(name)) => {
      let ident = syn::Ident::new(&name, Span::call_site());
      quote!(::#ident)
    }
    Err(_) => {
      return syn::Error::new(
        Span::call_site(),
        "mjml! requires a dependency on the manteau crate",
      )
      .to_compile_error()
      .into();
    }
  };

  codegen::generate(&node, &facade).into()
}
