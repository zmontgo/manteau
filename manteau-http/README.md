# manteau-http

Bounded HTTP driver. Part of [Manteau](../README.md).

`HttpClient` implements core’s `Http` port with pooled `reqwest` connections, request timeouts, bounded response reads, sensitive authentication headers, and redirects and automatic retries disabled. Protocol and acceptance decisions remain in core and provider crates.

Select `tls-rustls` or `tls-native` when depending on this package directly. The facade enables `tls-rustls` by default.

## Minimal use

```toml
[dependencies]
manteau-http = { version = "0.2", features = ["tls-rustls"] }
```

```rust
use manteau_http::HttpClient;
let http = HttpClient::new().unwrap();
let _ = http; // Reuse this client across core Sender requests.
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
