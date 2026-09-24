//! Typed email templates and independently selectable transports.
//! See the crate README for installation and the complete sending example.
pub use manteau_core::*;
pub use manteau_macros::mjml;
/// Common template, envelope, and transport imports.
pub mod prelude {
  pub use manteau_core::prelude::*;

  pub use crate::mjml;
}
#[cfg(feature = "cloudflare")]
pub use manteau_cloudflare::*;
#[cfg(feature = "jetemail")]
pub use manteau_jetemail::*;
#[cfg(feature = "mailjet")]
pub use manteau_mailjet::*;
#[cfg(feature = "mock")]
pub use manteau_mock::*;
#[cfg(feature = "stdout")]
pub use manteau_stdout::*;
