//! Transport adapters implementing [`Transport`](crate::transport::Transport).
//!
//! - [`mock`] — always available, in-memory test capture.
//! - [`stdout`] — behind the `stdout` feature, prints to stdout.
//! - [`mailjet`] — behind the `mailjet` feature, HTTP API client.

pub mod mock;

#[cfg(feature = "mailjet")]
pub mod mailjet;

#[cfg(feature = "stdout")]
pub mod stdout;
