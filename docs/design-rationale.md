# Design rationale

Why the spec says what it says: the alternatives considered, why they lost,
and the costs we accepted with eyes open. `language-spec.md` records *what*;
this records *why*. Written to be attackable — if you're here to argue the
whole idea is bad, §9 is your ammunition list.

## 1. Themed keywords, rationed

The first instinct was full theming (rename `if`, loops, everything). Rejected
early: renaming universal concepts costs every future reader and buys nothing.
The rule that survived: a themed word must carry semantic weight the plain
word doesn't. `genesis` (type birth), `spawn`/`emit` (stream process),
`cide`/`clean` (mortality), `entomb` (sealing), `Vein` (the stream itself).
Everything universal stays boring: `if`, `for`, `fn`, `return`.

Corollary decisions: no colons anywhere (annotations are positional
juxtaposition, `name Type`), no `->` return arrow, no `=>` lambdas in v1
(too close to the `->` reactive arrow; anonymous `fn` costs zero new tokens
and sugar can be added compatibly later).

## 2. Error model: from panic to mortality

The longest road in the design. Stations:

- **Start:** `cide` as a conventional panic, plus an open question about a
  recoverable path.
- **`Result<T, E>` rejected** — it drags the entire generics + pattern-match
  design into v1 to serve error handling.
- **Go-style multi-return chosen** for domain failures: `fn f(...) (T, Err)`,
  built-in nominal `Err`, absence spelled `clean`. No general nil enters the
  language; only `Err` has an absence value. Empty-string and comma-ok-only
  variants were rejected (invisible sentinel; no message).
- **Then the pivot:** what if there's no panic at all? Full removal was
  examined and rejected because the built-ins have only bad total answers —
  Pony defines `1/0 == 0` to stay total, and silently-wrong values
  propagating through a *dataflow* language are worse than a crash (you find
  out three stages downstream, no trace). Fallible indexing everywhere is a
  ceremony explosion under mandatory annotations.
- **Landing: death is demoted, not removed.** `cide` is legal only inside
  `spawn` (same scoping rule as `emit`). Functions cannot die — their
  signatures are the complete truth about failure. Machine faults (div by
  zero, index out of bounds) kill the enclosing process, exactly like OOM
  kills programs in "total" languages — that's the honesty compromise every
  such language already makes, made explicit. The program itself cannot
  crash: v1 policy is drain-and-report; supervision (Erlang-style restarts)
  is a deferred layer on the same semantics.

Checked exceptions (Java) were considered as prior art for
"signatures tell the truth" and rejected on ergonomics; the multi-return +
uniform process mortality split gets the property without the wrapping tax.

Accepted costs: the `if err != clean { return ... }` ladder (a `?`-alike
sugar is deferred, additive); a pure `fn` that detects a violated invariant
must become fallible, infecting caller signatures — that's the property
working, but it's real friction.

## 3. Memory: Autophage as per-process cells

Three designs considered for a GC in Rust:

1. **Pure refcounting** — simplest; leaks cycles. Ichor *almost* forbids
   cycles structurally (no nil, no optionals, mandatory initialization make a
   directly-self-containing struct unconstructible) but mutable collections
   reopen the hole: `n.children.push(n)`.
2. **Tracing mark-sweep** — handles cycles; notoriously miserable to build in
   Rust (rooting through the interpreter stack).
3. **Per-process cells (chosen)** — each process owns its heap; `emit` copies
   across the rendezvous, so cross-process references cannot exist; refcount
   within a living cell; on death, drop the whole cell — apoptosis is arena
   reclamation, cycles included. This is Erlang's memory model (per-process
   heaps, copy on send, reclaim on death). Chosen because the checker already
   guarantees no shared mutable state between processes — the concurrency
   model and the memory model turn out to be the same idea.

Consequence, embraced: **everything is a value.** Copy semantics for
assignment, params, construction, emit. No pointers, no references, no
aliasing. Closures capture by copy. Implementation is refcount +
copy-on-write (`Arc::make_mut`), so copies are free until mutation. The
escape hatch for genuinely shared mutable state is a stateful sequential
`spawn` process — processes are the language's replacement for pointers.

Accepted costs: copy-on-emit for large values (Erlang pays the same);
in-cell cycles leak until the process dies (bounded, documented); the v1
interpreter approximates cells with global Arc+COW, so cycle reclamation at
*process* death only truly arrives with the owned v1.1 runtime.

## 4. Rust over Go

Go was the default (author's language; the reference book is Go; goroutines
map 1:1 onto spawn). Rejected for the double-GC problem in its honest form:
for the v1 tree-walker Go would actually be a *single* free GC — but then
Autophage never exists as owned code, and v1.1 compilation would mean
rewriting the runtime out from under the language. In Rust, Autophage must
be built, so we own it, and the same runtime (values, cells, scheduler)
carries into v1.1. Accepted cost: v1 is slower to build in Rust.
Concurrency stays stdlib: one thread per process, `sync_channel(0)` is
literally the CSP rendezvous.

## 5. Concurrency decisions

- **CSP with implicit channels** — the language's reason to exist. No manual
  channel wiring; `|>` composes; rendezvous (unbuffered) only in v1.
- **`spawn` is not a `fn`** — a process is a stream transducer with a
  lifetime, not a call. Variable emit count (0/1/many per item) gives
  filter/map/flatMap/scan/window from one primitive.
- **Sequential by default, parallel opt-in** (`spawn<8>`, `spawn<auto>`).
  The central safety rule: mutable cross-item state in a parallel process is
  a compile error. Sequential processes keep stateful operators safe.
- **Parallel is unordered** — reorder buffers and head-of-line blocking
  rejected; bare `spawn` already provides ordered processing; a reordering
  stage can be stdlib later.
- **Sources and sinks are structural** — a spawn with no input is a source,
  no output slot is a sink. Themed keywords (`font`/`drain`) rejected: the
  shapes already say it.
- **First-class processes** — process types mirror declarations
  (`spawn(Vein<Int>) Vein<Int>`), anonymous spawn literals, and partial
  pipelines as composable values. Chosen so fan-out/fan-in and supervisors
  can be *library* code instead of syntax.

## 6. Type system scope for v1

Explicit annotations everywhere (no inference — one rule, zero exceptions,
including multi-return destructures and fn-literal bindings, where the
annotation is admittedly redundant with the literal). Nominal typing.
Generics parse everywhere but only built-ins are generic in v1; user
generics deferred intact (constraints/bounds/instantiation is the largest
single undesigned area). Collections are `Array`/`Set`/`Map` with colon-free
literals; map reads are comma-ok only. `entomb` forbids construction, not
mention. `ossify` stays a reserved word with no meaning (likely future:
compile-time consts).

## 7. Syntax hazards found and resolved

- **`IDENT {` ambiguity** (construction vs block) in unparenthesized `if`
  conditions — resolved Go-style: no construction literals in
  condition/iterable position.
- **No statement terminator** — resolved with Go-style automatic `Semi`
  insertion at newlines after statement-ending tokens.
- **`Ints` pluralization** for stream types — pleasant, brittle, rejected
  for `Vein<T>`.
- **`Err(...)` constructs call-style** while user structs construct
  braced-field-style — accepted inconsistency for a built-in; flagged, not
  hidden.

## 8. Implementation strategy

Tree-walking interpreter first (v1), compilation later (v1.1); lexer,
parser, AST, and checker are shared, only the execution layer is replaced.
Sequential spawn semantics before real parallelism. Zero Rust dependencies.
Plan: `docs/plans/2026-07-01-ichor-v1-interpreter.md`.

## 9. Known weaknesses (attack here)

Stated plainly so the red team doesn't have to dig:

1. **Annotation burden.** No inference plus mandatory multi-return
   destructure annotations makes fallible call sites heavy:
   `let cfg Config, err Err = parse(s)` then a three-line forwarding ladder,
   at every call. The `?`-sugar is deferred, which means v1 programs will
   feel this everywhere.
2. **No user generics means no real stdlib in-language.** `map`/`fold` are
   hardcoded checker/interpreter builtins. Until generics land, ichor can't
   express its own combinators — the "fan-out belongs in the stdlib" story
   is a promissory note v1 cannot cash.
3. **Copy-on-emit cost.** COW hides copies until mutation, but a pipeline
   that mutates large structs per item pays real copies per stage. No
   benchmarks exist; "Erlang pays this too" is an analogy, not a measurement.
4. **Unordered parallel is a footgun.** `spawn<8>` silently changing output
   order relative to bare `spawn` will surprise people; the safety rule
   prevents state races but not ordering bugs downstream.
5. **One Err type.** `Err` carries only a message string — no kinds, no
   wrapping, no matching. Fine for v1 demos; real error handling taxonomies
   have nowhere to live yet.
6. **Comma-ok is the only map read.** No default-value read, no
   entry-or-insert; every map access is two bindings.
7. **No Int→Str conversion or string interpolation** exists yet — you can't
   even print `"count: " + n`. First real program will hit this in minutes.
8. **Mortality model untested at scale.** Drain-and-report is clean on
   paper; whether "the stage died, the pipeline kept going, you find out at
   the end" is actually *nicer* than fail-fast for real workloads is an
   empirical question v1 exists to answer.
9. **Niche stacked on niche.** A themed hobby language + CSP-only
   concurrency + no-crash semantics + value-only memory is four bets
   multiplied together. Each is defensible; the intersection's audience may
   be one person. (That may also be fine — it's the point of a hobby
   language — but it should be said out loud.)
