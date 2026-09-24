# manteau-cloudflare

Cloudflare Email Routing adapter. Part of [Manteau](../README.md).

`Cloudflare` implements `HttpProvider` for its configured endpoint. It owns provider-specific header mapping and recipient outcome interpretation, including mixed outcomes. It performs no network I/O.

Pair the provider with core `Sender` and an `Http` driver; the `cloudflare` facade feature supplies standard assembly.

## Minimal use

```toml
[dependencies]
manteau-core = "0.2"
manteau-http = { version = "0.2", features = ["tls-rustls"] }
manteau-cloudflare = "0.2"
```

```rust
use manteau_core::Sender;
use manteau_http::HttpClient;
use manteau_cloudflare::Cloudflare;
let sender = Sender::new(Cloudflare::new("account-id", "token").unwrap(), HttpClient::new().unwrap());
let _ = sender;
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
