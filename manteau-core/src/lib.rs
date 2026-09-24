//! Typed email values, rendering, and adapter contracts.
//! Consumers normally import these types through `manteau`.
pub mod idempotency;
pub mod message;
pub mod models;
pub mod prelude;
pub mod render;
pub mod templating;
pub mod transport;
pub use idempotency::*;
pub use message::*;
pub use models::*;
pub use render::{HtmlBody, InvalidHtmlBody, InvalidPlaintextBody, MjmlDocument, PlaintextBody, RenderError, RenderErrorKind, Rendered, Renderer};
pub use transport::*;

pub mod http;
pub mod sender;
pub use sender::{
  HttpProvider, IdempotentProvider, SendError, Sender, StatusPolicy,
};
