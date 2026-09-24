//! A local newsletter capture demonstrating `mjml!` control flow.
//!
//! The test checks capture count without printing private rendered mail.

use manteau::{Message, Transport, prelude::*};
use manteau_mock::MockTransport;

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

#[tokio::test]
async fn newsletter() -> Result<(), Box<dyn std::error::Error>> {
  let articles = vec![
    Article {
      title:    "New product features",
      excerpt:  "A roundup of this month’s improvements.",
      category: Category::Feature,
    },
    Article {
      title:    "New account settings available",
      excerpt:  "Manage your notification preferences.",
      category: Category::Update,
    },
    Article {
      title:    "Tip: managing notifications",
      excerpt:  "Choose only the updates you want to receive.",
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
    Envelope::new(
      Address::new("newsletter@example.com".parse()?),
      Recipients::to(Address::new("you@example.com".parse()?)),
      HeaderText::new("Your monthly newsletter")?,
    ),
    template,
  )
  .prepare(&manteau::MrmlRenderer::new())?;

  // MockTransport is always available — no feature flag needed.
  let transport = MockTransport::new();
  transport.send(&msg).await?;
  assert_eq!(transport.sent().len(), 1);

  Ok(())
}
