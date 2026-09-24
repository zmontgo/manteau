//! Explicit local inspection of private mail through standard output.
//! This adapter prints recipients (including Bcc), subjects, and bodies. Use it
//! only where stdout is an appropriate destination for that information.
#![deny(missing_docs)]

use std::io::Write;

use async_trait::async_trait;
use manteau_core::{
  Acceptance, MessageId, PreparedMessage, Receipt, Transport, TransportFailure,
};
/// A local printer. It never sends email to a provider.
#[derive(Debug, Clone, Default)]
pub struct StdoutTransport {
  verbose: bool,
}

impl StdoutTransport {
  /// Print headers and a 280-character HTML preview (plaintext when HTML is
  /// absent).
  pub fn new() -> Self {
    Self::default()
  }

  /// Print both complete body alternatives instead of a preview.
  pub fn verbose(mut self, verbose: bool) -> Self {
    self.verbose = verbose;
    self
  }
}

/// Successful local output, not delivery evidence. Contains no provider IDs.
#[derive(Debug)]
pub struct StdoutReceipt;
impl Receipt for StdoutReceipt {
  fn ids(&self) -> &[MessageId] {
    &[]
  }
}

/// Stdout failed, potentially after writing part of the message.
#[derive(Debug, thiserror::Error)]
#[error("could not write email to stdout")]
pub struct StdoutError(#[source] std::io::Error);
impl TransportFailure for StdoutError {
  fn is_transient(&self) -> bool {
    false
  }

  fn is_auth(&self) -> bool {
    false
  }

  fn is_message_rejected(&self) -> bool {
    false
  }

  fn acceptance(&self) -> Acceptance {
    Acceptance::Unknown
  }
}

#[async_trait]
impl Transport for StdoutTransport {
  type Error = StdoutError;
  type Receipt = StdoutReceipt;

  async fn send(
    &self,
    message: &PreparedMessage,
  ) -> Result<StdoutReceipt, StdoutError> {
    let mut out = std::io::stdout().lock();
    let envelope = message.envelope();
    writeln!(out, "From: {}", envelope.from().mailbox())
      .map_err(StdoutError)?;
    for (label, addresses) in [
      ("To", envelope.recipients().to_addresses()),
      ("Cc", envelope.recipients().cc_addresses()),
      ("Bcc", envelope.recipients().bcc_addresses()),
    ] {
      for address in addresses {
        writeln!(out, "{label}: {}", address.mailbox()).map_err(StdoutError)?;
      }
    }
    writeln!(out, "Subject: {}", envelope.subject()).map_err(StdoutError)?;
    if self.verbose {
      writeln!(
        out,
        "\n{}\n\n{}",
        message.body().html(),
        message.body().text()
      )
      .map_err(StdoutError)?;
    } else {
      let body = if message.body().html().is_empty() {
        message.body().text()
      } else {
        message.body().html()
      };
      let mut chars = body.chars();
      let preview: String = chars.by_ref().take(280).collect();
      writeln!(
        out,
        "\n{preview}{}",
        if chars.next().is_some() { "…" } else { "" }
      )
      .map_err(StdoutError)?;
    }
    out.flush().map_err(StdoutError)?;
    Ok(StdoutReceipt)
  }
}
