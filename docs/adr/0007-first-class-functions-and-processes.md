# 7. First-class functions and processes

Status: accepted (2026-07-01)

## Context

Fn values appeared in spec examples with no fn type syntax, and spawns were
first-class only inside pipeline expressions. Fan-out/fan-in and supervision
are promised as *library* code (ADR 0002), which is impossible unless
processes are values user code can accept, compose, and return.

## Decision

- Fn and spawn types mirror their declaration shapes minus name and body:
  `fn(Int) Int`, `fn(Str) (Config, Err)`, `spawn(Vein<Int>) Vein<Int>`,
  `spawn() Vein<Int>` (source), `spawn(Vein<Int>)` (sink), `spawn()`
  (complete pipeline). Worker count is a definition attribute, not part of
  the type.
- Anonymous spawn literals work inline, like anonymous `fn`.
- Partial pipelines are process values: `|>` composes two compatible
  processes into a new one. A complete `spawn()` value runs when used as an
  expression statement.
- Binding annotations stay mandatory for fn/spawn values — no manifest-type
  exception; one rule, zero exceptions.
- Captures are by copy (forced by ADR 0004); a spawn literal's captures copy
  into its cell.

## Consequences

- Pipelines are an algebra; topology helpers and supervisors become ordinary
  library code once user generics land.
- Annotated fn-value bindings are redundant with their literals — accepted
  verbosity.
- Mutating a captured `var` inside a literal is a compile error rather than
  a silent private copy.
- Spec §3.3, §4.8; rationale §5.
