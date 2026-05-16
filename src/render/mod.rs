//! Rendering adapter: walks the [`crate::templating`] model and emits MJML,
//! then HTML, then plaintext.

pub mod error;
#[allow(clippy::module_inception)]
pub mod render;
pub mod writer;

pub use error::{RenderError, RenderErrorKind};
pub use render::{render_html, render_plaintext, Rendered};
pub use writer::{ElementWriter, MjmlWriter};
