//! Rendering capability required by template preparation.

/// Converts core-generated MJML to HTML and, when needed, HTML to plaintext.
/// The two operations let core skip plaintext conversion when the caller
/// supplied an alternative. Implementations decide which rendering libraries
/// to use and must not resolve remote resources implicitly.
pub trait Renderer: Send + Sync {
  /// Concrete failure with a preserved source chain.
  type Error: std::error::Error + Send + Sync + 'static;

  /// Render a complete MJML document to HTML.
  fn html(&self, mjml: &str) -> Result<String, Self::Error>;

  /// Generate a readable plaintext alternative from rendered HTML.
  fn plaintext(&self, html: &str) -> Result<String, Self::Error>;
}
