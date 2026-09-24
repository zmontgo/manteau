//! A renderer implementation can replace `mrml` without changing `Mailer` or
//! a transport. Its outputs must uphold the checked body contracts.

use std::convert::Infallible;

use manteau::{
  Address, Envelope, HeaderText, HtmlBody, Mailer, Message, MjmlDocument,
  PlaintextBody, Recipients, Renderer,
  templating::{Body, Column, Push, Section, Template, Text},
};
use manteau_mock::MockTransport;

struct StaticRenderer;

impl Renderer for StaticRenderer {
  type Error = Infallible;

  fn html(&self, _: &MjmlDocument) -> Result<HtmlBody, Self::Error> {
    Ok(HtmlBody::new("<p>Hello</p>").unwrap())
  }

  fn plaintext(&self, _: &HtmlBody) -> Result<PlaintextBody, Self::Error> {
    Ok(PlaintextBody::new("Hello").unwrap())
  }
}

#[tokio::test]
async fn custom_renderer() -> Result<(), Box<dyn std::error::Error>> {
  let envelope = Envelope::new(
    Address::new("sender@example.com".parse()?),
    Recipients::to(Address::new("reader@example.com".parse()?)),
    HeaderText::new("Hello")?,
  );
  let template = Template::new(
    Body::new()
      .push(Section::new().push(Column::new().push(Text::new("Hello")))),
  );
  let mailer = Mailer::new(StaticRenderer, MockTransport::new());
  mailer.send(Message::new(envelope, template)).await?;

  let captured = mailer.transport().sent();
  assert_eq!(captured[0].body().html(), "<p>Hello</p>");
  assert_eq!(captured[0].body().text(), "Hello");
  Ok(())
}
