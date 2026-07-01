# ichor

A programming language: CSP stream processes as the core primitive, process-
scoped mortality (the program cannot crash), value semantics with a
per-process-cell GC (Autophage). Design complete; implementation not started.

## Orientation

- `language-spec.md` — **source of truth.** Complete spec + append-only
  decision log (§9). Any design change lands here and in the log, same
  commit.
- `docs/adr/` — architectural decision records, append-only; supersede,
  never rewrite.
- `docs/design-rationale.md` — why decisions went the way they did, rejected
  alternatives, and §9: known weaknesses, stated for red-teaming.
- `docs/plans/2026-07-01-ichor-v1-interpreter.md` — the v1 implementation
  plan: 17 TDD tasks, Rust, zero dependencies. Not started.

## Rules

- v1 implementation is Rust stable, edition 2021, empty `[dependencies]`.
- The checker owns every static guarantee (emit/cide scoping, parallel state
  rule, pipeline typing); the evaluator trusts it.
- Machine faults are process death, never a Rust panic.
- Spec drift discovered during implementation goes back into
  `language-spec.md` + decision log in the same commit as the code.
- Commits: terse, verb-first, module prefix (`lexer:`, `spec:`, `plan:`).
