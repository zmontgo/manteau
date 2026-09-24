# manteau-stdout

Explicit local output transport. Part of [Manteau](../README.md).

`StdoutTransport` implements core `Transport` by writing recipients, subject, and body preview to standard output. Verbose mode writes complete HTML and plaintext bodies. Its receipt means local output, not provider acceptance.

Output includes private mail data, including Bcc addresses. Use it only where stdout is an intended destination for that information.

## Minimal use

```toml
[dependencies]
manteau-stdout = "0.2"
```

```rust
use manteau_stdout::StdoutTransport;
let local_output = StdoutTransport::new().verbose(false);
let _ = local_output;
```

See the [architecture guide](../ARCHITECTURE.md) and [migration guide](../MIGRATING.md).
