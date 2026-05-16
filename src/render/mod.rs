//! Rendering adapter: walks the [`crate::templating`] model and emits MJML,
//! then HTML, then plaintext.

pub mod writer;

pub use writer::{ElementWriter, MjmlWriter};
