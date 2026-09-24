//! A custom MJML element can join a typed column without exposing raw markup
//! construction to its callers. The extension author owns renderer support.

use manteau::{
  render::{AttributeName, ElementName, MjmlWriter},
  templating::{Block, Body, Column, Element, Push, Section, Template},
};

#[derive(Debug)]
struct Divider {
  color: &'static str,
}

impl Element for Divider {
  fn write_mjml(&self, writer: &mut MjmlWriter) {
    writer
      .open(ElementName::custom("mj-divider").unwrap())
      .attr(
        AttributeName::custom("border-color").unwrap(),
        Some(&self.color),
      )
      .close_self();
  }
}

#[test]
fn custom_element() {
  let template = Template::new(
    Body::new().push(
      Section::new()
        .push(Column::new().push(Block::custom(Divider { color: "#333333" }))),
    ),
  );
  let mut writer = MjmlWriter::new();
  template.write_mjml(&mut writer);

  assert!(
    writer
      .as_str()
      .contains("<mj-divider border-color=\"#333333\"/>")
  );
}
