//! Core-owned MJML generation, rendering contracts, and validated bodies.

mod error;
mod pipeline;
mod port;
pub mod writer;

pub use error::{RenderError, RenderErrorKind};
pub use pipeline::{EmptyBody, Rendered};
pub use port::Renderer;
pub use writer::{ElementWriter, MjmlWriter};
