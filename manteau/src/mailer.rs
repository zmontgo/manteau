//! Consumer workflow that composes a renderer with a delivery transport.

use manteau_core::{PreparedMessage, RenderError, Renderer, Transport};

use crate::Message;

/// Configured email capability. One-step sends render once and submit once;
/// durable callers can prepare, persist, then use the underlying transport.
pub struct Mailer<R, T> {
  renderer:  R,
  transport: T,
}

impl<R: Renderer, T: Transport> Mailer<R, T> {
  /// Bind the chosen renderer and delivery mechanism.
  pub fn new(renderer: R, transport: T) -> Self {
    Self {
      renderer,
      transport,
    }
  }

  /// Prepare immutable content that can be persisted before dispatch.
  pub fn prepare(
    &self,
    message: &Message,
  ) -> Result<PreparedMessage, RenderError> {
    message.prepare(&self.renderer)
  }

  /// Expose the configured delivery capability for a persisted submission.
  pub fn transport(&self) -> &T {
    &self.transport
  }

  /// Consume authoring intent, render once, and submit once. On failure the
  /// returned value retains the original message or exact prepared content.
  pub async fn send(
    &self,
    message: Message,
  ) -> Result<T::Receipt, MailerError<T::Error>> {
    let prepared = self
      .prepare(&message)
      .map_err(|error| MailerError::Preparation { message, error })?;
    self
      .transport
      .send(&prepared)
      .await
      .map_err(|error| MailerError::Submission { prepared, error })
  }
}

#[cfg(any(feature = "mailjet", feature = "cloudflare", feature = "jetemail"))]
impl<R: Renderer, P: manteau_core::HttpProvider>
  Mailer<R, manteau_core::Sender<P, manteau_http::HttpClient>>
{
  /// Assemble a provider with the standard pooled HTTP driver.
  pub fn with_http(
    renderer: R,
    provider: P,
  ) -> Result<Self, manteau_core::http::HttpError> {
    let http = manteau_http::HttpClient::new()?;
    Ok(Self::new(
      renderer,
      manteau_core::Sender::new(provider, http),
    ))
  }
}

impl<R, T> std::fmt::Debug for Mailer<R, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Mailer([redacted])")
  }
}

/// Preparation or submission failure retaining what the caller needs to
/// diagnose or recover without recreating an email with different content.
#[derive(thiserror::Error)]
pub enum MailerError<E: manteau_core::TransportFailure> {
  /// Rendering failed; the authoring message can be retried after correction.
  #[error("could not prepare email")]
  Preparation {
    /// Original authoring intent.
    message: Message,
    /// Rendering failure with explicit source access.
    #[source]
    error:   RenderError,
  },

  /// Submission failed; inspect acceptance evidence before deciding to retry.
  #[error("could not submit prepared email")]
  Submission {
    /// Exact immutable message sent or possibly sent to the provider.
    prepared: PreparedMessage,
    /// Transport failure retaining provider evidence.
    #[source]
    error:    E,
  },
}

impl<E: manteau_core::TransportFailure> std::fmt::Debug for MailerError<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}
