# 3. Process-scoped mortality; the program cannot crash

Status: accepted (2026-07-01)

## Context

Languages with panic/exceptions have invisible control flow: any call can
blow up the program unless someone remembered to catch. The design goals:
signatures tell the whole truth about failure, and modern software shouldn't
crash. Full panic removal was examined and fails on built-ins — the total
alternatives are Pony's `1/0 == 0` (silent corruption in a dataflow
language) or fallible indexing everywhere (ceremony explosion under
mandatory annotations). `Result<T, E>` was rejected for dragging all of
generics into v1.

## Decision

Split failures by kind:

- **Domain failures are values:** Go-style multi-return `(T, Err)`; built-in
  nominal `Err` with `clean` as its absence; comma-ok map reads. No general
  nil.
- **`cide` is spawn-only** (same scoping rule as `emit`): deliberate process
  death. Functions cannot die — a fn without `Err` in its signature cannot
  fail.
- **Machine faults** (Int div/mod by zero, index out of bounds) kill the
  enclosing process — the same honesty compromise total languages already
  make for OOM, made explicit.
- **The program is the root process** and cannot crash from inside the
  language. V1 dead-stage policy: drain-and-report. Supervision/restarts are
  a deferred layer on the same semantics.

## Consequences

- A `fn` signature is a complete failure contract; no invisible control flow
  exists.
- Error-forwarding ladders are verbose until a `?`-alike lands (deferred,
  additive).
- An invariant violation deep in pure code must surface as `Err` through
  every caller's signature — the property working, felt as friction.
- Runtime must implement death → stream auto-close → drain without deadlock.
- Spec §6; rationale §2.
