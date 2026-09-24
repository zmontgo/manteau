# manteau-render

MJML rendering adapter. Part of [Manteau](../README.md).

`MrmlRenderer` implements the core `Renderer` port using `mrml` for HTML and `html2text` for plaintext. It renders supplied structured templates without fetching remote content or resolving includes.

Use `MrmlRenderer::new()` with `Mailer::new(renderer, transport)`, or enable the facade’s default `render-mrml` feature.

## Minimal use

```toml
[dependencies]
manteau-core = "0.2"
manteau-render = "0.2"
```

```rust
use manteau_core::templating::{Body, Column, Push, Section, Template, Text};
use manteau_render::MrmlRenderer;
let template = Template::new(
    Body::new().push(Section::new().push(Column::new().push(Text::new("Hello")))),
);
let rendered = template.render(&MrmlRenderer::new()).unwrap();
assert!(!rendered.html().is_empty());
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
