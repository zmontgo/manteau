//! Manteau — ergonomic, typed APIs for MJML emails with pluggable transports.
//!
//! Compose an MJML email through `::new` constructors and chained setters,
//! render it to HTML and plaintext, and ship it through any [`Transport`]
//! (Mailjet, stdout, mock, or your own). Validation lives on the type
//! boundary — invalid email addresses, malformed colors, out-of-range
//! percentages cannot exist as constructed values.
//!
//! # Example
//!
//! ```
//! # use manteau::{Address, Message, MockTransport, Transport};
//! # use manteau::templating::{Body, Column, Section, Template, Text};
//! # let rt = tokio::runtime::Runtime::new().unwrap();
//! # rt.block_on(async {
//! let template = Template::new(Body::new().push_section(
//!   Section::new().push_column(Column::new().push(Text::new("Hello, world!"))),
//! ));
//!
//! let msg = Message::new(
//!   Address::new("hello@example.com".parse().unwrap()),
//!   vec![Address::new("you@example.com".parse().unwrap())],
//!   "Hi",
//!   template,
//! );
//!
//! let transport = MockTransport::new();
//! transport.send(&msg).await.unwrap();
//! assert_eq!(transport.sent().len(), 1);
//! # });
//! ```
//!
//! # Feature flags
//!
//! - `mailjet` — `MailjetTransport`, posts to Mailjet's v3.1 send API. Implies
//!   `tls-rustls`.
//! - `stdout` — `StdoutTransport`, prints to stdout. For local development.
//! - `tls-rustls` / `tls-native` — selects the TLS backend for the HTTP
//!   transports. `tls-rustls` is the default chosen by `mailjet`.

pub mod message;
pub mod models;
pub mod prelude;
pub mod render;
pub mod templating;
pub mod transport;
pub mod transports;

pub use message::Message;
pub use models::{Address, EmailAddress, EmailAddressError, MessageId};
pub use render::{RenderError, RenderErrorKind, Rendered};
pub use transport::{Receipt, Transport, TransportFailure};
#[cfg(feature = "mailjet")]
pub use transports::mailjet::{
  MailjetConfig, MailjetError, MailjetErrorKind, MailjetReceipt,
  MailjetTransport,
};
pub use transports::mock::{MockReceipt, MockTransport};
#[cfg(feature = "stdout")]
pub use transports::stdout::{StdoutReceipt, StdoutTransport};
