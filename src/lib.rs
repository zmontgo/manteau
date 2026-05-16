//! Manteau — typed builders for MJML emails with pluggable transports.
//!
//! Restructure in progress. Modules are wired in incrementally as they land.

pub mod models;
pub mod templating;

pub use models::{Address, EmailAddress, EmailAddressError, MessageId};
