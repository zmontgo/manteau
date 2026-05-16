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

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let template = Template::new(
    Body::new().push_section(
      Section::new().push_column(
        Column::new().push(Text::new("Hello, world!")),
      ),
    ),
  );
  
  let msg = Message::new(
    Address::new("hello@example.com".parse()?),
    vec![Address::new("you@example.com".parse()?)],
    "Hi",
    template,
  );
  
  let transport = MailjetTransport::new(
    std::env::var("MAILJET_API_KEY")?,
    std::env::var("MAILJET_API_SECRET")?,
  );
  
  transport.send(&msg).await?;
  Ok(())
}
```

## Runtime-conditional construction

The `push_*` accumulators let you build a template incrementally. For
example, tacking on a coupon section if a user qualifies:

```rust
# use manteau::templating::{Body, Column, Section, Text};
# let user_coupon_eligible = true;
# let welcome_section = Section::new();
# let coupon_section = Section::new();
let mut body = Body::new().push_section(welcome_section);
if user_coupon_eligible {
  body = body.push_section(coupon_section);
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

#[async_trait::async_trait]
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

## Note about AI

With the use of AI and mostly-AI built applications being a hot-button issue, I want to be transparent.

The first version of this crate was an internal tool I needed built quickly, which I realized may benefit others.
Thus, it was built almost entirely using AI, and hastily published. Because of this, I do not consider the v0.1.0 release well-tested, and while I rather carefully refined the API I wanted, I default to not considering it necessarily well-built.

I will be hand-revising this in the coming days until a properly vetted 0.2.0 release can be produced. At that point, I would consider it feature-complete. If you see this message, I invite you to contribute. I have a deep love for the craft of software design, and would love to partner with others who do as well.

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
