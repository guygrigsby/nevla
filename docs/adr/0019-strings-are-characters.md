# 19. Strings are character sequences, not bytes

Status: accepted 2026-07-10 (recording a v1 decision after the fact)

## Context

nevla adopted CPython's string model from day one: a `str` is a
sequence of Unicode characters, `s[i]` yields a one-character `str`,
`len` counts characters, and there is no byte string or rune type. Go
deliberately chose the other lane (strings are bytes, `rune` for code
points, iteration decodes UTF-8). ADR 0013 requires Go deviations to
be recorded with a reason; this one never was. The Python-leak audit
flagged the missing record, not the design.

## Decision

Keep the character model, now on purpose:

- For scripts and ML glue, characters are what users mean; byte-level
  string surgery is rare and the py bridge or a future `bytes` type
  can own it when it becomes real.
- One string type with one indexing meaning keeps the no-nil,
  no-crash story simple: no silent byte-splitting of a code point.
- The cost is accepted: indexing is O(n) or the runtime pays for an
  index, and byte-precise protocols need something else eventually.

The character toolkit around the model is nevla-named, not
Python-named (ADR 0017): `char(n int) str` and `charcode(c str) int`
rather than `chr`/`ord`.

## Consequences

- A `bytes` type is future work with its own ADR if demand arrives;
  it will not change `str` semantics.
- The spec's sections 3.2, 5.1, and 7.5 are the normative home of the
  model; this ADR is why.
