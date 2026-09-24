//! Typed email templates and independently selectable transports.
//! See the crate README for installation and the complete sending example.
#![deny(missing_docs)]

extern crate self as manteau;
mod mailer;
pub mod message;

pub use mailer::{Mailer, MailerError};
pub use manteau_core::*;
pub use manteau_macros::mjml;
#[cfg(feature = "render-mrml")]
pub use manteau_render::{InvalidTextWidth, MrmlError, MrmlRenderer};
pub use message::Message;
/// Common template, envelope, and transport imports.
pub mod prelude {
  pub use manteau_core::prelude::*;

  pub use crate::{Mailer, Message, mjml};
}

#[cfg(feature = "cloudflare")]
pub use manteau_cloudflare::{
  Cloudflare, CloudflareError, CloudflareErrorKind, CloudflareReceipt,
};
#[cfg(feature = "jetemail")]
pub use manteau_jetemail::{
  JetEmail, JetEmailError, JetEmailErrorKind, JetEmailReceipt,
};
#[cfg(feature = "mailjet")]
pub use manteau_mailjet::{
  Mailjet, MailjetError, MailjetErrorKind, MailjetReceipt,
};
#[cfg(feature = "mock")]
pub use manteau_mock::*;
#[cfg(feature = "stdout")]
pub use manteau_stdout::*;
