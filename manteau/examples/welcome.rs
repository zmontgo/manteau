//! Welcome email — quickstart for the `mjml!` macro.
//!
//! Builds a multi-section welcome message with a styled card containing a
//! one-time password and a CTA button, then renders the rendered HTML
//! preview via the stdout transport.
//!
//! Run with:
//!
//! ```text
//! cargo run --example welcome --features stdout
//! ```

use manteau::{Message, StdoutTransport, Transport, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let temp_password = "u8X-7tWq-12pL";
  let invited_by = "Alice";
  let login_url = Url::try_parse("https://example.com/login")?;

  let template = Template::new(mjml! {
    <Body background-color="#f5f5f4">
      <Section padding-top="32px" padding-bottom="24px">
        <Column>
          <Text
            font-family="Arial, sans-serif"
            font-size="22px"
            font-weight={FontWeight::Bold}
            color="#1c1917"
          >
            "Example Travel"
          </Text>
        </Column>
      </Section>

      <Wrapper
        background-color="#ffffff"
        border-radius="8px"
        padding-top="32px" padding-right="32px"
        padding-bottom="32px" padding-left="32px"
      >
        <Section
          padding-top="0px" padding-right="0px"
          padding-bottom="0px" padding-left="0px"
        >
          <Column>
            <Text font-size="20px" font-weight={FontWeight::Bold} color="#1c1917">
              "Welcome"
            </Text>
            <Text font-size="15px" color="#57534e">
              "You've been invited by " {invited_by} ". Use the temporary "
              "password below to sign in for the first time."
            </Text>
          </Column>
        </Section>

        <Section
          padding-top="0px" padding-right="0px"
          padding-bottom="0px" padding-left="0px"
        >
          <Column
            background-color="#f5f5f4"
            border-radius="8px"
            padding-top="16px" padding-right="20px"
            padding-bottom="16px" padding-left="20px"
          >
            <Text
              font-family="ui-monospace, SFMono-Regular, Menlo, monospace"
              font-weight={FontWeight::Bold}
              font-size="14px"
            >
              {temp_password}
            </Text>
          </Column>
        </Section>

        <Section
          padding-top="0px" padding-right="0px"
          padding-bottom="0px" padding-left="0px"
        >
          <Column padding-top="16px">
            <Button
              href={login_url}
              background-color="#292524"
              color="#fafaf9"
              border-radius="8px"
              font-size="14px"
              font-weight={FontWeight::SemiBold}
              align={ButtonAlignment::Left}
            >
              "Sign In"
            </Button>
          </Column>
        </Section>
      </Wrapper>
    </Body>
  });

  let msg = Message::new(
    Address::new("noreply@example.com".parse()?),
    vec![Address::new("you@example.com".parse()?)],
    "Welcome to Example Travel",
    template,
  );

  let transport = StdoutTransport::new();
  let receipt = transport.send(&msg).await?;
  println!("\nSent. Message IDs: {:?}", receipt.ids);

  Ok(())
}
