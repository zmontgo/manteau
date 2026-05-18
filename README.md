# Manteau

Ergonomically build and send emails with Rust.

Focused on providing typed guarantees that your email content is valid, and abstracting away the messy details of talking to an email provider API.

Currently, manteau supports composing an MJML email with typed builders **or** the declarative `mjml!` proc-macro, rendering it via [mrml](https://crates.io/crates/mrml), and shipping it through anything that implements the `Transport` trait. Mailjet is the first-class provider; if you need another, contributions are welcome.

## Quick start

```toml
[dependencies]
manteau = { version = "0.1", features = ["mailjet"] }
tokio   = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use manteau::prelude::*;
use manteau::{MailjetTransport, Message, Transport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let template = Template::new(mjml! {
    <Body background-color="#f5f5f4">
      <Section padding-top="32px">
        <Column>
          <Text font-size="22px" font-weight={FontWeight::Bold}>
            "Welcome to Example Travel"
          </Text>
          <Text color="#57534e">
            "You've been invited to join our travel-security platform."
          </Text>
          <Button
            href="https://example.com/login"
            background-color="#292524"
            color="#fafaf9"
          >
            "Sign In"
          </Button>
        </Column>
      </Section>
    </Body>
  });

  let msg = Message::new(
    Address::new("noreply@example.com".parse()?),
    vec![Address::new("you@example.com".parse()?)],
    "Welcome",
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

The `mjml!` macro recognizes a closed set of element tags — `<Body>`, `<Wrapper>`, `<Section>`, `<Column>`, `<Text>`, `<Button>`, `<Image>` — and validates attribute values at compile time: malformed colors, out-of-range percentages, or invalid URLs become build errors that point at the offending attribute.

## Attribute syntax

Two forms:

- **String literals** — `font-size="22px"`, `color="#f00"`, `width="50%"`, `href="https://example.com"`. The macro inspects the contents at expansion time and emits the right typed constructor (`Pixels::new(22)`, `Color::hex(0xff0000)`, `Percentage::new(50).expect(...)`, `Url::try_parse(...).expect(...)`). Unrecognized strings are passed through unchanged, so `font-family="Arial, sans-serif"` and other free-form text attributes work.
- **Braced expressions** — `color={Color::hex(0x1c1917)}`, `font-weight={FontWeight::Bold}`, `align={Alignment::Center}`. For typed values that aren't expressible as parseable strings (closed-set enums like `FontWeight`/`Alignment`/`TextTransform`/`ButtonAlignment`) or for spliced-in Rust values.

Recognized string-literal forms:

| Form | Becomes |
|---|---|
| `"#rgb"` / `"#rrggbb"` | `Color::hex(...)` |
| `"Npx"` | `Pixels::new(N)` |
| `"N.Nem"` / `"N.Nrem"` | `Em::new(N.N)` / `Rem::new(N.N)` |
| `"N%"` | `Percentage::new(N).expect(...)` (0–100 validated at expansion) |
| `"http(s)://…"`, `"mailto:…"`, `"tel:…"`, `"data:…"` | `Url::try_parse(...).expect(...)` (URL syntax validated at expansion) |

## Control flow

Inside container-bodied elements (`<Body>`, `<Wrapper>`, `<Section>`, `<Column>`), the macro accepts `@if`, `@for`, `@while`, and `@match` for runtime-conditional content. Each branch's body is a brace-delimited block of nodes (child elements, `{expr}` splices, more control flow).

```rust
use manteau::prelude::*;

let users = vec!["Alice", "Bob"];
let admin = true;

let template = Template::new(mjml! {
  <Body>
    <Section>
      <Column>
        @for name in &users {
          <Text>"Hello, " {name} "!"</Text>
        }

        @if admin {
          <Text>"You have admin privileges."</Text>
        } @else {
          <Text>"Standard user account."</Text>
        }
      </Column>
    </Section>
  </Body>
});
```

Text-bodied elements (`<Text>`, `<Button>`) accept the same control-flow forms; each branch's parts are concatenated into the surrounding string:

```rust
let template = Template::new(mjml! {
  <Body><Section><Column>
    <Text>
      "Status: "
      @match n {
        0 => { "no items" }
        1 => { "one item" }
        _ => { "many items" }
      }
    </Text>
  </Column></Section></Body>
});
```

## Without the macro: typed builder API

Every element exposes a typed builder with `::new(...)` constructors and chained setter methods. The macro expands to these calls; you can write them directly when the macro's syntax fights what you're doing — e.g. assembling a template from async-awaited intermediate values, or programmatically constructing trees from a configuration source.

```rust
use manteau::prelude::*;

let body = Body::new()
  .background_color(Color::hex(0xf5f5f4))
  .push(
    Section::new()
      .padding_top(Pixels::new(32))
      .push(
        Column::new().push(
          Text::new("Hello, world!")
            .font_weight(FontWeight::Bold)
            .color(Color::hex(0x1c1917)),
        ),
      ),
  );

let _template = Template::new(body);
```

Children are added via the `Push` trait (re-exported in the prelude). `Body` accepts `Section` and `Wrapper`; `Wrapper` accepts `Section`; `Section` accepts `Column`; `Column` accepts anything implementing `Into<Block>` (which covers `Text`, `Button`, `Image`, and any consumer-defined `Element` via `Block::custom`).

## Runnable examples

Three end-to-end examples under `examples/`:

```text
cargo run --example welcome --features stdout
cargo run --example newsletter
cargo run --example builder
```

- **`welcome`** — a styled welcome email with a wrapper card containing a CTA button; rendered via `StdoutTransport`.
- **`newsletter`** — control-flow demo (`@for` over a list, `@if` for conditional sections, `@match` inside `{...}` for variant dispatch); rendered via `MockTransport`.
- **`builder`** — the same content as `welcome` expressed via the typed builder API for comparison.

## Feature flags

- `mailjet` — `MailjetTransport`, posts to Mailjet's v3.1 send API. Implies `tls-rustls`.
- `stdout` — `StdoutTransport`, prints to stdout. For local development.
- `tls-rustls` / `tls-native` — selects the TLS backend for the HTTP transports. `tls-rustls` is the default chosen by `mailjet`.

`MockTransport` is always available and captures sent messages in memory; convenient for tests.

## Custom transports

Implement `Transport` for your own provider. Your error type implements `TransportFailure` so retry middleware can branch on the universal categories (`is_transient`, `is_auth`, `is_message_rejected`, `retry_after`).

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

Implement `Element` and wrap with `Block::custom`. Consumer-defined elements participate in the typed builder (`Column::push(Block::custom(MyDivider))`) but are not recognized by the `mjml!` macro's closed tag set — splice them in via `{expr}`.

```rust
use manteau::prelude::*;
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

The first version of this crate was an internal tool I needed built quickly, which I realized may benefit others. Thus, it was built almost entirely using AI, and hastily published. Because of this, I do not consider the v0.1.0 release well-tested, and while I rather carefully refined the API I wanted, I default to not considering it necessarily well-built.

I will be hand-revising this in the coming days until a properly vetted 0.2.0 release can be produced. At that point, I would consider it feature-complete. If you see this message, I invite you to contribute. I have a deep love for the craft of software design, and would love to partner with others who do as well.

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
