use crate::render::MjmlWriter;
use crate::templating::button::Button;
use crate::templating::element::Element;
use crate::templating::image::Image;
use crate::templating::text::Text;

/// Anything that can live inside a [`Column`] — a leaf MJML element.
///
/// Library-known leaves are first-class variants for ergonomic, compile-time
/// typed construction. Consumer-defined elements plug in via [`Block::Custom`]
/// (any type implementing [`Element`]).
///
/// [`Column`]: crate::templating::column::Column
#[derive(Debug)]
pub enum Block {
    Text(Text),
    Button(Button),
    Image(Image),
    Custom(Box<dyn Element>),
}

impl Element for Block {
    fn write_mjml(&self, w: &mut MjmlWriter) {
        match self {
            Self::Text(t) => t.write_mjml(w),
            Self::Button(b) => b.write_mjml(w),
            Self::Image(i) => i.write_mjml(w),
            Self::Custom(c) => c.write_mjml(w),
        }
    }
}

impl From<Text> for Block {
    fn from(t: Text) -> Self {
        Self::Text(t)
    }
}

impl From<Button> for Block {
    fn from(b: Button) -> Self {
        Self::Button(b)
    }
}

impl From<Image> for Block {
    fn from(i: Image) -> Self {
        Self::Image(i)
    }
}
