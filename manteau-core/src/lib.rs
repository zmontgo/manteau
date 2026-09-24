//! Typed email values, rendering, and adapter contracts.
//! Consumers normally import these types through `manteau`.
pub mod models;
pub mod message;
pub mod transport;
pub mod idempotency;
pub mod render;
pub mod templating;
pub mod prelude;
pub use models::*;
pub use message::*;
pub use transport::*;
pub use idempotency::*;
pub use render::{Rendered, RenderError, RenderErrorKind};
