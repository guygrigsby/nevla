# 8. Simplify to a minimal core around spawn

Status: accepted (2026-07-01). Supersedes 0001, 0003, 0004; amends 0002.

## Context

The first design multiplied four bets: a themed hobby language, CSP-only
concurrency, no-crash semantics, and value-only memory with a named GC
(rationale §9.9 called this out). On reflection the whole was too large for
a toy language, but one piece is worth keeping: the spawn mechanic — stream
transducers with variable emit count, implicit channels, `|>` composition,
and processes as first-class values.

## Decision

Cut the language to the minimal core that carries spawn:

- **Theme cut** (supersedes 0001): `genesis`→`type`, `Vein<T>`→`Stream<T>`;
  `entomb`, `ossify`, `cide`, `clean` deleted. The colon-free positional
  syntax survives on its merits, not as identity. Repo/CLI name ichor stays.
- **Mortality cut** (supersedes 0003): no process death, no drain-and-report,
  no supervision story. Machine faults (divide by zero, index out of bounds,
  missing map key) and explicit `panic()` kill the program with a message.
- **Memory doctrine cut** (supersedes 0004): value semantics and
  copy-on-emit survive as the simplest correct implementation (refcount +
  copy-on-write), not as a named per-process-cell GC. Cycles leak until
  program exit; acceptable for v1.
- **Parallelism cut** (amends 0002): no `spawn<N>`/`spawn<auto>`, no
  parallel-state checker rule. Spawn is sequential and ordered. The rest of
  0002 — CSP, implicit rendezvous channels, variable emit count, structural
  sources/sinks — stands unchanged.
- **Errors cut to panics**: no `Err`, no multi-return, no destructuring.
  Comma-ok's job is done by `m.has(k)` + panicking `m[k]`.
- **Runtime**: thread per stage with `sync_channel(0)` rendezvous, chosen
  over pull-based iterators to keep true CSP semantics and a
  zero-semantic-change path to reintroducing parallelism later.

0005 (Rust) stands with a weakened rationale: the "Autophage must be owned"
argument is gone, but the owned runtime still carries into v1.1 compilation
and stdlib Rust (`std::thread`, `sync_channel(0)`, `Arc::make_mut`) covers
the whole runtime with zero dependencies. 0006 and 0007 stand unchanged.

## Consequences

- The checker shrinks to four rules (spec §7); the interpreter loses the
  scheduler policy surface, mortality bookkeeping, and destructuring.
- Multi-return is the one cut that is hard to retrofit; flagged and
  accepted.
- The 17-task implementation plan targets the old spec and is superseded; a
  new, smaller plan is needed.
- Spec §8 log entries 23–32 record the details.
