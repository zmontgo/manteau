# manteau-mock

Local capture transport. Part of [Manteau](../README.md).

`MockTransport` implements core `Transport` by storing immutable prepared messages in memory. `sent()` snapshots captures; `take_sent()` drains them. A `MockReceipt` proves local capture only and has no provider message IDs.

Use `Mailer::new(renderer, MockTransport::new())` for tests and previews. Enable the facade’s `mock` feature for its re-export.

## Minimal use

```toml
[dependencies]
manteau-mock = "0.2"
```

```rust
use manteau_mock::MockTransport;
let capture = MockTransport::new();
assert!(capture.sent().is_empty());
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
