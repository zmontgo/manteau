//! Consumer-authored email intent before rendering or transport submission.

use manteau_core::{
  Envelope, PlaintextBody, PreparedMessage, RenderError, Renderer,
  templating::Template,
};

/// An envelope, template, and optional plaintext alternative awaiting
/// preparation. A provider never receives this authoring value; it receives
/// the immutable `PreparedMessage` produced from it.
pub struct Message {
  envelope: Envelope,
  template: Template,
  text:     Option<PlaintextBody>,
}

impl Message {
  /// Bind a validated envelope to a typed template.
  pub fn new(envelope: Envelope, template: Template) -> Self {
    Self {
      envelope,
      template,
      text: None,
    }
  }

  /// Supply plaintext instead of deriving it from rendered HTML.
  pub fn text(mut self, text: PlaintextBody) -> Self {
    self.text = Some(text);
    self
  }

  /// Render once and freeze the exact envelope and bodies for all later sends.
  /// The authoring value remains available if rendering fails.
  #[tracing::instrument(skip_all)]
  pub fn prepare(
    &self,
    renderer: &impl Renderer,
  ) -> Result<PreparedMessage, RenderError> {
    let body = self
      .template
      .render_with_text(renderer, self.text.as_ref())?;
    Ok(PreparedMessage::new(self.envelope.clone(), body))
  }
}

impl std::fmt::Debug for Message {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Message([redacted])")
  }
}
