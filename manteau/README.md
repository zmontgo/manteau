# Manteau

Manteau is a Rust library for authoring and submitting transactional email. It gives email structure, addresses, links, dimensions, rendered bodies, and provider results distinct types. You can compose an MJML template with typed builders or the `mjml!` macro, render it through a selected renderer, and submit one prepared message through a selected transport.

This is an unstable `0.x` API. Version `0.2` changes public APIs and package layout; see [MIGRATING.md](../MIGRATING.md). No `1.x` release is planned before a manual API review.

## Start with local capture

```toml
[dependencies]
manteau = { version = "0.2", features = ["mock"] }
tokio = { version = "1", features = ["macros", "rt"] }
```

```rust
use manteau::{Address, Envelope, HeaderText, Mailer, Message, MockTransport,
              MrmlRenderer, Recipients, templating::{Body, Column, Push,
              Section, Template, Text}};

#[tokio::test]
async fn compose_and_capture() -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::new(
        Body::new().push(
            Section::new().push(Column::new().push(Text::new("Hello!"))),
        ),
    );
    let envelope = Envelope::new(
        Address::new("sender@example.com".parse()?),
        Recipients::to(Address::new("reader@example.com".parse()?)),
        HeaderText::new("Welcome")?,
    );
    let mailer = Mailer::new(MrmlRenderer::new(), MockTransport::new());
    mailer.send(Message::new(envelope, template)).await?;

    assert_eq!(mailer.transport().sent().len(), 1);
    Ok(())
}
```

`MockTransport` captures immutable prepared messages. It does not report provider acceptance or delivery. [The tested examples](tests/examples/README.md) also show `mjml!`, builder composition, and conditional content.

## Choose the capabilities you need

| Package | Role |
| --- | --- |
| [`manteau`](../manteau) | Consumer API: message intent, `Mailer`, selected re-exports, and optional standard assembly |
| [`manteau-core`](../manteau-core) | Checked values, template structure, rendering and submission ports, and shared submission mechanics |
| [`manteau-macros`](../manteau-macros) | The `mjml!` syntax; normally imported through `manteau` |
| [`manteau-render`](../manteau-render) | MJML/HTML and plaintext rendering through `mrml` and `html2text` |
| [`manteau-http`](../manteau-http) | Bounded pooled HTTP I/O through `reqwest` |
| [`manteau-mailjet`](../manteau-mailjet), [`manteau-cloudflare`](../manteau-cloudflare), [`manteau-jetemail`](../manteau-jetemail) | Provider-specific request admission, payloads, and response evidence |
| [`manteau-mock`](../manteau-mock), [`manteau-stdout`](../manteau-stdout) | Local capture and explicit local output |

The facade enables the `render-mrml` and `tls-rustls` features by default. Provider features (`mailjet`, `cloudflare`, `jetemail`) and local transport features (`mock`, `stdout`) are opt-in. A provider is a protocol capability, not a client: pair it with `Sender<P, H>` and an HTTP implementation, or use `Mailer::with_http(renderer, provider)` when a provider feature is enabled. The HTTP driver never chooses provider policy. See [ARCHITECTURE.md](../ARCHITECTURE.md) for ownership and replay boundaries.

## Template syntax and validation

`mjml!` currently recognizes `Body`, `Wrapper`, `Section`, `Column`, `Text`, `Button`, and `Image`. Other MJML elements are not yet implemented. Literal color, URL, font-family, alignment, weight, transform, line-height, and measurement attributes are checked during macro expansion where their role is known. Braced values are ordinary Rust expressions and pass through the typed builder API.

```rust
# use manteau::{mjml, templating::Template};
let template = Template::new(mjml! {
    <Body>
        <Section>
            <Column>
                <Text color="rebeccapurple" font-size="18px">"Hello"</Text>
                <Button href="https://example.com/account">"Open account"</Button>
            </Column>
        </Section>
    </Body>
});
# let _ = template;
```

A `Template` is a structured MJML document. It does not yet hydrate placeholders from persisted data; macro interpolation evaluates Rust expressions when the tree is built. Rendering returns checked, nonempty HTML and plaintext bodies, but this is **not** a sanitizer or an XSS-safety claim. The renderer does not fetch remote resources. Validate untrusted content before placing it in trusted custom elements.

## Submission and recovery

`Mailer::send` renders once and submits once. A failure retains either the original `Message` or the exact `PreparedMessage`; inspect the transport error's acceptance evidence before retrying. For durable work, call `Mailer::prepare`, persist the prepared message, then submit through `Mailer::transport`. Only transports implementing `IdempotentTransport` can use `Submission` with a persisted key, message, scope, and first-attempt time. Provider acceptance is not delivery, and an ambiguous network failure is not proof of rejection.

Manteau does not schedule retries, provide a database, or send a live email during its tests. Provider protocol tests use local HTTP servers.

License: MIT or Apache-2.0.
