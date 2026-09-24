# Architecture

Manteau separates the decision to send a particular email from the physical act of talking to a provider. The primary flow is:

```text
Message (authoring intent) + Renderer
    -> PreparedMessage (immutable envelope and checked HTML/plaintext)
    -> Transport or Sender<Provider, Http>
    -> provider receipt or failure with acceptance evidence
```

`manteau-core` owns values and the ports it needs: `Renderer`, `Transport`, `Http`, and `HttpProvider`. It composes the reusable mechanics of preparation, one HTTP dispatch, response bounds, and idempotent replay checks from those ports alone. It does not select `mrml`, `reqwest`, or a provider. This is dependency inversion at the capability boundary, not an attempt to make every type generic.

`manteau` owns developer-facing message intent and the configured `Mailer<R, T>`. It renders once, sends once, and returns the exact prepared content after a submission failure. A caller that needs durable delivery can prepare and persist content before dispatch. The main crate optionally assembles a provider with the standard HTTP driver. It does not schedule retries or claim delivery.

`manteau-render` implements rendering through `mrml` and `html2text`. `manteau-http` implements physical HTTP through `reqwest`: pooled connections, timeouts, bounded reads, TLS selection, and no redirects or automatic retries. Provider crates implement `HttpProvider`—their request admission, wire shape, documented status policy, and response evidence. `manteau-jetemail` also implements `IdempotentProvider`; that supertrait requirement means keyed submission is available only for a provider that declares deduplication retention. `manteau-mock` and `manteau-stdout` are direct `Transport` implementations and do not inherit HTTP concerns.

A provider receipt means what that provider's response proves. `Acceptance::Unknown` is distinct from `NotAccepted`. A mixed-recipient outcome must not be flattened into a whole-envelope retry. HTTP transport errors can happen after a provider accepts a request. Retrying those requires a durable `Submission` bound to the same key, exact prepared message, provider account and routing scope, serialization version, and retention window. Persist this value before the first attempt. An endpoint and credential hash cannot prove an unchanged provider region; callers must keep routing consistent.

Most public data fields are private. Core-owned constructors establish contracts for addresses, headers, URLs, dimensions, JSON bodies, bounded responses, and rendered output. `HtmlBody` guarantees nonempty output without disallowed control characters; it does not guarantee sanitized HTML. Adapter extension points must uphold their documented contracts. The macro compiles syntax to the same checked builders that handwritten Rust calls.

Provider adapters may depend on core and pure encoding dependencies. The main crate may depend on adapters for optional assembly. Core must not depend on any concrete adapter; provider crates must not depend on the main crate or HTTP driver in normal dependencies. Tests can assemble both sides of a port without changing production dependency direction.
