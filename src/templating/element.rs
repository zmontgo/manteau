//! The [`Element`] trait — the contract every renderable MJML element
//! implements, including consumer-defined elements.

use crate::render::MjmlWriter;

/// Anything that can write itself as an MJML fragment.
///
/// Library-known elements (`Text`, `Button`, `Image`, `Column`, `Section`,
/// `Body`, `Template`) implement this. Consumers extending the element set
/// for their own MJML elements implement it on their own types — those then
/// flow through containers via the `Custom` variant of [`Block`].
///
/// [`Block`]: crate::templating::block::Block
pub trait Element: std::fmt::Debug + Send + Sync {
  /// Write this element's MJML representation into `w`.
  fn write_mjml(&self, w: &mut MjmlWriter);
}
