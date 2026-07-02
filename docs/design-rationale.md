# Design rationale

Why the spec says what it says. `language-spec.md` records *what*; this
records *why*. The first design's rationale is preserved in git history and
in the superseded ADRs (0001, 0003, 0004); this document covers the
simplified core (ADR 0008).

## 1. The simplification

The first design multiplied four bets — themed hobby language, CSP-only
concurrency, no-crash semantics, value-only memory with a named GC — and its
own §9 admitted the intersection's audience might be one person. The
simplification pass kept the single idea with real leverage and cut the
rest:

- **Kept:** spawn as a stream transducer (0/1/many emits per item gives
  filter/map/flatMap/scan from one primitive), implicit rendezvous channels,
  `|>` composition, structural sources/sinks, and processes as first-class
  values (anonymous literals, partial pipelines).
- **Cut:** the theme (`genesis`, `Vein`, `entomb`, `ossify`, `cide`,
  `clean`), process mortality and the no-crash doctrine, opt-in parallelism
  (`spawn<N>`/`spawn<auto>`) and its checker rule, multi-return and
  destructuring, `Set<T>`, `do`/`while`, the `Err` model, and Autophage as a
  named memory design.

The test applied to each feature: is it load-bearing for writing pipelines?
Everything that failed the test went, however well-designed it was.

## 2. Errors are panics

Mortality was the old answer to "what happens on divide by zero." With it
cut, the honest v1 answer is a program-killing panic — what every young
language does. `(T, Err)` values were considered and deferred rather than
kept: with no error values, multi-return's only remaining job was comma-ok
map reads, and `m.has(k)` + a panicking `m[k]` covers that. Cutting
multi-return deletes destructuring from the grammar and checker.

This is the one cut that is hard to retrofit later (error values are
additive; multi-return syntax less so). Flagged during design; accepted.

## 3. Sequential only, but threaded

Parallelism (`spawn<N>`, unordered emit, the parallel-state rule) was cut as
not essential to the mechanic. The runtime still runs one OS thread per
stage with `sync_channel(0)` rendezvous — true CSP — rather than the simpler
pull-based iterator chain. Chosen deliberately: pull iterators would halve
the interpreter but make later parallelism a semantic change; threads make
it an additive one. This is the one place the simplified design pays for a
future it may never build.

A panic in any stage aborts the whole program. No drain, no report, no
supervision.

## 4. Value semantics as posture, not doctrine

Copy semantics everywhere (assignment, params, `emit`), no aliasing,
implemented as refcount + copy-on-write (`Arc::make_mut`). Kept not for
purity but because it is the *simplest* correct memory story for a Rust
tree-walker — shared mutation would mean `RefCell` everywhere — and
copy-on-emit means stages never share mutable state. Cycles (`n.children.push(n)`)
leak until program exit; acceptable for v1, documented.

## 5. What survived untouched

- Colon-free positional syntax (`name Type`, no `->` return arrow): Go-shaped,
  costs nothing, kept on its merits.
- Explicit annotations, no inference; nominal typing.
- Generics parse everywhere, built-ins only (`Stream`, `Array`, `Map`).
- First-class functions and processes (ADR 0007) — fn/spawn types mirror
  declaration shapes; partial pipelines compose.
- Rust, zero dependencies, tree-walker first (ADRs 0005, 0006). The "own
  the GC" argument for Rust is gone; the owned-runtime-into-v1.1 argument
  remains sufficient.

## 6. Known weaknesses

1. **No error values.** Real programs need fallibility; v1 programs panic.
   The deferred `(T, Err)` design has no multi-return to land on — it will
   need its own syntax decision when it comes.
2. **No user generics, so no in-language stdlib** — `map`/`fold` stay
   hardcoded builtins (unchanged from the first design).
3. **Copy-on-emit cost** for large mutated-per-stage values; unmeasured.
4. **Thread-per-stage is heavier than the language now needs** — the price
   of keeping the parallelism door open (§3).
5. **No string interpolation or Int→Str conversion** — first real program
   hits this in minutes (unchanged; stdlib problem, not a design problem).
