//! Top-level [`Message`] type — the value that flows through
//! [`Transport::send`](crate::transport::Transport::send).

use typed_builder::TypedBuilder;

use crate::{
  models::Address,
  render::{RenderError, Rendered, render_html, render_plaintext},
  templating::Template,
};

/// An email: envelope (from/to/cc/bcc/subject) plus a renderable [`Template`].
#[non_exhaustive]
#[derive(Debug, Clone, TypedBuilder)]
pub struct Message {
  pub from:    Address,
  #[builder(setter(into))]
  pub to:      Vec<Address>,
  #[builder(default, setter(into))]
  pub cc:      Vec<Address>,
  #[builder(default, setter(into))]
  pub bcc:     Vec<Address>,
  #[builder(setter(into))]
  pub subject: String,
  pub content: Template,
  /// Override the plaintext alternative. When `None`, plaintext is derived
  /// from the rendered HTML via `html2text`.
  #[builder(default, setter(strip_option, into))]
  pub text:    Option<String>,
}

impl Message {
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
