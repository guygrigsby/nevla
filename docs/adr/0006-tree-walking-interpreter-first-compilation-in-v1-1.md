# 6. Tree-walking interpreter first; compilation in v1.1

Status: accepted (2026-07-01)

## Context

Fastest path to running programs versus performance. The language's novel
claims (mortality model, cell memory, CSP ergonomics) need validation by
real programs before any backend investment makes sense.

## Decision

v1 is a tree-walking interpreter, complete against the spec. v1.1 moves to
compilation (backend — bytecode VM or native — picked when v1 works). Lexer,
parser, AST, and checker are shared; only the execution layer is replaced.
Within v1: sequential spawn semantics first, true parallelism last.

## Consequences

- Semantics get validated cheaply; performance claims are explicitly not a
  v1 goal.
- The front-end must not grow interpreter-only assumptions; the checker owns
  all static guarantees so any backend inherits them.
- True arena-backed cells (ADR 0004) are deferred to the v1.1 runtime.
- Spec §10; plan `docs/plans/2026-07-01-ichor-v1-interpreter.md`.
