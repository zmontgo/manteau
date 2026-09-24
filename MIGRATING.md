# Migrating from 0.1 to 0.2

`0.2` is an intentionally breaking minor release while Manteau remains unstable. Review your call sites; mechanical import replacements alone are insufficient.

- The repository is a workspace. Import the consumer API from `manteau`; use `manteau-core` when implementing a port, `manteau-render` for the concrete renderer, `manteau-http` for physical HTTP, and a provider package for its protocol. Provider features on `manteau` are optional.
- Construct an `Envelope` from a sender `Address`, nonempty `Recipients`, and checked `HeaderText`. A `Message` combines that envelope with a `Template`; it is authoring intent, not the object a provider sends.
- Call `Message::prepare(&renderer)` or use `Mailer::new(renderer, transport)`. Delivery transports receive an immutable `PreparedMessage` containing checked rendered body alternatives. A failed `Mailer::send` retains the original `Message` or exact `PreparedMessage` in `MailerError`.
- Provider-specific transport constructors have been replaced by protocol types (`Mailjet`, `Cloudflare`, `JetEmail`) and core `Sender<P, H>`. With a provider feature, `Mailer::with_http(renderer, provider)` assembles the standard pooled HTTP driver. Provider adapters do not create their own HTTP clients.
- Idempotency is a separate capability. `IdempotentProvider: HttpProvider` and `IdempotentTransport: Transport` expose keyed behavior only where the provider contract supports it. Persist the complete `Submission` before the first request, and retain the exact key and prepared message for replay.
- Invariant-bearing fields are private. Use checked constructors and inspection methods for templates, bodies, URLs, colors, dimensions, headers, and response values. `Url::try_parse`, `ImageUrl::try_parse`, and `Color::try_parse` return errors instead of accepting arbitrary strings.
- The `mjml!` macro still supports a focused set of tags. Literal color, link, image source, font family, alignment, weight, transform, line-height, and dimensions follow their checked builder contracts. Renaming the `manteau` dependency works.

No 0.2 API promises provider delivery merely because a request was accepted. Inspect provider receipt and acceptance evidence separately. See the [quickstart](README.md) and [architecture guide](ARCHITECTURE.md).
