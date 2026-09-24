# Manteau 0.2 work

Each block is a separate jj change. Complete its checks, update this list,
write a succinct `jj describe`, then run `jj new` before the next block.

- [x] 1. Split the repository into root-level core, facade, macros, and adapter packages; centralize metadata and set version 0.2.0.
- [x] 2. Establish validated envelopes, immutable prepared messages, meaningful rendering owners, and truthful transport outcome contracts; migrate all adapters and callers.
- [ ] 3. Enforce HTTP credential, endpoint, timeout, response-size, redirect, retry, and provider-response guarantees; add adversarial protocol tests.
- [ ] 4. Strengthen template value types and extension boundaries; remove unowned functions; fix macro hygiene and diagnostic parity; add compile-fail coverage.
- [ ] 5. Polish public API documentation, crate READMEs, tested examples, migration notes, and repository guidance; remove unrelated project references.
- [ ] 6. Verify independent packages, supported features and compiler version, docs, tests, formatting, lints, and publishable artifacts; record measured optimization results and remaining external limitations.

## Contract decisions

- This is a breaking 0.2.0 release; 1.0 requires the author's manual review.
- Methods belong to meaningful types. Procedural macros and Rust test entry
  points are exceptions. Examples will be tested library-style demonstrations.
- Rendering and envelope validation precede transport submission. Prepared
  messages retain exactly the submitted content and support cheap immutable sharing.
- Provider acceptance is distinct from delivery; temporary failure is distinct
  from safe retry. Uncertain outcomes remain explicit.
- Provider-specific restrictions belong to adapters. Core envelopes support
  To, Cc, and Bcc with at least one recipient overall.
- Formatting diagnostics must not disclose credentials or private mail.

## Verification by block

1. `cargo check --workspace --all-features --lib --offline` passed. Adapter
   protocol tests and consumer examples will migrate with the contract block.

2. Workspace all-target/all-feature check passed; 48 core/facade/mock tests
   and 47 adapter unit/protocol tests passed (native TLS). Protocol servers
   require execution outside the sandbox. The all-feature test build hit the
   host temporary-file quota while compiling aws-lc; final verification will
   use a workspace-local temporary directory.
