//! The welcome email built with owner methods rather than `mjml!`.
//!
//! This is useful when composing trees from ordinary Rust values.

use manteau::{Message, Transport, prelude::*};
use manteau_mock::MockTransport;

#[tokio::test]
async fn builder() -> Result<(), Box<dyn std::error::Error>> {
  let temp_password = "u8X-7tWq-12pL";
  let invited_by = "Alice";
  let login_url = Url::try_parse("https://example.com/login")?;

  let body = Body::new()
    .background_color(Color::hex(0xfafafa).unwrap())
    .push(
      Section::new()
        .padding(
          PaddingOptions::new()
            .t(Pixels::new(32).into())
            .b(Pixels::new(24).into()),
        )
        .push(
          Column::new().push(
            Text::new("Example Travel")
              .font_family(FontFamily::new("Arial, sans-serif").unwrap())
              .font_size(Pixels::new(22))
              .font_weight(FontWeight::Bold)
              .color(Color::hex(0x1c1917).unwrap()),
          ),
        ),
    )
    .push(
      Wrapper::new()
        .background_color(Color::hex(0xffffff).unwrap())
        .border_radius(Pixels::new(8))
        .padding_top(Pixels::new(32))
        .padding_right(Pixels::new(32))
        .padding_bottom(Pixels::new(32))
        .padding_left(Pixels::new(32))
        .push(
          Section::new().padding(PaddingOptions::default()).push(
            Column::new()
              .push(
                Text::new("Welcome")
                  .font_size(Pixels::new(20))
                  .font_weight(FontWeight::Bold)
                  .color(Color::hex(0x1c1917).unwrap()),
              )
              .push(
                Text::new(format!(
                  "You've been invited by {invited_by}. Use the temporary \
                   password below to sign in for the first time."
                ))
                .font_size(Pixels::new(15))
                .color(Color::hex(0x57534e).unwrap()),
              ),
          ),
        )
        .push(
          Section::new().padding(PaddingOptions::default()).push(
            Column::new()
              .background_color(Color::hex(0xf5f5f4).unwrap())
              .border_radius(Pixels::new(8))
              .padding_top(Pixels::new(16))
              .padding_right(Pixels::new(20))
              .padding_bottom(Pixels::new(16))
              .padding_left(Pixels::new(20))
              .push(
                Text::new(temp_password)
                  .font_family(
                    FontFamily::new(
                      "ui-monospace, SFMono-Regular, Menlo, monospace",
                    )
                    .unwrap(),
                  )
                  .font_weight(FontWeight::Bold)
                  .font_size(Pixels::new(14)),
              ),
          ),
        )
        .push(
          Section::new().padding(PaddingOptions::default()).push(
            Column::new().padding_top(Pixels::new(16)).push(
              Button::new("Sign In", login_url)
                .background_color(Color::hex(0x292524).unwrap())
                .color(Color::hex(0xfafaf9).unwrap())
                .border_radius(Pixels::new(8))
                .font_size(Pixels::new(14))
                .font_weight(FontWeight::SemiBold)
                .align(ButtonAlignment::Left),
            ),
          ),
        ),
    );

  let template = Template::new(body);

  let msg = Message::new(
    Envelope::new(
      Address::new("noreply@example.com".parse()?),
      Recipients::to(Address::new("you@example.com".parse()?)),
      HeaderText::new("Welcome to Example Travel")?,
    ),
    template,
  )
  .prepare(&manteau::MrmlRenderer::new())?;

  let transport = MockTransport::new();
  transport.send(&msg).await?;
  assert_eq!(transport.sent().len(), 1);

  Ok(())
}
