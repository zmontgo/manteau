//! Common imports for using manteau's typed MJML DSL.
//!
//! Glob-import this to get every first-class element type, child enum,
//! attribute value type, and (when added in a later PR) the `mjml!` macro
//! and the [`Push`] trait that unifies the parent/child append API across
//! containers.
//!
//! ```
//! use manteau::prelude::*;
//!
//! let _ = Body::new().push_section(
//!   Section::new().push_column(Column::new().push(Text::new("Hi"))),
//! );
//! ```
//!
//! [`Push`]: crate::templating::push::Push
// The Push trait re-export is added when item F lands.
// The mjml! macro re-export is added when the macro lands.

pub use crate::templating::attributes::prelude::*;
pub use crate::templating::{
  Block, Body, BodyChild, Button, Column, Element, Image, Section, Template,
  Text, Wrapper,
};
