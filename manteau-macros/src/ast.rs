//! AST for parsed `mjml!` content.
//!
//! The parser builds these; codegen walks them. Keeping them flat and
//! cheap-to-construct.

use proc_macro2::Span;
use syn::{Expr, Ident, LitStr};

/// The first-class element tags the macro recognizes. Anything outside this
/// set is a parse error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
  Body,
  Wrapper,
  Section,
  Column,
  Text,
  Button,
  Image,
}

impl TagKind {
  pub fn from_ident(ident: &Ident) -> Option<Self> {
    match ident.to_string().as_str() {
      "Body" => Some(Self::Body),
      "Wrapper" => Some(Self::Wrapper),
      "Section" => Some(Self::Section),
      "Column" => Some(Self::Column),
      "Text" => Some(Self::Text),
      "Button" => Some(Self::Button),
      "Image" => Some(Self::Image),
      _ => None,
    }
  }

  /// What kind of body content is allowed inside this tag.
  pub fn body_kind(self) -> BodyKind {
    match self {
      Self::Body | Self::Wrapper | Self::Section | Self::Column => {
        BodyKind::Container
      }
      Self::Text | Self::Button => BodyKind::Text,
      Self::Image => BodyKind::Empty,
    }
  }

  /// The Rust type name (used unchanged; the macro emits `::manteau::Foo`).
  pub fn type_name(self) -> &'static str {
    match self {
      Self::Body => "Body",
      Self::Wrapper => "Wrapper",
      Self::Section => "Section",
      Self::Column => "Column",
      Self::Text => "Text",
      Self::Button => "Button",
      Self::Image => "Image",
    }
  }

  /// Required attribute that must appear in the open tag and is passed to
  /// the constructor. None means `::new()` takes no arguments (for
  /// container-bodied elements) or takes the body content (text-bodied
  /// elements without a required URL attribute).
  pub fn required_attr(self) -> Option<&'static str> {
    match self {
      Self::Button => Some("href"),
      Self::Image => Some("src"),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
  /// Body is a list of child nodes pushed via the `Push` trait.
  Container,
  /// Body is text content composed into a single string and passed to
  /// `::new(content)` (or `::new(content, required_attr)` for Button).
  Text,
  /// Self-closing only; no body allowed.
  Empty,
}

/// One attribute on an element open tag, e.g. `font-size="20px"`.
#[derive(Debug, Clone)]
pub struct Attr {
  /// The original kebab-case name as written in source.
  pub name:      String,
  /// Span of the name token, for error reporting.
  pub name_span: Span,
  /// The attribute's value. Always a string literal or braced expression
  /// in source syntax; the `values` module turns string literals into
  /// typed expressions where it can recognize the contents.
  pub value:     AttrValue,
}

#[derive(Debug, Clone)]
pub enum AttrValue {
  /// A string literal `"..."` — the macro inspects its contents to decide
  /// whether to emit a typed constructor (for units, colors, URLs) or pass
  /// through as a string.
  StringLit(LitStr),
  /// A braced Rust expression `{expr}`. Passed through unchanged.
  Expr(Expr),
}

/// A node in the body of an element. Container-bodied elements have a
/// `Vec<Node>`; text-bodied elements have a `Vec<TextPart>`.
#[derive(Debug, Clone)]
pub enum Node {
  Element(Element),
  /// `{expr}` interpolation in container body — must be `Into<Block>`.
  Interp(Expr),
  /// `{raw: expr}` — pre-built `Block` or `Into<Block>` value. Codegen
  /// currently treats this identically to `Interp`; the variant remains
  /// as a grammar hook for a future semantic distinction.
  Raw(Expr),
  /// `@if cond { ... } [@else { ... }]`
  If {
    cond:        Expr,
    then_branch: Vec<Node>,
    else_branch: Option<Vec<Node>>,
  },
  /// `@for pat in iter { ... }`
  For {
    pat:  syn::Pat,
    iter: Expr,
    body: Vec<Node>,
  },
  /// `@while cond { ... }` — also handles `@while let pat = expr { ... }`
  /// since `Expr::While` would consume the braced body; we parse the
  /// guard with `Expr::parse_without_eager_brace` instead.
  While { cond: Expr, body: Vec<Node> },
  /// `@match expr { pat => { ... }, ... }`
  Match { scrutinee: Expr, arms: Vec<MatchArm> },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
  pub pat:   syn::Pat,
  pub guard: Option<Expr>,
  pub body:  Vec<Node>,
}

/// One part of text-bodied element content. Concatenated via `format!`.
#[derive(Debug, Clone)]
pub enum TextPart {
  /// A literal run of text.
  Literal(String),
  /// `{expr}` interpolation — must be `Display`.
  Interp(Expr),
}

#[derive(Debug, Clone)]
pub struct Element {
  pub kind:     TagKind,
  /// Span of the opening tag identifier, for error reporting.
  pub tag_span: Span,
  pub attrs:    Vec<Attr>,
  pub body:     ElementBody,
}

#[derive(Debug, Clone)]
pub enum ElementBody {
  /// Container-bodied element: a list of child nodes.
  Container(Vec<Node>),
  /// Text-bodied element: a sequence of literal/interp parts.
  Text(Vec<TextPart>),
  /// Self-closing element (or empty-bodied with `<Foo/>`).
  Empty,
}
