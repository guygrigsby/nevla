# ichor

A small programming language: CSP stream processes (`spawn`, `emit`,
`Stream<T>`, `|>`) as the core primitive, sequential-only in v1,
thread-per-stage rendezvous runtime, panics for all errors. Simplified from
a larger first design by ADR 0008. Design complete; implementation not
started.

## Orientation

- `language-spec.md` — **source of truth.** Spec + append-only decision
  log (§8). Any design change lands here and in the log, same commit.
- `docs/adr/` — architectural decision records, append-only; supersede,
  never rewrite. 0008 is the simplification.
- `docs/design-rationale.md` — why the simplified core looks this way,
  and known weaknesses.
- `docs/plans/2026-07-01-ichor-v1-interpreter.md` — SUPERSEDED (targets the
  old spec). New plan pending.

## Rules

- v1 implementation is Rust stable, edition 2021, empty `[dependencies]`.
- The checker owns every static guarantee (emit scoping, annotations,
  pipeline typing — spec §7 lists all four rules); the evaluator trusts it.
- Ichor-level faults (div by zero, index OOB, missing map key, `panic()`)
  abort the program with a clean message, never a raw Rust panic trace.
- Spec drift discovered during implementation goes back into
  `language-spec.md` + decision log in the same commit as the code.
- Commits: terse, verb-first, module prefix (`lexer:`, `spec:`, `plan:`).
