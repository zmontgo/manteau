//! Errors produced by the render pipeline.
//!
//! Follows manteau's standard error shape — opaque struct over a private
//! `kind` taxonomy + preserved `source`. `Display` delegates to the source so
//! operator logs carry the upstream library's message intact. Callers branch
//! on [`RenderError::kind`] for programmatic decisions.

/// The category of a [`RenderError`] failure.
///
/// Closed set, named for the failure source within the render pipeline.
/// All variants are pure tags — incident details (input that failed, mrml
/// position, html2text context) live in the `source` chain.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RenderErrorKind {
  /// `mrml::parse` rejected the MJML we generated. Indicates a bug in
  /// manteau's renderer producing malformed MJML.
  #[error("failed to parse generated MJML")]
  Parse,
  /// `mrml` parsed the MJML but failed to render it to HTML. Indicates a
  /// bug in mrml or pathological MJML structure.
  #[error("failed to render MJML to HTML")]
  Render,
  /// `html2text` failed to convert the rendered HTML to plaintext.
  /// Should not occur on well-formed HTML produced by mrml.
  #[error("failed to convert HTML to plaintext")]
  Plaintext,
}

/// Error returned from the render pipeline.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct RenderError {
  kind:   RenderErrorKind,
  #[source]
  source: Box<dyn std::error::Error + Send + Sync>,
}

impl RenderError {
  /// The category of failure. Callers branch on this for programmatic
  /// decisions; the human-readable message comes from the source chain.
  pub fn kind(&self) -> RenderErrorKind { self.kind }
}

impl RenderErrorKind {
  /// Internal constructor — pair a kind with the underlying source error.
  /// Not exposed publicly so consumers cannot fabricate manteau errors.
  pub(crate) fn err(
    self,
    source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
  ) -> RenderError {
    RenderError {
      kind:   self,
      source: source.into(),
    }
  }
}
