//! Top-level [`Message`] type — the value that flows through
//! [`Transport::send`](crate::transport::Transport::send).

use crate::{
  models::Address,
  render::{RenderError, Rendered, render_html, render_plaintext},
  templating::Template,
};

/// An email: envelope (from/to/cc/bcc/subject) plus a renderable [`Template`].
///
/// Required envelope fields go through [`Message::new`]; optional `cc`,
/// `bcc`, and `text` override are chained setters.
///
/// ```
/// # use manteau::{Address, Message};
/// # use manteau::templating::{Body, Template};
/// let from = Address::new("hello@example.com".parse()?);
/// let to = vec![Address::new("you@example.com".parse()?)];
/// let msg = Message::new(from, to, "Hi", Template::new(Body::new()))
///   .cc(vec![Address::new("loop@example.com".parse()?)]);
/// # Ok::<(), manteau::EmailAddressError>(())
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Message {
  pub from:    Address,
  pub to:      Vec<Address>,
  pub cc:      Vec<Address>,
  pub bcc:     Vec<Address>,
  pub subject: String,
  pub content: Template,
  /// Override the plaintext alternative. When `None`, plaintext is derived
  /// from the rendered HTML via `html2text`.
  pub text:    Option<String>,
}

impl Message {
  pub fn new(
    from: Address,
    to: Vec<Address>,
    subject: impl Into<String>,
    content: Template,
  ) -> Self {
    Self {
      from,
      to,
      cc: Vec::new(),
      bcc: Vec::new(),
      subject: subject.into(),
      content,
      text: None,
    }
  }

  pub fn cc(mut self, cc: Vec<Address>) -> Self {
    self.cc = cc;
    self
  }

  /// Append one address to `cc`. Mirror of [`Message::cc`] for the
  /// runtime-conditional pattern (parallel to `push_section` on `Body`).
  pub fn push_cc(mut self, addr: Address) -> Self {
    self.cc.push(addr);
    self
  }

  pub fn bcc(mut self, bcc: Vec<Address>) -> Self {
    self.bcc = bcc;
    self
  }

  /// Append one address to `bcc`. Mirror of [`Message::bcc`] for the
  /// runtime-conditional pattern.
  pub fn push_bcc(mut self, addr: Address) -> Self {
    self.bcc.push(addr);
    self
  }

  /// Override the plaintext alternative. Without this, plaintext is derived
  /// from the rendered HTML via `html2text`.
  pub fn text(mut self, text: impl Into<String>) -> Self {
    self.text = Some(text.into());
    self
  }

  /// Render the content to HTML and plaintext. Single source of truth for
  /// what gets handed to a transport.
  ///
  /// If [`Message::text`] is `Some`, that value is used as the plaintext
  /// alternative verbatim; otherwise plaintext is derived from the HTML.
  #[tracing::instrument(skip_all, fields(subject = %self.subject))]
  pub fn render(&self) -> Result<Rendered, RenderError> {
    let html = render_html(&self.content)?;
    let text = match &self.text {
      Some(t) => t.clone(),
      None => render_plaintext(&html)?,
    };
    Ok(Rendered { html, text })
  }
}
