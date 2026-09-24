//! Rendering adapter: walks the [`crate::templating`] model and emits MJML,
//! then HTML, then plaintext.

pub mod error;
pub mod pipeline;
pub mod writer;

pub use error::{RenderError, RenderErrorKind};
pub use pipeline::{Rendered, render_html, render_plaintext};
pub use writer::{ElementWriter, MjmlWriter};
