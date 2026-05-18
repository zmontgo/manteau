//! Procedural macros for the `manteau` crate. Not intended for direct use:
//! depend on `manteau` and import its macros from `manteau::prelude` or
//! through `manteau::mjml` directly. The shape here mirrors that
//! re-export, so anything published from this crate eventually surfaces
//! under `manteau::*`.

use proc_macro::TokenStream;
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
/// `::manteau::prelude::*`, so the caller doesn't need any specific
/// imports for the expansion to compile — but the prelude is the
/// conventional way to make the surrounding code readable.
#[proc_macro]
pub fn mjml(input: TokenStream) -> TokenStream {
  let node = parse_macro_input!(input as ast::Node);
  codegen::generate(&node).into()
}
