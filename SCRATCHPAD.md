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
