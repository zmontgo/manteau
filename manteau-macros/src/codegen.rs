//! Codegen for the parsed `mjml!` AST.
//!
//! Walks the tree and emits a single block expression that constructs the
//! corresponding manteau builder. All manteau references are
//! fully-qualified through `::manteau::prelude` so the macro doesn't depend
//! on what the caller has in scope.

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::ast::*;
use crate::values;

/// Top-level entry. The root is always a single element.
pub fn generate(root: &Node) -> TokenStream {
  match root {
    Node::Element(el) => gen_element(el),
    _ => syn::Error::new(Span::call_site(), "mjml! root must be an element")
      .to_compile_error(),
  }
}

/// Generate code for one element. Returns an expression that evaluates to
/// the element's typed value.
fn gen_element(el: &Element) -> TokenStream {
  let ty_ident = Ident::new(el.kind.type_name(), el.tag_span);
  let ty_path =
    quote_spanned! { el.tag_span => ::manteau::prelude::#ty_ident };

  // Split attributes into "required constructor arg" and "setters".
  let (required_attr, setter_attrs) = split_required(el);

  // Build the constructor call. Three shapes:
  //
  //   - text-bodied + required attr     → Button::new(content, attr_value)
  //   - text-bodied + no required attr  → Text::new(content)
  //   - any body + required attr        → Image::new(attr_value)
  //   - container/empty + none          → Body::new() (etc.)
  let ctor = match (el.kind.body_kind(), required_attr) {
    (BodyKind::Text, Some(attr)) => {
      let content_expr = build_text_content(&el.body, el.tag_span);
      let attr_val = match gen_attr_value(attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
      };
      quote_spanned! { el.tag_span =>
        #ty_path::new(#content_expr, #attr_val)
      }
    }
    (BodyKind::Text, None) => {
      let content_expr = build_text_content(&el.body, el.tag_span);
      quote_spanned! { el.tag_span => #ty_path::new(#content_expr) }
    }
    (_, Some(attr)) => {
      let val = match gen_attr_value(attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
      };
      quote_spanned! { el.tag_span => #ty_path::new(#val) }
    }
    _ => quote_spanned! { el.tag_span => #ty_path::new() },
  };

  // Chain setter calls.
  let setter_calls =
    setter_attrs.iter().map(|attr| match gen_setter_call(attr) {
      Ok(ts) => ts,
      Err(e) => e.to_compile_error(),
    });

  // Build children. Text-bodied elements have already consumed their body.
  let body_stmts: Vec<TokenStream> = match &el.body {
    ElementBody::Container(nodes) => {
      nodes.iter().map(gen_container_node).collect()
    }
    ElementBody::Text(_) | ElementBody::Empty => Vec::new(),
  };

  if body_stmts.is_empty() {
    quote_spanned! { el.tag_span =>
      #ctor #(#setter_calls)*
    }
  } else {
    quote_spanned! { el.tag_span => {
      let __el = #ctor #(#setter_calls)*;
      #(#body_stmts)*
      __el
    }}
  }
}

/// Split an element's attributes into the constructor-required one (if
/// any) and the rest (which become chained setter calls).
///
/// For `Button` (text-bodied + required `href`) and `Image` (empty-bodied
/// + required `src`), the required attribute is pulled out so codegen can
/// pass it as a constructor argument. Everything else becomes a setter
/// chain after construction.
fn split_required<'a>(el: &'a Element) -> (Option<&'a Attr>, Vec<&'a Attr>) {
  let Some(req_name) = el.kind.required_attr() else {
    return (None, el.attrs.iter().collect());
  };

  let mut required: Option<&Attr> = None;
  let mut rest = Vec::new();
  for a in &el.attrs {
    if a.name == req_name && required.is_none() {
      required = Some(a);
    } else {
      rest.push(a);
    }
  }
  (required, rest)
}

/// Emit the call `.snake_name(value)` for one attribute.
fn gen_setter_call(attr: &Attr) -> syn::Result<TokenStream> {
  let method = Ident::new(&kebab_to_snake(&attr.name), attr.name_span);
  let val = gen_attr_value(attr)?;
  Ok(quote_spanned! { attr.name_span =>
    .#method(#val)
  })
}

fn gen_attr_value(attr: &Attr) -> syn::Result<TokenStream> {
  match &attr.value {
    AttrValue::StringLit(lit) => values::parse_value(lit),
    AttrValue::Expr(expr) => Ok(quote_spanned! { attr.name_span => #expr }),
  }
}

fn kebab_to_snake(s: &str) -> String { s.replace('-', "_") }

/// Whether we're generating statements that introduce a new `__el` binding
/// each time (`let __el = ...`) or that reassign an existing mutable `__el`
/// (`__el = ...`). The former is used at the top of an element body where
/// shadowing is fine; the latter is used inside `for` loops where the
/// binding must persist across iterations.
#[derive(Debug, Clone, Copy)]
enum RebindCtx {
  Let,
  Assign,
}

fn rebind(ctx: RebindCtx, span: Span, value: TokenStream) -> TokenStream {
  match ctx {
    RebindCtx::Let => quote_spanned! { span =>
      let __el = #value;
    },
    RebindCtx::Assign => quote_spanned! { span =>
      __el = #value;
    },
  }
}

/// Generate a rebind statement for one container-body node.
fn gen_container_node(node: &Node) -> TokenStream {
  gen_container_node_ctx(node, RebindCtx::Let)
}

fn gen_container_node_ctx(node: &Node, ctx: RebindCtx) -> TokenStream {
  match node {
    Node::Element(child) => {
      let child_expr = gen_element(child);
      let span = child.tag_span;
      rebind(
        ctx,
        span,
        quote_spanned! { span =>
          ::manteau::prelude::Push::push(__el, #child_expr)
        },
      )
    }
    Node::Interp(expr) | Node::Raw(expr) => {
      let span = expr_span(expr);
      rebind(
        ctx,
        span,
        quote_spanned! { span =>
          ::manteau::prelude::Push::push(__el, #expr)
        },
      )
    }
    Node::If {
      cond,
      then_branch,
      else_branch,
    } => {
      let then_stmts = then_branch
        .iter()
        .map(|n| gen_container_node_ctx(n, RebindCtx::Let));
      let else_stmts: Option<Vec<TokenStream>> =
        else_branch.as_ref().map(|nodes| {
          nodes
            .iter()
            .map(|n| gen_container_node_ctx(n, RebindCtx::Let))
            .collect()
        });
      let else_block = match else_stmts {
        Some(stmts) => quote! {
          else {
            let __el = __el;
            #(#stmts)*
            __el
          }
        },
        None => quote! { else { __el } },
      };
      let if_expr = quote! {
        if #cond {
          let __el = __el;
          #(#then_stmts)*
          __el
        } #else_block
      };
      rebind(ctx, Span::call_site(), if_expr)
    }
    Node::For { pat, iter, body } => {
      let body_stmts = body
        .iter()
        .map(|n| gen_container_node_ctx(n, RebindCtx::Assign));
      let loop_expr = quote! {
        {
          let mut __el = __el;
          for #pat in #iter {
            #(#body_stmts)*
          }
          __el
        }
      };
      rebind(ctx, Span::call_site(), loop_expr)
    }
    Node::While { cond, body } => {
      let body_stmts = body
        .iter()
        .map(|n| gen_container_node_ctx(n, RebindCtx::Assign));
      let loop_expr = quote! {
        {
          let mut __el = __el;
          while #cond {
            #(#body_stmts)*
          }
          __el
        }
      };
      rebind(ctx, Span::call_site(), loop_expr)
    }
    Node::Match { scrutinee, arms } => {
      let arm_exprs = arms.iter().map(|arm| {
        let pat = &arm.pat;
        let stmts = arm
          .body
          .iter()
          .map(|n| gen_container_node_ctx(n, RebindCtx::Let));
        let guard = arm.guard.as_ref().map(|g| quote! { if #g });
        quote! {
          #pat #guard => {
            let __el = __el;
            #(#stmts)*
            __el
          }
        }
      });
      let match_expr = quote! {
        match #scrutinee {
          #(#arm_exprs),*
        }
      };
      rebind(ctx, Span::call_site(), match_expr)
    }
  }
}

/// Build the `String` content for a text-bodied element by concatenating
/// literal pieces and `Display`-formatted interpolations.
fn build_text_content(body: &ElementBody, span: Span) -> TokenStream {
  let parts = match body {
    ElementBody::Text(parts) => parts,
    _ => return quote_spanned! { span => ::std::string::String::new() },
  };

  if parts.is_empty() {
    return quote_spanned! { span => "" };
  }

  // Single literal: emit as a `&str`.
  if let [TextPart::Literal(s)] = parts.as_slice() {
    let lit = syn::LitStr::new(s, span);
    return quote_spanned! { span => #lit };
  }

  // Build a `format!` call. Escape `{` and `}` in literal pieces by
  // doubling them, per `format!` syntax.
  let mut fmt_str = String::new();
  let mut args = Vec::<TokenStream>::new();
  for part in parts {
    match part {
      TextPart::Literal(s) => {
        for c in s.chars() {
          if c == '{' || c == '}' {
            fmt_str.push(c);
            fmt_str.push(c);
          } else {
            fmt_str.push(c);
          }
        }
      }
      TextPart::Interp(expr) => {
        fmt_str.push_str("{}");
        let espan = expr_span(expr);
        args.push(quote_spanned! { espan => #expr });
      }
    }
  }

  let lit = syn::LitStr::new(&fmt_str, span);
  quote_spanned! { span =>
    ::std::format!(#lit, #(#args),*)
  }
}

fn expr_span(expr: &syn::Expr) -> Span {
  syn::spanned::Spanned::span(expr)
}
