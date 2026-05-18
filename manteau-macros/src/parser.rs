//! Parser for the `mjml!` macro body.
//!
//! Implements `syn::parse::Parse` for the AST root. The grammar is roughly:
//!
//! ```text
//! node     := element | interp | raw | if | for | while | match
//! element  := '<' ident attr* '>' body '</' ident '>'
//!           | '<' ident attr* '/>'
//! attr     := ident ('-' ident)* '=' (string-lit | brace-expr)
//! body     := node* | text-part*       (depends on tag)
//! interp   := '{' expr '}'
//! raw      := '{' 'raw' ':' expr '}'
//! if       := '@' 'if' expr '{' node* '}' ('@' 'else' '{' node* '}')?
//! for      := '@' 'for' pat 'in' expr '{' node* '}'
//! while    := '@' 'while' (expr | 'let' pat '=' expr) '{' node* '}'
//! match    := '@' 'match' expr '{' (pat ('if' expr)? '=>' '{' node* '}' ','?)* '}'
//! ```

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Pat, Result, Token, braced, token};

use crate::ast::*;

impl Parse for Node {
  fn parse(input: ParseStream) -> Result<Self> {
    // The root must be a single element.
    let el = parse_element(input)?;
    if !input.is_empty() {
      return Err(
        input.error("expected end of macro input after root element"),
      );
    }
    Ok(Node::Element(el))
  }
}

/// Parse one element, starting at `<`.
fn parse_element(input: ParseStream) -> Result<Element> {
  let _lt: Token![<] = input.parse()?;
  let tag_ident: Ident = input.parse()?;
  let tag_span = tag_ident.span();

  let kind = TagKind::from_ident(&tag_ident).ok_or_else(|| {
    syn::Error::new(
      tag_span,
      format!(
        "`{}` is not a known mjml element; expected one of: Body, Wrapper, \
         Section, Column, Text, Button, Image",
        tag_ident
      ),
    )
  })?;

  // Parse attributes until we see `>` or `/>`.
  let mut attrs = Vec::new();
  let self_closing;
  loop {
    if input.peek(Token![/]) {
      let _: Token![/] = input.parse()?;
      let _: Token![>] = input.parse()?;
      self_closing = true;
      break;
    }
    if input.peek(Token![>]) {
      let _: Token![>] = input.parse()?;
      self_closing = false;
      break;
    }
    attrs.push(parse_attr(input)?);
  }

  // Validate required attributes are present.
  if let Some(required) = kind.required_attr()
    && !attrs.iter().any(|a| a.name == required)
  {
    return Err(syn::Error::new(
      tag_span,
      format!(
        "`<{}>` requires the `{}` attribute",
        kind.type_name(),
        required
      ),
    ));
  }

  let body = if self_closing {
    if kind.body_kind() == BodyKind::Text {
      // Allow self-closed text elements as empty content.
      ElementBody::Text(Vec::new())
    } else {
      ElementBody::Empty
    }
  } else {
    let body = match kind.body_kind() {
      BodyKind::Container => {
        ElementBody::Container(parse_container_body(input, kind)?)
      }
      BodyKind::Text => ElementBody::Text(parse_text_body(input, kind)?),
      BodyKind::Empty => {
        // Empty-bodied elements must self-close; if we got here with an
        // open tag, require an immediate close.
        if !peek_closing(input) {
          return Err(syn::Error::new(
            tag_span,
            format!(
              "`<{}>` cannot have body content; use `<{} />`",
              kind.type_name(),
              kind.type_name()
            ),
          ));
        }
        ElementBody::Empty
      }
    };
    consume_closing_tag(input, kind, tag_span)?;
    body
  };

  Ok(Element {
    kind,
    tag_span,
    attrs,
    body,
  })
}

/// Parse one attribute. Form: `name="value"` or `name={expr}`. The name may
/// contain hyphens (which `syn` does not treat as part of an ident, so we
/// reassemble them).
fn parse_attr(input: ParseStream) -> Result<Attr> {
  let first: Ident = input.parse()?;
  let name_span = first.span();
  let mut name = first.to_string();

  while input.peek(Token![-]) {
    let _: Token![-] = input.parse()?;
    let part: Ident = input.parse()?;
    name.push('-');
    name.push_str(&part.to_string());
  }

  let _: Token![=] = input.parse()?;

  let value = if input.peek(LitStr) {
    AttrValue::StringLit(input.parse()?)
  } else if input.peek(token::Brace) {
    let content;
    braced!(content in input);
    let expr: Expr = content.parse()?;
    AttrValue::Expr(expr)
  } else {
    return Err(input.error(
      "expected string literal or `{expression}` for attribute value",
    ));
  };

  Ok(Attr {
    name,
    name_span,
    value,
  })
}

/// Parse the body of a container element until we hit `</`.
fn parse_container_body(
  input: ParseStream,
  _parent: TagKind,
) -> Result<Vec<Node>> {
  let mut nodes = Vec::new();
  while !peek_closing(input) {
    if input.is_empty() {
      return Err(input.error("unexpected end of input inside element body"));
    }
    nodes.push(parse_container_node(input)?);
  }
  Ok(nodes)
}

fn parse_container_node(input: ParseStream) -> Result<Node> {
  if input.peek(Token![@]) {
    return parse_control_flow(input);
  }
  if input.peek(Token![<]) {
    let el = parse_element(input)?;
    return Ok(Node::Element(el));
  }
  if input.peek(token::Brace) {
    return parse_brace_node(input);
  }

  // Bare text is not allowed in container-bodied elements.
  Err(input.error(
    "expected child element, `{expr}` interpolation, or `@if`/`@for` in \
     container body; bare text is only allowed inside text-bodied elements \
     like `<Text>`",
  ))
}

fn parse_brace_node(input: ParseStream) -> Result<Node> {
  let content;
  braced!(content in input);

  // Check for `raw: expr` form.
  if content.peek(Ident) && content.peek2(Token![:]) {
    let kw: Ident = content.fork().parse()?;
    if kw == "raw" {
      let _: Ident = content.parse()?;
      let _: Token![:] = content.parse()?;
      let expr: Expr = content.parse()?;
      return Ok(Node::Raw(expr));
    }
  }

  let expr: Expr = content.parse()?;
  Ok(Node::Interp(expr))
}

fn parse_control_flow(input: ParseStream) -> Result<Node> {
  let at: Token![@] = input.parse()?;
  if input.peek(Token![if]) {
    let _: Token![if] = input.parse()?;
    return parse_if(input);
  }
  if input.peek(Token![for]) {
    let _: Token![for] = input.parse()?;
    return parse_for(input);
  }
  if input.peek(Token![while]) {
    let _: Token![while] = input.parse()?;
    return parse_while(input);
  }
  if input.peek(Token![match]) {
    let _: Token![match] = input.parse()?;
    return parse_match(input);
  }
  Err(syn::Error::new(
    at.span,
    "expected `@if`, `@for`, `@while`, or `@match` after `@`",
  ))
}

fn parse_if(input: ParseStream) -> Result<Node> {
  let cond: Expr = Expr::parse_without_eager_brace(input)?;
  let then_branch = parse_braced_body(input)?;
  let else_branch = if input.peek(Token![@]) {
    // Peek for `@else` without committing.
    let fork = input.fork();
    let _: Token![@] = fork.parse()?;
    if fork.peek(Token![else]) {
      let _: Token![@] = input.parse()?;
      let _: Token![else] = input.parse()?;
      Some(parse_braced_body(input)?)
    } else {
      None
    }
  } else {
    None
  };
  Ok(Node::If {
    cond,
    then_branch,
    else_branch,
  })
}

fn parse_for(input: ParseStream) -> Result<Node> {
  let pat = Pat::parse_single(input)?;
  let _: Token![in] = input.parse()?;
  let iter: Expr = Expr::parse_without_eager_brace(input)?;
  let body = parse_braced_body(input)?;
  Ok(Node::For { pat, iter, body })
}

/// `@while cond { ... }` and `@while let pat = expr { ... }`. The guard is
/// parsed as an expression without eagerly consuming the braced body that
/// follows.
fn parse_while(input: ParseStream) -> Result<Node> {
  let cond: Expr = if input.peek(Token![let]) {
    // `while let pat = expr { ... }` — build an ExprLet for the guard.
    let let_token: Token![let] = input.parse()?;
    let pat = Pat::parse_multi_with_leading_vert(input)?;
    let eq_token: Token![=] = input.parse()?;
    let scrut: Expr = Expr::parse_without_eager_brace(input)?;
    Expr::Let(syn::ExprLet {
      attrs: Vec::new(),
      let_token,
      pat: Box::new(pat),
      eq_token,
      expr: Box::new(scrut),
    })
  } else {
    Expr::parse_without_eager_brace(input)?
  };
  let body = parse_braced_body(input)?;
  Ok(Node::While { cond, body })
}

/// `@match expr { pat [if guard] => { ... }, pat => { ... }, ... }`
///
/// Each arm's body is required to be a braced block of nodes. Commas
/// between arms are optional (consistent with Rust `match` when arm bodies
/// are blocks).
fn parse_match(input: ParseStream) -> Result<Node> {
  let scrutinee: Expr = Expr::parse_without_eager_brace(input)?;
  let content;
  braced!(content in input);

  let mut arms = Vec::new();
  while !content.is_empty() {
    let pat = Pat::parse_multi_with_leading_vert(&content)?;
    let guard = if content.peek(Token![if]) {
      let _: Token![if] = content.parse()?;
      let g: Expr = Expr::parse_without_eager_brace(&content)?;
      Some(g)
    } else {
      None
    };
    let _: Token![=>] = content.parse()?;
    let body = parse_braced_body(&content)?;
    // Optional trailing comma between arms.
    let _ = content.parse::<Token![,]>().ok();
    arms.push(MatchArm { pat, guard, body });
  }

  Ok(Node::Match { scrutinee, arms })
}

fn parse_braced_body(input: ParseStream) -> Result<Vec<Node>> {
  let content;
  braced!(content in input);
  let mut nodes = Vec::new();
  while !content.is_empty() {
    nodes.push(parse_container_node(&content)?);
  }
  Ok(nodes)
}

/// Parse the body of a text-bodied element. Following Maud's design, the
/// only valid body content is:
/// - String literals (`"..."`)
/// - `{expr}` splices
/// - Closing tag `</Tag>` ends the body
///
/// There is no "bare text" — every literal piece of text must be quoted.
/// This eliminates whitespace ambiguity entirely: the user's quoted
/// strings carry exactly the characters they want, no more, no less.
fn parse_text_body(
  input: ParseStream,
  _parent: TagKind,
) -> Result<Vec<TextPart>> {
  let mut parts = Vec::new();

  while !peek_closing(input) {
    if input.is_empty() {
      return Err(input.error("unexpected end of input inside text body"));
    }
    if input.peek(Token![<]) {
      return Err(input.error(
        "nested elements are not allowed inside text-bodied elements \
         (`<Text>`, `<Button>`)",
      ));
    }
    if input.peek(token::Brace) {
      let content;
      braced!(content in input);
      let expr: Expr = content.parse()?;
      parts.push(TextPart::Interp(expr));
      continue;
    }
    if input.peek(LitStr) {
      let lit: LitStr = input.parse()?;
      parts.push(TextPart::Literal(lit.value()));
      continue;
    }
    return Err(input.error(
      "expected string literal `\"...\"` or `{expr}` splice in text body; \
       bare text is not supported — wrap all literal text in double quotes",
    ));
  }

  Ok(parts)
}

fn peek_closing(input: ParseStream) -> bool {
  input.peek(Token![<]) && input.peek2(Token![/])
}

fn consume_closing_tag(
  input: ParseStream,
  kind: TagKind,
  open_span: Span,
) -> Result<()> {
  let _: Token![<] = input.parse()?;
  let _: Token![/] = input.parse()?;
  let close: Ident = input.parse()?;
  if close != kind.type_name() {
    return Err(syn::Error::new(
      close.span(),
      format!(
        "mismatched closing tag: expected `</{}>` (opened at this span: \
         {:?}), got `</{}>`",
        kind.type_name(),
        open_span,
        close
      ),
    ));
  }
  let _: Token![>] = input.parse()?;
  Ok(())
}
