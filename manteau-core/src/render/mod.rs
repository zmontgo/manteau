//! Core-owned MJML generation, rendering contracts, and validated bodies.

mod error;
mod name;
mod pipeline;
mod port;
pub mod writer;

pub use error::{RenderError, RenderErrorKind};
pub use name::{AttributeName, ElementName, InvalidMarkupName};
pub use pipeline::{Rendered, RenderedBodyError};
pub use port::{HtmlBody, InvalidHtmlBody, InvalidPlaintextBody, MjmlDocument, PlaintextBody, Renderer};
pub use writer::{ElementWriter, MjmlWriter};
