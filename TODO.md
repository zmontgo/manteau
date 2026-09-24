# Manteau 0.2 work

The active goal is the complete 0.2 redesign. Every numbered block is its own
jj change: finish its checks, update this list, write a succinct description,
then run `jj new`. Do not combine later blocks to preserve a green build.
`SCRATCHPAD.md` is deferred work, not part of this goal.

## Responsibility map

| Owner | Responsibility | Why it lives here |
| --- | --- | --- |
| `manteau-core` | Validated shared values, template structure, ports, and reusable preparation/submission operations expressed through those ports | The authoritative contracts and mechanisms must be usable without selecting a concrete adapter. |
| `manteau` | Consumer-facing message intent, configured preparation/send workflows, recovery preserving prepared content, and optional standard assembly | This is where developers compose capabilities into the library's email workflow. It must provide behavior beyond re-exports. |
| `manteau-mailjet`, `manteau-cloudflare`, `manteau-jetemail` | Provider-specific admission, wire representations, response evidence, and documented provider capabilities | Only the provider knows its protocol. Shared dispatch belongs in core; physical I/O belongs in the HTTP adapter. |
| `manteau-http` | HTTP port implementation: connections, TLS, authentication headers, bounded reads, timeout, redirect/retry controls | These are physical transport mechanisms shared by the provider implementations. |
| `manteau-render` | Rendering port implementation using mrml/html2text | Concrete rendering libraries must not determine core's dependency graph. |
| `manteau-mock`, `manteau-stdout` | Direct transport implementations for capture and local output | These capabilities do not use an HTTP protocol and should not inherit its machinery. |
| `manteau-macros` | Compile-time syntax translated into calls to authoritative checked APIs | Generated code must preserve the same contracts as handwritten builders. |

Production dependency direction: provider/physical/rendering adapters depend on
core; core depends on none of them; the main crate may optionally depend on them
for assembly. Macros must not depend on the main crate in a way that creates a
cycle. Tests may assemble both sides of a port without changing that direction.

Toasty supplies the dependency-inversion example, not a module-by-module recipe:
its core includes shared driver/schema contracts and capability-based schema
construction; its main crate owns database composition and query execution
planning. Manteau should draw the equivalent boundary around email preparation
and submission, without inventing a query-engine-shaped layer.

## Completed changes

- [x] 1. **Workspace split** — virtual root `Cargo.toml`; sibling `manteau`,
  `manteau-core`, `manteau-macros`, and five adapter packages. Shared 0.2.0
  metadata and local package dependencies. Change: `ympmksux`.
  Verified: workspace all-feature library check.
- [x] 2. **Prepared-message boundary** — core `models.rs`, `message.rs`,
  `idempotency.rs`, and `transport.rs`: checked headers, nonempty recipients,
  neutral envelopes, immutable shared bodies, and explicit acceptance facts.
  Migrated every adapter to prepared input and examples to test entry points.
  Change: `wxolsyps`. Verified: workspace all-target/all-feature check, 48
  core/facade/mock tests and 47 adapter tests with native TLS. This was an
  intermediate boundary; the responsibility corrections below supersede its
  concrete-renderer placement and provider-owned send procedures.

## Remaining change blocks

- [x] 3. **Core-owned HTTP submission mechanism and ports** — `yxosryop`.
  - `manteau-core/src/http/` (split current `http.rs` by meaningful owners):
    validated endpoint/configuration, redacted credentials, admitted request,
    bounded response, and the physical `Http` port. No reqwest dependency.
  - `manteau-core/src/sender.rs`: `HttpProvider` port and configured `Sender<P,H>`.
    Core owns the one request procedure: admit provider-specific input, encode
    the payload, dispatch through `Http`, obtain provider interpretation, retain
    evidence, and return a typed result. Provider request admission must be a
    single operation, not separately callable validation and unchecked encoding.
  - Distinguish physical HTTP facts from provider acceptance evidence. Remove
    hard-coded 409 exemptions and universal assumptions that a status proves
    non-acceptance. Provider interpretations supply the evidence; core handles it.
  - `manteau-core/src/idempotency.rs`: bind key, immutable message, provider
    scope, and first-attempt time in one persisted submission. Enforce scope and
    retention before dispatch. Preserve replay error detail. Document trusted
    persistence, clock assumptions, and cancellation uncertainty.
  - `manteau-http/src/lib.rs`: implement only core's HTTP port using reqwest;
    own connection pooling, TLS, sensitive headers, total timeout, response size,
    and no redirects/automatic retries. No provider status policy or mail logic.
  - Migrate existing provider adapters enough to use the new ports; remove their
    duplicate send procedures and production dependency on the HTTP adapter.
  - Verify core composition with in-memory port implementations and physical
    driver behavior against loopback servers; prove the dependency direction.
  - Replace stale provider-transport tests with assemblies of provider + Sender
    + HTTP driver. Check admission failures dispatch zero requests and ordinary
    submissions dispatch exactly one. Fix facade exports to avoid colliding
    provider payload names.
  - Audit replay identity beyond credentials and URL: document provider region
    constraints and payload-version stability. Do not claim the scope hash alone
    proves either. Reject unsupported replay conditions before dispatch.

- [x] 4. **Provider-specific implementations and protocol evidence** — `xrtuunmy`.
  - `manteau-jetemail/src/`: a configured JetEmail type implementing core's
    provider ports, its request/response values, documented envelope restrictions,
    queue acceptance and idempotency-conflict interpretation. No header assembly,
    HTTP client, retry procedure, or generic response-reading implementation.
  - `manteau-mailjet/src/`: provider request mapping and strict response evidence
    for Status and all To/Cc/Bcc recipients. Reject malformed or contradictory
    success; preserve real IDs and recipient association.
  - `manteau-cloudflare/src/`: provider header encoding, request mapping, real
    message IDs, complete delivered/queued/bounced/suppressed recipient report.
    Preserve mixed outcomes without allowing whole-envelope blind retry.
  - Provider crates depend on core plus necessary pure encoding libraries, not
    each other, `manteau`, reqwest, or `manteau-http` in normal dependencies.
  - Keep mock/stdout as direct implementations of the core Transport port: they
    do not pretend to be HTTP providers or manufacture provider IDs.
  - `manteau-*/tests/`: malformed success, unexpected recipients, mixed outcomes,
    response-size/time bounds, redirects, errors containing private text, stable
    keyed payloads, expired and changed-scope replay, Unicode headers, and no
    unintended second physical request. Use official provider documentation;
    mocks verify our implementation, not the external service's guarantees.

- [x] 5. **Rendering inversion and reusable core operations** — `swttrouk`.
  - `manteau-core/src/render/`: define the rendering port and output/error
    contracts; retain template-to-MJML generation, escaping, and reusable
    preparation mechanics against ports. No mrml/html2text dependency or
    implicit concrete renderer selection.
  - `manteau-render/src/`: concrete rendering adapter implementing the port with
    mrml/html2text, configured plaintext width, and preserved source errors.
    Rendering must not fetch remote resources or silently resolve includes.
  - Keep authoritative typed template values in core; do not duplicate a raw
    tree and a parallel set of typed facade wrappers just to copy Toasty's shape.
  - Test rendering composition with a deterministic port implementation; test
    concrete escaping/rendering behavior in the rendering adapter. Verify core
    builds without either rendering implementation dependency.
  - Justify each port against actual substitution and caller duplication; do
    not add ceremony or artificial methods to make a narrow port look deep.
    Record where the explicit inversion requirement calls for a narrow port.

- [x] 6. **Main-crate consumer behavior and configured composition** — current change.
  - Strengthen the rendering boundary as part of this composition: the
    `Renderer` port returns core-owned checked HTML/plaintext values, and main
    consumes those values without repeating validation. Name the exact promise
    (for example nonempty HTML output and disallowed control characters);
    never equate “renderer output” with XSS-safe HTML by assertion alone.
  - `manteau/src/message.rs`: consumer intent combines a validated envelope,
    template, and optional plaintext alternative. Adapters only receive core's
    immutable PreparedMessage; they do not depend on this authoring type.
  - `manteau/src/mailer.rs`: a configured service owns supplied renderer and
    transport capabilities. Compose preparation and one send; preserve prepared
    content on submission failure so recovery never requires rerendering.
  - Provide a deliberate prepare/persist/submit path for durable callers, using
    the same core mechanisms. No scheduler, automatic retry, provider failover,
    database, or promise of eventual delivery.
  - `manteau/src/lib.rs`, `prelude.rs`, and feature wiring: ergonomic standard
    assembly, selected re-exports, optional provider and rendering adapters, and
    direct access to underlying configured capabilities when needed.
  - Avoid forwarding-only wrappers and inventing business workflows to make the
    main crate larger. Its domain is authoring/preparing/submitting email.
  - Test custom renderer/transport composition, exactly one render per prepared
    message, failed-send recovery retaining identical content, and mock usage.

- [x] 7. **Value contracts, ownership, and extension boundaries** — current change.
  - Audit public fields across every workspace crate, including `Body`, all
    MJML elements, envelopes, provider models, and configuration. Make
    invariant-bearing state private; provide owner methods for legitimate
    inspection and changes. Direct field mutation must be justified by the
    type's actual contract, not convenience. Check serde and macro construction
    paths as alternate constructors.
  - Audit every port argument and result (`Renderer`, `Http`, `HttpProvider`,
    `Transport`) for primitive or generic values whose meaning makes callers
    recheck them. Introduce owner types with private state and checked
    construction for JSON request bodies, response status/evidence, bounded
    responses, HTML/plaintext, and provider receipt facts where the contract
    warrants it. Remove generic parameters that only hide an unconstrained
    string or serialization contract. Test deserialization and extension
    boundaries as well as normal constructors.
  - Evaluate `safehtml` 0.2's `SafeHtml`/`SafeUrl` for author-controlled HTML
    fragments and URLs. Its safety contract cannot be attached to complete
    MJML-rendered email until every custom element, style, URL, and unchecked
    conversion path satisfies it. Consider `ammonia` for explicitly sanitized
    snippets; its default policy changes email layout, so it is not a blanket
    renderer-output validator. Document the selected contracts and tradeoffs.
  - `manteau-core/src/templating/attributes/`: finite numeric values, field-valid
    dimensions and URL roles, truthful color validation, checked construction,
    and intentional conversion APIs. Keep typed builders discoverable.
  - `manteau-core/src/templating/` and `render/writer.rs`: private invariant-bearing
    state, typed nesting, validated markup names, escaped content, and accurate
    custom-element obligations. No claim that arbitrary downstream implementations
    are statically proven correct.
  - Compile-fail tests for invalid construction/nesting/mutation and runtime
    boundary tests for untrusted strings and persistence. Do not add MJML tags
    from the deferred full-coverage request.
  - Decision: keep `HttpProvider::Payload` as a provider-owned concrete
    serialization type because core's only operation on it is JSON encoding;
    the resulting `JsonBody` and bounded response are core-owned checked
    values. `HtmlBody` promises nonempty, control-safe email markup, not
    sanitized HTML. `safehtml::SafeHtml` is suitable to evaluate for trusted
    HTML fragments at a future insertion boundary; its stronger XSS contract
    would be false for arbitrary MJML-rendered output. `SafeUrl` is a useful
    candidate for links, while Manteau currently needs its narrower email-link
    and image-source policies.

- [x] 8. **Test helper ownership** — current change.
  - Replace bare fixture/helper functions in `manteau/tests/`, core tests,
    and provider protocol tests with meaningful owner methods or local test
    bodies. Keep `#[test]` and `#[tokio::test]` entry points as the explicit
    exception. Avoid empty namespace structs and new traits used only to
    disguise a helper. Keep tests focused on behavior, not wrapper mechanics.
  - Confirm all non-procedural-macro production source has no bare functions
    and no binary `main` entry points.

- [x] 9. **Existing macro hygiene and contract parity** — `suolnrxn`.
  - `manteau-macros/src/`: resolve renamed consumer dependencies, preserve spans,
    and generate calls to the authoritative checked types. Fix existing literal
    validation drift and ensure no expansion-time accepted input can panic during
    runtime construction. Keep the already supported grammar coherent.
  - `manteau/tests/` and consumer fixture crates: renamed imports, macro/builder
    equivalence, invalid literals/nesting, and useful existing error locations.
  - Broader diagnostics research and complete tag/attribute coverage remain in
    SCRATCHPAD.md; do not incorporate them into this block.
  - Verified renamed consumer fixture with an independent manifest, macro
    integration tests, and macro compilation. Attribute-role parsing now uses
    core's checked color/URL/font contracts and emits typed enum literals;
    duplicate attributes fail at the duplicate name.

- [x] 10. **Published-library documentation and examples** — `tzrssksu`.
  - Add `#![deny(missing_docs)]` to every library crate and document each
    public crate, module, type, variant, field, method, and extension point to
    a useful depth. Treat the lint as a release gate, not a local exception.
  - Root and every package README: purpose, dependency/feature selection, minimal
    complete usage, links to focused guides, and accurate scope/guarantees.
  - Rustdoc throughout: type purpose, constructor invariants, operation effects,
    errors, privacy, recovery obligations, and intentional extension boundaries.
    Remove stale implementation history, untested claims, and narration.
  - `manteau/tests/examples/` plus included documentation: focused tested examples
    for builders, macros, custom elements, custom ports, sending, capture, and
    persisted replay. No binary entry points or examples teaching secret logging.
  - `MIGRATING.md`: breaking 0.1-to-0.2 API changes and import/package migration.
    `ARCHITECTURE.md`: actual dependency inversion and ownership after code lands.
  - Rewrite ignored AGENTS.md for this library; remove unrelated project
    references from all current working-tree code and documentation.
  - Space source and examples for reading: blank lines between top-level
    definitions and between distinct steps inside functions; expand compressed
    one-line control flow and expressions. Keep tightly related short definitions
    together only where that improves comprehension. Run rustfmt after edits;
    formatting alone does not provide the requested visual grouping.
  - Verified workspace all-feature rustdoc, tested examples, and all-feature
    workspace tests including local provider protocol servers. The tested
    JetEmail protocol case is linked as the durable replay example.

- [ ] 11. **Release verification and measured optimization**.
  - Root manifests/Cargo.lock: consistent 0.2 versions, workspace dependency
    graph, minimal production dependencies, correct independent package metadata.
  - `.github/workflows/ci.yml`: workspace and independent-package checks, supported
    feature/TLS combinations, compiler-version check, unit/protocol/compile-fail
    tests, doctests, formatting, clippy, rustdoc, and package-content verification.
  - Verify the declared minimum Rust version against resolved dependencies; do
    not claim support based on a newer compiler passing.
  - Measure preparation/reuse and allocation-relevant changes with representative
    messages; preserve cheap immutable sharing and connection reuse. Add no cache
    or speculative optimization without evidence.
  - Inspect publishable artifacts and license/readme inclusion without publishing.
    Record checks actually run and external verification limits in the handoff.
  - Completion requires passing migrated integration tests, not just library
    compilation. Verify default, no-default, provider-selected, and all-feature
    configurations; run protocol tests with their required loopback permission.
  - Final source/ownership/dependency scan, succinct jj description, `jj new`,
    complete the goal, then wait for direction on SCRATCHPAD.md.

## Operating constraints

- Breaking release 0.2.0; 1.0 requires the author's manual review.
- Methods belong to meaningful types; procedural macros and Rust tests are exceptions.
- No provider implementation enters core; core owns the contracts its services need.
- No adapter chooses cross-provider or application policy.
- Acceptance, delivery, temporary failure, and safe replay are distinct facts.
- Diagnostic formatting excludes credentials and private mail.
- Live provider delivery is not authorized by tests; use local protocol servers.
- Local protocol tests require sandbox escalation for loopback binding. The host
  /tmp quota interfered with aws-lc compilation; use target/tmp for compiler temps.
