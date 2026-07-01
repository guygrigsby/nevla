# 2. CSP stream processes as the core primitive

Status: accepted (2026-07-01)

## Context

The language needs a reason to exist. General-purpose syntax experiments
don't justify a new language; a concurrency model baked into the core might.

## Decision

Communicating Sequential Processes with implicit channels. `spawn` defines a
stream transducer over `Vein<T>` — not a function: it has a lifetime, reacts
per item (`input -> item`), and may `emit` zero, one, or many values per
input (filter/map/flatMap/scan/window from one primitive). Channels are
unbuffered rendezvous, never wired by hand; `|>` composes stages. Sequential
by default with exclusive access to internal `var` state; parallelism is
explicit (`spawn<N>`, `spawn<auto>`), unordered, and mutable cross-item state
in a parallel process is a compile error. Sources and sinks are structural
(no input param / no output slot).

## Consequences

- Stateful stream operators are safe by default; the type checker carries
  the concurrency safety story, not runtime discipline.
- Unordered parallel output will surprise users who flip `spawn` to
  `spawn<8>`; documented, not mitigated (reordering is future stdlib).
- Buffered channels, fan-out/fan-in, and supervision are deliberately
  excluded from v1 syntax — they must be expressible as libraries later
  (enabled by ADR 0007).
- Spec §4; rationale §5.
