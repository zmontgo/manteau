//! Buffered writer for MJML output.
//!
//! [`MjmlWriter`] owns the output buffer and handles escaping. Elements never
//! `format!` raw MJML — they declare tag + attributes + content through
//! [`ElementWriter`]'s builder methods, which handle the writing.

/// Accumulating buffer that produces an MJML document.
#[derive(Debug, Default)]
pub struct MjmlWriter {
  buf: String,
}

impl MjmlWriter {
  pub fn new() -> Self { Self { buf: String::new() } }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      buf: String::with_capacity(capacity),
    }
  }

  /// Begin an element. Returns a builder that accumulates attributes, then
  /// terminates with [`ElementWriter::text`], [`ElementWriter::children`],
  /// or [`ElementWriter::close_self`].
  pub fn open(&mut self, tag: &'static str) -> ElementWriter<'_> {
    self.buf.push('<');
    self.buf.push_str(tag);
    ElementWriter { writer: self, tag }
  }

  /// Write a raw MJML fragment without escaping. Reserved for cases where
  /// the caller has already produced an MJML-safe string (e.g. concatenating
  /// pre-rendered children). Most code should use `open`.
  pub fn write_raw(&mut self, fragment: &str) { self.buf.push_str(fragment); }

  /// Consume the writer and return the accumulated MJML string.
  pub fn into_string(self) -> String { self.buf }

  /// Borrow the current buffer for inspection.
  pub fn as_str(&self) -> &str { &self.buf }
}

/// Per-element builder. Methods are consuming so that "attr after close" is a
/// compile-time error — the type system makes misuse impossible.
pub struct ElementWriter<'a> {
  writer: &'a mut MjmlWriter,
  tag:    &'static str,
}

impl<'a> ElementWriter<'a> {
  /// Add an attribute if `value` is `Some`. The value's `Display` impl
  /// produces the attribute text; the writer handles escaping.
  pub fn attr<V: std::fmt::Display>(
    self,
    name: &str,
    value: Option<&V>,
  ) -> Self {
    if let Some(v) = value {
      self.writer.buf.push(' ');
      self.writer.buf.push_str(name);
      self.writer.buf.push_str("=\"");
      escape_attr_into(&mut self.writer.buf, &v.to_string());
      self.writer.buf.push('"');
    }
    self
  }

  /// Close the opening tag, write escaped text, write the closing tag.
  pub fn text(self, content: &str) {
    self.writer.buf.push('>');
    escape_text_into(&mut self.writer.buf, content);
    self.writer.buf.push_str("</");
    self.writer.buf.push_str(self.tag);
    self.writer.buf.push('>');
  }

  /// Close the opening tag, invoke `f` to write children, write the
  /// closing tag.
  pub fn children(self, f: impl FnOnce(&mut MjmlWriter)) {
    self.writer.buf.push('>');
    f(self.writer);
    self.writer.buf.push_str("</");
    self.writer.buf.push_str(self.tag);
    self.writer.buf.push('>');
  }

  /// Close as a self-closing tag (`<tag .../>`).
  pub fn close_self(self) { self.writer.buf.push_str("/>"); }
}

fn escape_attr_into(buf: &mut String, s: &str) {
  for c in s.chars() {
    match c {
      '&' => buf.push_str("&amp;"),
      '"' => buf.push_str("&quot;"),
      '<' => buf.push_str("&lt;"),
      '>' => buf.push_str("&gt;"),
      _ => buf.push(c),
    }
  }
}

fn escape_text_into(buf: &mut String, s: &str) {
  for c in s.chars() {
    match c {
      '&' => buf.push_str("&amp;"),
      '<' => buf.push_str("&lt;"),
      '>' => buf.push_str("&gt;"),
      _ => buf.push(c),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn open_and_text() {
    let mut w = MjmlWriter::new();
    w.open("mj-text").text("hello");
    assert_eq!(w.into_string(), "<mj-text>hello</mj-text>");
  }

  #[test]
  fn attrs_written_when_some() {
    let mut w = MjmlWriter::new();
    let color = Some("red".to_string());
    let absent: Option<String> = None;
    w.open("mj-text")
      .attr("color", color.as_ref())
      .attr("font-size", absent.as_ref())
      .text("hi");
    assert_eq!(w.into_string(), r#"<mj-text color="red">hi</mj-text>"#);
  }

  #[test]
  fn text_content_escaped() {
    let mut w = MjmlWriter::new();
    w.open("mj-text").text("a < b & c > d");
    assert_eq!(
      w.into_string(),
      "<mj-text>a &lt; b &amp; c &gt; d</mj-text>"
    );
  }

  #[test]
  fn attr_value_escaped() {
    let mut w = MjmlWriter::new();
    let v = Some(r#"a"b<c"#.to_string());
    w.open("mj-image").attr("src", v.as_ref()).close_self();
    assert_eq!(w.into_string(), r#"<mj-image src="a&quot;b&lt;c"/>"#);
  }

  #[test]
  fn children_callback() {
    let mut w = MjmlWriter::new();
    w.open("mj-section").children(|w| {
      w.open("mj-column").text("x");
    });
    assert_eq!(
      w.into_string(),
      "<mj-section><mj-column>x</mj-column></mj-section>"
    );
  }

  #[test]
  fn self_closing() {
    let mut w = MjmlWriter::new();
    let src = Some("https://example.com/img.png".to_string());
    w.open("mj-image").attr("src", src.as_ref()).close_self();
    assert_eq!(
      w.into_string(),
      r#"<mj-image src="https://example.com/img.png"/>"#
    );
  }
}
