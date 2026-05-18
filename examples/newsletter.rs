//! Newsletter — demonstrates `mjml!`'s control-flow syntax.
//!
//! Shows `@for` over a list, `@if`/`@else` for conditional sections, and
//! `@match` for variant dispatch. Renders through `MockTransport` (no
//! features needed) and prints the resulting MJML.
//!
//! Run with:
//!
//! ```text
//! cargo run --example newsletter
//! ```

use manteau::prelude::*;
use manteau::{Message, MockTransport, Transport};

#[derive(Clone)]
struct Article {
  title:    &'static str,
  excerpt:  &'static str,
  category: Category,
}

#[derive(Clone)]
enum Category {
  Feature,
  Update,
  Tip,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let articles = vec![
    Article {
      title:    "Visa updates for European travel",
      excerpt:  "Schengen requirements are changing in late 2026 …",
      category: Category::Feature,
    },
    Article {
      title:    "New travel-policy module available",
      excerpt:  "Admins can now configure pre-trip approvals per group.",
      category: Category::Update,
    },
    Article {
      title:    "Tip: travel insurance disclosures",
      excerpt:  "Use the schema editor to require coverage details up front.",
      category: Category::Tip,
    },
  ];

  let subscriber_name = "Zach";
  let show_promo = true;

  let template = Template::new(mjml! {
    <Body background-color="#fafafa">
      <Section padding-top="32px">
        <Column>
          <Text font-size="24px" font-weight={FontWeight::Bold}>
            "Hello, " {subscriber_name} "!"
          </Text>
          <Text font-size="14px" color="#6b7280">
            "Here's what's new this month."
          </Text>
        </Column>
      </Section>

      @for article in &articles {
        <Section padding-top="0px" padding-bottom="16px">
          <Column
            background-color="#ffffff"
            border-radius="6px"
            padding-top="16px" padding-right="20px"
            padding-bottom="16px" padding-left="20px"
          >
            <Text
              font-size="11px"
              font-weight={FontWeight::SemiBold}
              text-transform={TextTransform::Uppercase}
              color="#9ca3af"
              letter-spacing="0.06em"
            >
              @match article.category {
                Category::Feature => { "Feature" }
                Category::Update  => { "Update"  }
                Category::Tip     => { "Tip"     }
              }
            </Text>
            <Text font-size="17px" font-weight={FontWeight::Bold}>
              {article.title}
            </Text>
            <Text font-size="14px" color="#4b5563">
              {article.excerpt}
            </Text>
          </Column>
        </Section>
      }

      @if show_promo {
        <Section padding-top="24px" padding-bottom="32px">
          <Column
            background-color="#1f2937"
            border-radius="8px"
            padding-top="20px" padding-right="24px"
            padding-bottom="20px" padding-left="24px"
          >
            <Text
              font-size="14px"
              font-weight={FontWeight::SemiBold}
              color="#fafaf9"
            >
              "Upgrade to Pro this month and save 20%."
            </Text>
            <Button
              href="https://example.com/upgrade"
              background-color="#fafaf9"
              color="#1f2937"
              border-radius="6px"
              font-size="13px"
              font-weight={FontWeight::SemiBold}
            >
              "See plans"
            </Button>
          </Column>
        </Section>
      }
    </Body>
  });

  let msg = Message::new(
    Address::new("newsletter@example.com".parse()?),
    vec![Address::new("you@example.com".parse()?)],
    "Your monthly travel newsletter",
    template,
  );

  // MockTransport is always available — no feature flag needed.
  let transport = MockTransport::new();
  transport.send(&msg).await?;
  let sent = transport.sent();
  println!("Captured {} message(s).", sent.len());
  let rendered = sent[0].render()?;
  println!("\n─── rendered HTML ────────────────────────────────────────");
  let preview: String = rendered.html.chars().take(800).collect();
  println!("{}…", preview);

  Ok(())
}
