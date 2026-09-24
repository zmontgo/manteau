# Working in Manteau

Read this file and the current `TODO.md` before changing code. `SCRATCHPAD.md` holds deferred ideas; do not fold them into the active goal without the user's direction. Use `jj` for version control. Each numbered TODO block belongs in its own described jj change, followed by `jj new`.

Manteau is a published Rust library for authoring and submitting email. It remains `0.x` until the author manually reviews its public API. Treat every exposed type, feature, README example, and doc comment as part of the product. There are no production binary entry points. Procedural macros and Rust test entry points are exceptions to the preference for methods over bare functions.

## Development philosophy

The type knows. Data owners answer questions about their data and own its permitted actions. Prefer private fields and checked constructors. Make invalid states structurally unavailable when the contract can express that. Once a boundary validates a value, trust that boundary instead of repeating checks inside the system.

Structure over discipline. A compiler-enforced type or privilege boundary ages better than a reminder in a comment. Ownership signatures communicate intent: borrow for inspection and consume for a commit where that distinction matters.

The lazy principle means minimizing future cognitive load through current research. Ask whether an abstraction should exist before polishing it. Prefer mature crates over hand-rolled mechanisms when the crate's contract actually fits. Cut and wrap duplication into the right owner instead of adding call-site helpers.

Contract shape determines the API. Expose primitive methods when the owner promises primitives; provide a compound operation when atomicity or invariant preservation is the promise. Traits need real substitution and must remove duplication from callers. Do not add a trait solely to document one type's methods.

Vocabulary is infrastructure. Name a type for what it is, not for a caller's use. Keep business terms consistent across crates, rustdoc, and guides. Explain code systems-first: purpose, reason, dependents, then mechanics. Comments should defend a constraint or decision that types cannot express, not narrate each statement.

Everything is suspect, including prior generated code and this guide. Surface uncertainties. If the correct shape is unclear, research it and mark the decision point instead of shipping a plausible half-measure.

## Dependency direction

`manteau-core` defines checked shared values, ports, and reusable preparation/submission mechanics. `manteau` owns consumer message intent, rendering and transport composition, and optional standard assembly. Provider crates know their own request mapping and response evidence; the HTTP crate handles physical I/O; the renderer crate handles concrete rendering. Provider adapters do not own the shared send procedure or select application retry policy. Keep core free of concrete adapter dependencies.

Differentiate preparation, submission, provider acceptance, and delivery. A network error can leave acceptance unknown. A retry is only safe when evidence or an idempotent provider contract makes it safe. Preserve the exact prepared message for recovery and do not leak credentials or private mail through errors and traces.

Public fields of invariant-bearing types should be private. Give consumers owner methods for legitimate inspection and construction. `#![deny(missing_docs)]` applies to every library crate. Doc comments explain type purpose and guarantees, while guides show complete, tested usage. Space code for reading: blank lines between top-level definitions and between distinct steps inside functions; expand compressed control flow.

The repository root [ARCHITECTURE.md](ARCHITECTURE.md) records current boundaries, and [MIGRATING.md](MIGRATING.md) records breaking 0.2 changes. For local verification use the workspace checks in CI; protocol tests use local loopback servers, never live provider delivery.
