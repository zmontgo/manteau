//! Manteau — typed builders for MJML emails with pluggable transports.
//!
//! Restructure in progress. Modules are wired in incrementally as they land.

pub mod message;
pub mod models;
pub mod render;
pub mod templating;

pub use message::Message;
pub use models::{Address, EmailAddress, EmailAddressError, MessageId};
pub use render::{RenderError, RenderErrorKind, Rendered};
