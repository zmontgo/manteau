# Tested usage examples

These files are included by [`tests/usage.rs`](../usage.rs), so `cargo test -p manteau --test usage` compiles and runs them. They have Rust test entry points instead of binary `main` functions because Manteau is a library.

- [`welcome.rs`](welcome.rs) builds MJML with `mjml!`, prepares a message, and captures it locally.
- [`builder.rs`](builder.rs) constructs the same kind of tree through owner methods.
- [`newsletter.rs`](newsletter.rs) shows conditional and repeated content in `mjml!`.
- [`custom_element.rs`](custom_element.rs) extends a typed column with a checked-name MJML element.
- [`custom_renderer.rs`](custom_renderer.rs) implements the rendering port and composes it with `Mailer` and local capture.

The [JetEmail protocol test](../../../manteau-jetemail/tests/protocol.rs) demonstrates persisted `Submission` replay against a local HTTP server, including the same key and serialized body on both attempts. This is a protocol test because the replay contract belongs to an idempotent provider, not to every transport. It never calls the live service.
