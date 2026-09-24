# manteau-macros

MJML syntax for typed templates. Part of [Manteau](../README.md).

The `mjml!` procedural macro parses the currently supported MJML subset and emits calls to checked template builders. It resolves a renamed `manteau` dependency at expansion time. Use it through `manteau::mjml`; expression attributes use the builder API.

The supported tags are `Body`, `Wrapper`, `Section`, `Column`, `Text`, `Button`, and `Image`. Compile-time checks cover recognized literal attribute roles; complete MJML coverage is future work.

## Minimal use

```toml
[dependencies]
manteau = "0.2"
```

```rust
use manteau::{mjml, templating::Template};
let template = Template::new(mjml! {
    <Body><Section><Column><Text>"Hello"</Text></Column></Section></Body>
});
let _ = template;
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
