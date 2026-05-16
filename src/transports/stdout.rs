//! Stdout transport — prints the envelope and a preview (or full) HTML body
//! to stdout. For local development and CLI tools.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use typed_builder::TypedBuilder;

use crate::{
  message::Message,
  models::{Address, MessageId},
  render::RenderError,
  transport::{Receipt, Transport},
};

#[derive(Debug, Clone, TypedBuilder)]
pub struct StdoutTransport {
  /// When true, prints the full rendered HTML. When false (default),
  /// prints only the first 280 characters as a preview.
  #[builder(default)]
  verbose: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct StdoutReceipt {
  pub ids: Vec<MessageId>,
}

impl Receipt for StdoutReceipt {
  fn ids(&self) -> &[MessageId] { &self.ids }
}

static STDOUT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[async_trait]
impl Transport for StdoutTransport {
  type Error = RenderError;
  type Receipt = StdoutReceipt;

  #[tracing::instrument(skip_all, fields(subject = %message.subject))]
  async fn send(
    &self,
    message: &Message,
  ) -> Result<Self::Receipt, Self::Error> {
    let rendered = message.render()?;

    println!("─── manteau: outgoing message ─────────────────────────────");
    println!("From:    {}", format_address(&message.from));
    for to in &message.to {
      println!("To:      {}", format_address(to));
    }
    for cc in &message.cc {
      println!("Cc:      {}", format_address(cc));
    }
    for bcc in &message.bcc {
      println!("Bcc:     {}", format_address(bcc));
    }
    println!("Subject: {}", message.subject);
    println!("───────────────────────────────────────────────────────────");
    if self.verbose {
      println!("{}", rendered.html);
    } else {
      let preview: String = rendered.html.chars().take(280).collect();
      let ellipsis = if rendered.html.len() > 280 { "…" } else { "" };
      println!("{preview}{ellipsis}");
    }
    println!("───────────────────────────────────────────────────────────");

    let n = STDOUT_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(StdoutReceipt {
      ids: vec![MessageId::new(format!("stdout-{n}"))],
    })
  }
}

fn format_address(a: &Address) -> String {
  match &a.name {
    Some(name) => format!("{name} <{}>", a.email),
    None => a.email.to_string(),
  }
}
