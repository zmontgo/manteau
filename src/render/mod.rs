//! Rendering adapter: walks the [`crate::templating`] model and emits MJML,
//! then HTML, then plaintext.

pub mod error;
pub mod writer;

pub use error::{RenderError, RenderErrorKind};
pub use writer::{ElementWriter, MjmlWriter};
