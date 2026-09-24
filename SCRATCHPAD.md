# Deferred work for review

These are requests to remember, not authorization to expand the active 0.2
redesign goal. Finish that goal, then wait for the author's direction on this list.

1. **Macro diagnostics research and design.** Analyze every macro's error
   handling. Aim for specific, intentional messages explaining what went wrong
   and why, with spans pointing to the exact offending code. Research supporting
   crates and how popular macro libraries implement diagnostics. This is broader
   than the active goal's existing hygiene and diagnostic correctness fixes.
2. **Complete MJML coverage.** Analyze how to support every MJML tag and every
   field, with appropriate types enforcing correct use, macro support, and the
   diagnostics described above. Inventory the authoritative MJML specification,
   assess renderer support, and propose implementation and verification before
   starting this expansion.
3. **Runtime-hydrated templates.** Design reusable templates with typed slots
   for text, URLs, headers, and any other supported contexts. The binding API
   should reject missing or mismatched values, escape at the actual MJML sink,
   and keep unbound templates distinct from renderable ones. Measure compilation,
   hydration, and rendering costs before choosing a cache or engine. Do not
   substitute strings into finished MJML or HTML: that loses context. Candidate
   crates: `upon` supports compilation and reusable templates but defaults to
   unescaped output; MiniJinja supports automatic HTML escaping but it depends
   on template naming/context and its language is broader; TinyTemplate is
   small and HTML-escapes values but does not encode Manteau's separate URL and
   attribute contracts. A small structural slot engine may fit better than a
   general text engine. Sources:
   - https://docs.rs/upon/latest/upon/struct.Engine.html
   - https://docs.rs/minijinja/latest/minijinja/struct.Environment.html
   - https://docs.rs/tinytemplate/latest/tinytemplate/
