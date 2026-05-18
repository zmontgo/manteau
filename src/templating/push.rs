//! Unified parent/child append via the [`Push`] trait.
//!
//! Containers in this crate (`Body`, `Wrapper`, `Section`, `Column`) accept
//! children through a single trait method, [`Push::push`], that consumes
//! `self` and returns `Self`. Different parents accept different child
//! types — `Body` takes a `Section` or `Wrapper`, `Section` takes a
//! `Column`, `Column` takes anything that implements `Into<Block>` (so
//! `Text`, `Button`, `Image`, or a `Block::Custom`-wrapped consumer
//! element). The trait dispatches on the argument type at the call site,
//! so the same method name flows everywhere — which is exactly what the
//! `mjml!` macro needs to emit a uniform `.push(...)` regardless of the
//! container.
//!
//! ```
//! use manteau::prelude::*;
//!
//! let body = Body::new()
//!   .push(Section::new()
//!     .push(Column::new()
//!       .push(Text::new("Hello"))));
//! # let _ = body;
//! ```

use crate::templating::{
  block::Block, body::Body, body::BodyChild, column::Column, section::Section,
  wrapper::Wrapper,
};

/// Append a single child to `Self`, consuming and returning the modified
/// container. Implementors decide what child types they accept.
pub trait Push<Child> {
  fn push(self, child: Child) -> Self;
}

// ─── Body accepts Section, Wrapper, BodyChild ────────────────────────────

impl Push<Section> for Body {
  fn push(mut self, child: Section) -> Self {
    self.children.push(BodyChild::Section(child));
    self
  }
}

impl Push<Wrapper> for Body {
  fn push(mut self, child: Wrapper) -> Self {
    self.children.push(BodyChild::Wrapper(child));
    self
  }
}

impl Push<BodyChild> for Body {
  fn push(mut self, child: BodyChild) -> Self {
    self.children.push(child);
    self
  }
}

// ─── Wrapper accepts Section ─────────────────────────────────────────────

impl Push<Section> for Wrapper {
  fn push(mut self, child: Section) -> Self {
    self.sections.push(child);
    self
  }
}

// ─── Section accepts Column ──────────────────────────────────────────────

impl Push<Column> for Section {
  fn push(mut self, child: Column) -> Self {
    self.columns.push(child);
    self
  }
}

// ─── Column accepts anything Into<Block> ─────────────────────────────────
//
// Blanket impl over `Into<Block>` covers `Text`, `Button`, `Image` (each
// has a `From<X> for Block` impl), and `Block` itself (via the standard
// library's `From<T> for T`). Consumer-defined elements participate by
// wrapping themselves into a `Block::Custom` and pushing that.

impl<T: Into<Block>> Push<T> for Column {
  fn push(mut self, child: T) -> Self {
    self.children.push(child.into());
    self
  }
}
