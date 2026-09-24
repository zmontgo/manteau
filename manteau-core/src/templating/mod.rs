//! Model layer for MJML email templates.
//!
//! Each known MJML element is a typed-builder struct with strongly-typed
//! attribute fields. Rendering lives in [`crate::render`]; this module
//! defines the data only.

pub mod attributes;
/// Column content: text, buttons, and images.
pub mod block;
/// The top-level email body and its section or wrapper children.
pub mod body;
pub mod button;
/// A section column that holds visible content blocks.
pub mod column;
pub mod element;
pub mod image;
pub mod push;
/// A horizontal group of columns.
pub mod section;
/// A reusable MJML template and optional presentation metadata.
pub mod template;
/// Styled textual email content.
pub mod text;
/// A group of sections with shared visual styling.
pub mod wrapper;

pub use block::Block;
pub use body::{Body, BodyChild};
pub use button::Button;
pub use column::Column;
pub use element::Element;
pub use image::Image;
pub use push::Push;
pub use section::Section;
pub use template::Template;
pub use text::Text;
pub use wrapper::Wrapper;
