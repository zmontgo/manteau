# Manteau

Ergonomically build and send emails with Rust.

Focused on providing typed guarantees that your email content is valid, and abstract away the messy details of talking to an email provider API.

Currently, we support composing an MJML email with typed builders, rendering it, and shipping it via anything that implements the `Transport` trait. Mailjet is presently the only first-class implementer of this trait, but should you need another provider, we encourage you to consider contributing to this project and add it via a PR.

## Quick start

```toml
[dependencies]
manteau = { version = "0.1", features = ["mailjet"] }
```

```rust
use manteau::{Address, MailjetTransport, Message, Transport};
use manteau::templating::{Body, Column, Section, Template, Text};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let template = Template::builder()
    .body(Body::builder()
      .sections(vec![Section::builder()
        .columns(vec![Column::builder()
          .children(vec![Text::builder()
            .content("Hello, world!")
            .build()
            .into()])
          .build()])
        .build()])
      .build())
    .build();
  
  let msg = Message::builder()
    .from(Address::builder().email("hello@example.com".parse()?).build())
    .to(vec![Address::builder().email("you@example.com".parse()?).build()])
    .subject("Hi")
    .content(template)
    .build();
  
  let transport = MailjetTransport::builder()
    .api_key(std::env::var("MAILJET_API_KEY")?)
    .api_secret(std::env::var("MAILJET_API_SECRET")?)
    .build();
  
  transport.send(&msg).await?;
  # Ok(())
}
```

## Feature flags

- `mailjet` — `MailjetTransport`, posts to Mailjet's v3.1 send API. Implies
  `tls-rustls`.
- `stdout` — `StdoutTransport`, prints to stdout. For local development.
- `tls-rustls` / `tls-native` — selects the TLS backend for the HTTP
  transports. `tls-rustls` is the default chosen by `mailjet`.

`MockTransport` is always available; the doctest in `lib.rs` uses it.

## Custom transports

Implement `Transport` for your own provider. Your error type implements
`TransportFailure` so retry middleware can branch on the universal
categories (`is_transient`, `is_auth`, `is_message_rejected`,
`retry_after`).

```rust
use async_trait::async_trait;
use manteau::{Message, Transport, TransportFailure, Receipt, MessageId};

struct MyTransport;
struct MyReceipt { ids: Vec<MessageId> }
#[derive(Debug, thiserror::Error)]
#[error("failed")]
struct MyError;

impl Receipt for MyReceipt {
  fn ids(&self) -> &[MessageId] { &self.ids }
}

impl TransportFailure for MyError {
  fn is_transient(&self) -> bool { false }
  fn is_auth(&self) -> bool { false }
  fn is_message_rejected(&self) -> bool { false }
}

#[async_trait]
impl Transport for MyTransport {
  type Receipt = MyReceipt;
  type Error = MyError;
  async fn send(&self, _: &Message) -> Result<Self::Receipt, Self::Error> {
    Ok(MyReceipt { ids: vec![MessageId::new("custom-1")] })
  }
}
```

## Custom MJML elements

Implement `Element` and wrap with `Block::custom`:

```rust
use manteau::templating::{Block, Element};
use manteau::render::MjmlWriter;

#[derive(Debug)]
struct MyDivider;

impl Element for MyDivider {
  fn write_mjml(&self, w: &mut MjmlWriter) {
    w.open("mj-divider").close_self();
  }
}

let block = Block::custom(MyDivider);
```

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
