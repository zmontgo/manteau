# manteau-mailjet

Mailjet protocol adapter. Part of [Manteau](../README.md).

`Mailjet` implements `HttpProvider` for Mailjet’s Send API. It admits a prepared message, encodes the provider payload, and interprets provider response evidence for all To, Cc, and Bcc recipients. It performs no network I/O.

Pair `Mailjet::new(key, secret)?` with `Sender::new(provider, HttpClient::new()?)`, or enable the `mailjet` facade feature and use `Mailer::with_http(renderer, provider)`.

## Minimal use

```toml
[dependencies]
manteau-core = "0.2"
manteau-http = { version = "0.2", features = ["tls-rustls"] }
manteau-mailjet = "0.2"
```

```rust
use manteau_core::Sender;
use manteau_http::HttpClient;
use manteau_mailjet::Mailjet;
let sender = Sender::new(Mailjet::new("key", "secret").unwrap(), HttpClient::new().unwrap());
let _ = sender; // Supply a PreparedMessage to Transport::send.
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
