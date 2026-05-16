//! Manteau — typed builders for MJML emails with pluggable transports.
//!
//! Compose an MJML email through typed builders, render it to HTML and
//! plaintext, and ship it through any [`Transport`] (Mailjet, stdout, mock,
//! or your own). Validation lives on the type boundary — invalid email
//! addresses, malformed colors, out-of-range percentages cannot exist as
//! constructed values.
//!
//! # Example
//!
//! ```
//! # use manteau::{Address, Message, MockTransport, Transport};
//! # use manteau::templating::{Body, Column, Section, Template, Text};
//! # let rt = tokio::runtime::Runtime::new().unwrap();
//! # rt.block_on(async {
//! let template = Template::builder()
//!     .body(Body::builder()
//!         .sections(vec![Section::builder()
//!             .columns(vec![Column::builder()
//!                 .children(vec![Text::builder()
//!                     .content("Hello, world!")
//!                     .build()
//!                     .into()])
//!                 .build()])
//!             .build()])
//!         .build())
//!     .build();
//!
//! let msg = Message::builder()
//!     .from(Address::builder().email("hello@example.com".parse().unwrap()).build())
//!     .to(vec![Address::builder().email("you@example.com".parse().unwrap()).build()])
//!     .subject("Hi")
//!     .content(template)
//!     .build();
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
  MailjetError, MailjetErrorKind, MailjetReceipt, MailjetTransport,
};
pub use transports::mock::{MockReceipt, MockTransport};
#[cfg(feature = "stdout")]
pub use transports::stdout::{StdoutReceipt, StdoutTransport};
