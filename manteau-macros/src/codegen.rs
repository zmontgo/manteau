//! Codegen for the parsed `mjml!` AST.
//!
//! Walks the tree and emits a single block expression that constructs the
//! corresponding builder. The caller's dependency name is resolved at
//! expansion time, so renamed dependencies work too.

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::{ast::*, values};

/// Top-level entry. The root is always a single element.
pub fn generate(root: &Node, facade: &TokenStream) -> TokenStream {
  match root {
    Node::Element(el) => gen_element(el, facade),
    _ => syn::Error::new(Span::call_site(), "mjml! root must be an element")
      .to_compile_error(),
  }
}

/// Generate code for one element. Returns an expression that evaluates to
/// the element's typed value.
fn gen_element(el: &Element, facade: &TokenStream) -> TokenStream {
  let ty_ident = Ident::new(el.kind.type_name(), el.tag_span);
  let ty_path = quote_spanned! { el.tag_span => #facade::prelude::#ty_ident };

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
      let attr_val = match gen_attr_value(attr, el.kind, facade) {
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
      let val = match gen_attr_value(attr, el.kind, facade) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
      };
      quote_spanned! { el.tag_span => #ty_path::new(#val) }
    }
    _ => quote_spanned! { el.tag_span => #ty_path::new() },
  };

  // Chain setter calls.
  let setter_calls = setter_attrs.iter().map(|attr| {
    match gen_setter_call(attr, el.kind, facade) {
      Ok(ts) => ts,
      Err(e) => e.to_compile_error(),
    }
  });

  // Build children. Text-bodied elements have already consumed their body.
  let body_stmts: Vec<TokenStream> = match &el.body {
    ElementBody::Container(nodes) => nodes
      .iter()
      .map(|node| gen_container_node(node, facade))
      .collect(),
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
/// with required `src`), the required attribute is pulled out so codegen can
/// pass it as a constructor argument. Everything else becomes a setter
/// chain after construction.
fn split_required(el: &Element) -> (Option<&Attr>, Vec<&Attr>) {
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
fn gen_setter_call(
  attr: &Attr,
  kind: TagKind,
  facade: &TokenStream,
) -> syn::Result<TokenStream> {
  let method = Ident::new(&kebab_to_snake(&attr.name), attr.name_span);
  let val = gen_attr_value(attr, kind, facade)?;
  Ok(quote_spanned! { attr.name_span =>
    .#method(#val)
  })
}

fn gen_attr_value(
  attr: &Attr,
  kind: TagKind,
  facade: &TokenStream,
) -> syn::Result<TokenStream> {
  match &attr.value {
    AttrValue::StringLit(lit) if attr.name == "src" => {
      values::parse_image_url(lit, facade)
    }
    AttrValue::StringLit(lit) if attr.name == "font-family" => {
      values::parse_font_family(lit, facade)
    }
    AttrValue::StringLit(lit) if attr.name == "href" => {
      values::parse_url(lit, facade)
    }
    AttrValue::StringLit(lit)
      if attr.name == "color" || attr.name == "background-color" =>
    {
      values::parse_color(lit, facade)
    }
    AttrValue::StringLit(lit) if attr.name == "align" => {
      values::parse_alignment(lit, kind == TagKind::Button, facade)
    }
    AttrValue::StringLit(lit) if attr.name == "font-weight" => {
      values::parse_font_weight(lit, facade)
    }
    AttrValue::StringLit(lit) if attr.name == "text-transform" => {
      values::parse_text_transform(lit, facade)
    }
    AttrValue::StringLit(lit) if attr.name == "line-height" => {
      values::parse_line_height(lit, facade)
    }
    AttrValue::StringLit(lit) => values::parse_value(lit, facade),
    AttrValue::Expr(expr) => Ok(quote_spanned! { attr.name_span => #expr }),
  }
}

fn kebab_to_snake(s: &str) -> String {
  s.replace('-', "_")
}

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
fn gen_container_node(node: &Node, facade: &TokenStream) -> TokenStream {
  gen_container_node_ctx(node, RebindCtx::Let, facade)
}

fn gen_container_node_ctx(
  node: &Node,
  ctx: RebindCtx,
  facade: &TokenStream,
) -> TokenStream {
  match node {
    Node::Element(child) => {
      let child_expr = gen_element(child, facade);
      let span = child.tag_span;
      rebind(ctx, span, quote_spanned! { span =>
        #facade::prelude::Push::push(__el, #child_expr)
      })
    }
    Node::Interp(expr) | Node::Raw(expr) => {
      let span = expr_span(expr);
      rebind(ctx, span, quote_spanned! { span =>
        #facade::prelude::Push::push(__el, #expr)
      })
    }
    Node::If {
      cond,
      then_branch,
      else_branch,
    } => {
      let then_stmts = then_branch
        .iter()
        .map(|n| gen_container_node_ctx(n, RebindCtx::Let, facade));
      let else_stmts: Option<Vec<TokenStream>> =
        else_branch.as_ref().map(|nodes| {
          nodes
            .iter()
            .map(|n| gen_container_node_ctx(n, RebindCtx::Let, facade))
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
        .map(|n| gen_container_node_ctx(n, RebindCtx::Assign, facade));
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
        .map(|n| gen_container_node_ctx(n, RebindCtx::Assign, facade));
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
          .map(|n| gen_container_node_ctx(n, RebindCtx::Let, facade));
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

/// Build the `String` content for a text-bodied element.
///
/// Three shapes, in order of decreasing simplicity:
///
/// 1. Empty body → `String::new()`.
/// 2. Single literal → emit the `&str` directly.
/// 3. Anything else (multiple parts, any interp, any control flow) → emit a
///    block that constructs a `String` by appending each part's contribution in
///    order. Control-flow parts expand to native Rust
///    `if`/`for`/`while`/`match` whose branches push fragments onto the same
///    `__s`.
fn build_text_content(body: &ElementBody, span: Span) -> TokenStream {
  let parts = match body {
    ElementBody::Text(parts) => parts,
    _ => return quote_spanned! { span => ::std::string::String::new() },
  };

  if parts.is_empty() {
    return quote_spanned! { span => ::std::string::String::new() };
  }

  // Fast path: single literal → emit as `&str`. The caller wraps it via
  // `impl Into<String>` on the constructor signature.
  if let [TextPart::Literal(s)] = parts.as_slice() {
    let lit = syn::LitStr::new(s, span);
    return quote_spanned! { span => #lit };
  }

  // Fully qualified formatting avoids unused imports in literal-only branches.
  let part_stmts = parts.iter().map(|p| gen_text_part(p, span));
  quote_spanned! { span =>
    {
      let mut __s = ::std::string::String::new();
      #(#part_stmts)*
      __s
    }
  }
}

/// Emit one statement that appends a `TextPart`'s contribution to `__s`.
/// Recursive across control-flow variants.
fn gen_text_part(part: &TextPart, span: Span) -> TokenStream {
  match part {
    TextPart::Literal(s) => {
      let mut chars = s.chars();
      if let (Some(ch), None) = (chars.next(), chars.next()) {
        let lit = syn::LitChar::new(ch, span);
        quote_spanned! { span => __s.push(#lit); }
      } else {
        let lit = syn::LitStr::new(s, span);
        quote_spanned! { span => __s.push_str(#lit); }
      }
    }
    TextPart::Interp(expr) => {
      let espan = expr_span(expr);
      quote_spanned! { espan =>
        ::std::fmt::Write::write_fmt(&mut __s, ::std::format_args!("{}", #expr))
          .expect("formatting into a String is infallible");
      }
    }
    TextPart::If {
      cond,
      then_branch,
      else_branch,
    } => {
      let then_stmts = then_branch.iter().map(|p| gen_text_part(p, span));
      let else_clause = match else_branch {
        Some(branch) => {
          let else_stmts = branch.iter().map(|p| gen_text_part(p, span));
          quote! {
            else {
              #(#else_stmts)*
            }
          }
        }
        None => quote! {},
      };
      quote! {
        if #cond {
          #(#then_stmts)*
        } #else_clause
      }
    }
    TextPart::For { pat, iter, body } => {
      let body_stmts = body.iter().map(|p| gen_text_part(p, span));
      quote! {
        for #pat in #iter {
          #(#body_stmts)*
        }
      }
    }
    TextPart::While { cond, body } => {
      let body_stmts = body.iter().map(|p| gen_text_part(p, span));
      quote! {
        while #cond {
          #(#body_stmts)*
        }
      }
    }
    TextPart::Match { scrutinee, arms } => {
      let arm_exprs = arms.iter().map(|arm| {
        let pat = &arm.pat;
        let body_stmts = arm.body.iter().map(|p| gen_text_part(p, span));
        let guard = arm.guard.as_ref().map(|g| quote! { if #g });
        quote! {
          #pat #guard => {
            #(#body_stmts)*
          }
        }
      });
      quote! {
        match #scrutinee {
          #(#arm_exprs),*
        }
      }
    }
  }
}

fn expr_span(expr: &syn::Expr) -> Span {
  syn::spanned::Spanned::span(expr)
}
