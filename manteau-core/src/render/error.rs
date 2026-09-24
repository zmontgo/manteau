//! Failure information for preparation through a rendering port.

/// The stage where preparing an email failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderErrorKind {
  /// The renderer could not turn generated MJML into HTML.
  Html,
  /// The renderer could not derive plaintext from HTML.
  Plaintext,
}

impl std::fmt::Display for RenderErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Html => f.write_str("could not render email HTML"),
      Self::Plaintext => f.write_str("could not render email plaintext"),
    }
  }
}

/// Sanitized preparation failure. The source retains details for an operator
/// who explicitly inspects it; ordinary formatting omits private content.
#[derive(thiserror::Error)]
#[error("{kind}")]
pub struct RenderError {
  kind:   RenderErrorKind,
  #[source]
  source: Box<dyn std::error::Error + Send + Sync>,
}

impl RenderError {
  /// The stage that failed.
  pub fn kind(&self) -> RenderErrorKind {
    self.kind
  }

  pub(crate) fn html(
    source: impl std::error::Error + Send + Sync + 'static,
  ) -> Self {
    Self {
      kind:   RenderErrorKind::Html,
      source: Box::new(source),
    }
  }

  pub(crate) fn plaintext(
    source: impl std::error::Error + Send + Sync + 'static,
  ) -> Self {
    Self {
      kind:   RenderErrorKind::Plaintext,
      source: Box::new(source),
    }
  }
}

impl std::fmt::Debug for RenderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}
