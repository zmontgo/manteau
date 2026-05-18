//! Model layer for MJML email templates.
//!
//! Each known MJML element is a typed-builder struct with strongly-typed
//! attribute fields. Rendering lives in [`crate::render`]; this module
//! defines the data only.

pub mod attributes;
pub mod block;
pub mod body;
pub mod button;
pub mod column;
pub mod element;
pub mod image;
pub mod section;
pub mod template;
pub mod text;
pub mod wrapper;

pub use block::Block;
pub use body::{Body, BodyChild};
pub use button::Button;
pub use column::Column;
pub use element::Element;
pub use image::Image;
pub use section::Section;
pub use template::Template;
pub use text::Text;
pub use wrapper::Wrapper;
