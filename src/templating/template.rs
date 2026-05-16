use typed_builder::TypedBuilder;

use crate::{
  render::MjmlWriter,
  templating::{body::Body, element::Element},
};

/// The top-level MJML document — `<mjml>` wrapping an optional `<mj-head>`
/// (`title`, `preview_text`) and a required [`Body`].
///
/// This is what [`Message::content`] holds. Renaming from the older `Email`
/// keeps the vocabulary honest: an [`Message`] is an email; a `Template` is
/// the renderable body the message carries.
///
/// [`Message`]: crate::message::Message
/// [`Message::content`]: crate::message::Message
#[non_exhaustive]
#[derive(Debug, Clone, TypedBuilder)]
pub struct Template {
  pub body:         Body,
  #[builder(default, setter(strip_option, into))]
  pub preview_text: Option<String>,
  #[builder(default, setter(strip_option, into))]
  pub title:        Option<String>,
}

impl Element for Template {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mjml").children(|w| {
      if self.title.is_some() || self.preview_text.is_some() {
        w.open("mj-head").children(|w| {
          if let Some(t) = &self.title {
            w.open("mj-title").text(t);
          }
          if let Some(p) = &self.preview_text {
            w.open("mj-preview").text(p);
          }
        });
      }
      self.body.write_mjml(w);
    });
  }
}
