//! Builder API — same content as the `welcome` example, expressed via the
//! typed builder DSL without the `mjml!` macro.
//!
//! Useful when:
//!   - you prefer a code-shape that builds the tree imperatively,
//!   - you need to assemble a template from values that are inconvenient to
//!     splice through `{expr}` (long generic chains, async-awaited intermediate
//!     values, etc.),
//!   - or you want to know what the macro is roughly expanding to.
//!
//! The macro and the builder produce the same `Template`; pick whichever
//! reads better for the situation.
//!
//! Run with:
//!
//! ```text
//! cargo run --example builder
//! ```

use manteau::{Message, MockTransport, Transport, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let temp_password = "u8X-7tWq-12pL";
  let invited_by = "Alice";
  let login_url = Url::try_parse("https://example.com/login")?;

  let body = Body::new()
    .background_color(Color::hex(0xfafafa))
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
              .font_family(FontFamily::new("Arial, sans-serif"))
              .font_size(Pixels::new(22))
              .font_weight(FontWeight::Bold)
              .color(Color::hex(0x1c1917)),
          ),
        ),
    )
    .push(
      Wrapper::new()
        .background_color(Color::hex(0xffffff))
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
                  .color(Color::hex(0x1c1917)),
              )
              .push(
                Text::new(format!(
                  "You've been invited by {invited_by}. Use the temporary \
                   password below to sign in for the first time."
                ))
                .font_size(Pixels::new(15))
                .color(Color::hex(0x57534e)),
              ),
          ),
        )
        .push(
          Section::new().padding(PaddingOptions::default()).push(
            Column::new()
              .background_color(Color::hex(0xf5f5f4))
              .border_radius(Pixels::new(8))
              .padding_top(Pixels::new(16))
              .padding_right(Pixels::new(20))
              .padding_bottom(Pixels::new(16))
              .padding_left(Pixels::new(20))
              .push(
                Text::new(temp_password)
                  .font_family(FontFamily::new(
                    "ui-monospace, SFMono-Regular, Menlo, monospace",
                  ))
                  .font_weight(FontWeight::Bold)
                  .font_size(Pixels::new(14)),
              ),
          ),
        )
        .push(
          Section::new().padding(PaddingOptions::default()).push(
            Column::new().padding_top(Pixels::new(16)).push(
              Button::new("Sign In", login_url)
                .background_color(Color::hex(0x292524))
                .color(Color::hex(0xfafaf9))
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
    Address::new("noreply@example.com".parse()?),
    vec![Address::new("you@example.com".parse()?)],
    "Welcome to Example Travel",
    template,
  );

  let transport = MockTransport::new();
  transport.send(&msg).await?;
  println!("Built template via the typed builder. Captured 1 message.");

  Ok(())
}
