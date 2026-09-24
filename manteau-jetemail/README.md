# manteau-jetemail

JetEmail protocol adapter. Part of [Manteau](../README.md).

`JetEmail` implements the provider protocol and `IdempotentProvider`. It owns payload mapping, response interpretation, and documented idempotency retention; core owns common HTTP dispatch and replay checks. It performs no network I/O.

Use the `jetemail` facade feature for standard HTTP assembly. Persist a whole `Submission` before a keyed attempt and keep the same routing region throughout replay.

## Minimal use

```toml
[dependencies]
manteau-core = "0.2"
manteau-http = { version = "0.2", features = ["tls-rustls"] }
manteau-jetemail = "0.2"
```

```rust
use manteau_core::Sender;
use manteau_http::HttpClient;
use manteau_jetemail::JetEmail;
let sender = Sender::new(JetEmail::new("token").unwrap(), HttpClient::new().unwrap());
let _ = sender; // Supports IdempotentTransport when replay is required.
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
