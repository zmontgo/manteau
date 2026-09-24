# manteau-core

Checked email values and ports. Part of [Manteau](../README.md).

Use this crate when implementing a renderer, delivery transport, HTTP driver, or provider protocol. It owns validated envelopes, immutable prepared messages, MJML structure, rendering contracts, HTTP requests/responses, and the shared `Sender` procedure. It does not depend on concrete providers, `reqwest`, or a renderer implementation.

`manteau_core::{Renderer, Transport, HttpProvider, Sender}` are extension points; see [`manteau`](../manteau/README.md) for application composition.

## Minimal use

```toml
[dependencies]
manteau-core = "0.2"
```

```rust
use manteau_core::templating::{Body, Column, Push, Section, Template, Text};
let template = Template::new(
    Body::new().push(Section::new().push(Column::new().push(Text::new("Hello")))),
);
let _ = template;
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
