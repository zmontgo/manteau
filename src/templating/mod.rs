//! Model layer for MJML email templates.
//!
//! Each known MJML element is a typed-builder struct with strongly-typed
//! attribute fields. Rendering lives in [`crate::render`]; this module
//! defines the data only.

pub mod attributes;
pub mod element;

pub use element::Element;
