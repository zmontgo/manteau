//! Common imports for using manteau's typed MJML DSL.
//!
//! Glob-import this to get every first-class element type, child enum, the
//! [`Push`] trait that unifies parent/child appends across containers,
//! and every attribute value type. The `mjml!` macro re-export will be
//! added here when the macro lands.
//!
//! ```
//! use manteau::prelude::*;
//!
//! let _ = Body::new().push(
//!   Section::new().push(Column::new().push(Text::new("Hi"))),
//! );
//! ```
//!
//! [`Push`]: crate::templating::push::Push

pub use crate::models::{Address, EmailAddress, MessageId};
pub use crate::templating::attributes::prelude::*;
pub use crate::templating::{
  Block, Body, BodyChild, Button, Column, Element, Image, Push, Section,
  Template, Text, Wrapper,
};
