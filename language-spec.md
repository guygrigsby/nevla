# Language Spec (working draft)

> Status: simplified core, design pass complete. A first design produced a
> much larger language (themed keywords, process mortality, opt-in
> parallelism, a named GC); a simplification pass (2026-07-01) cut it to the
> minimal core around the one idea worth keeping: **CSP stream processes
> composed into pipelines**. The decision log (§8) records both passes.
>
> Name: **ichor** (repo and CLI). The in-language theme from the first
> design is gone; the name stays.

---

## 1. Design philosophy

One idea, minimally dressed: **stream processes as a first-class primitive.**
A `spawn` is a stream transducer; pipelines compose with `|>`; everything
else in the language exists to make writing those pipelines pleasant and
nothing more.

Everything is plain. No themed keywords, no doctrine. The language is
**statically typed** with **explicit annotations** (no inference),
**nominally typed**, and **garbage collected**. Errors are not modeled in
v1: anything unexpected panics and kills the program, like every other
young language.

Syntax is positional and colon-free: `name Type`, no `->` return arrow,
no `=>`. (Kept from the first design because it is Go-shaped and costs
nothing.)

---

## 2. Types and bindings

### 2.1 Primitives

- `Int` — 64-bit integer
- `Float` — 64-bit float
- `Str` — string
- `Bool` — boolean

No sized variants, no `Char`; indexing a `Str` yields a `Str`. No unit/void
type: a function that returns nothing omits the return slot (§3).

### 2.2 Bindings

```
let count Int = 0      // immutable (the default to reach for)
var counter Int = 0    // mutable, explicit opt-in
```

Positional: `let NAME TYPE = EXPR`. Every binding is annotated — no
inference, no exceptions.

### 2.3 Structs — `type`

```
type Vessel {
    name Str
    capacity Int
}
```

Construction mirrors declaration: braced field-value pairs by
juxtaposition, order-independent.

```
let v Vessel = Vessel { name "cup", capacity 100 }
```

Field access and (through `var` bindings) field mutation use dot syntax.
Nominal identity: two structurally identical types are distinct.

### 2.4 Collections

- `Array<T>` — growable ordered sequence. Literal `[1, 2, 3]`, indexing
  `xs[i]`, `xs.len()`, `xs.push(x)`, iteration with `for`.
- `Map<K, V>` — key/value. Literal by juxtaposition: `{ "ada" 36 }`.
  `m[k]` **panics if the key is missing**; check first with `m.has(k)`.
  `m[k] = v` inserts or updates (through a `var` binding).

Empty literals `[]` / `{}` are unambiguous because bindings are annotated.

No `Set` in v1.

### 2.5 Generics

`Type<Param>` syntax parses in all type positions, but only built-ins are
generic in v1: `Stream<T>`, `Array<T>`, `Map<K, V>`. User-defined generics
are deferred intact.

---

## 3. Functions — `fn`

```
fn fill(v Vessel, amount Int) Vessel {
    v.capacity = v.capacity - amount
    return v
}

fn log(msg Str) {
    // no return slot: returns nothing
}
```

Positional signatures: parameters `name Type` in parens, return type after
the closing paren, omitted when nothing is returned. **Single return value
only** — there is no multi-return and no tuple type.

### 3.1 First-class functions

Functions are values; a lambda is an anonymous `fn`, identical syntax minus
the name. Function types mirror the declaration shape: `fn(Int) Int`,
`fn(Str)` (returns nothing).

```
let square fn(Int) Int = fn(x Int) Int { return x * x }
let doubled Array<Int> = nums.map(fn(n Int) Int { return n * 2 })
```

Closures capture by copy (§6): mutating a captured `var` inside a closure
changes the closure's private copy.

---

## 4. Streams — `spawn`

The heart of the language. A `spawn` is a **stream transducer**: it consumes
an input `Stream<T>` and produces an output `Stream<U>`, reacting to each
item as it arrives over its whole lifetime. It cannot be called and cannot
`return` a value — it `emit`s.

```
spawn double(input Stream<Int>) Stream<Int> {
    input -> item {
        emit item * 2
    }
}
```

- Signature mirrors `fn`; the parameter and return types are stream types.
- `input -> item { ... }` — the **reactive body**: runs per item, looping
  implicitly until the input stream closes.
- `emit EXPR` — pushes a value to the output stream. Only legal inside a
  `spawn` body; `emit` in a `fn` is a compile error.

### 4.1 Variable emit count

An input item may emit zero, one, or many times — this is what makes `spawn`
a transducer rather than a map:

```
spawn evens(input Stream<Int>) Stream<Int> {
    input -> item {
        if item % 2 == 0 { emit item }       // zero emits = filter
    }
}

spawn runningTotal(input Stream<Int>) Stream<Int> {
    var sum Int = 0
    input -> item {
        sum = sum + item
        emit sum                              // state across items = scan
    }
}
```

Filter, map, flatMap, fold/scan, batching, and windowing all fall out of one
primitive. Processing is **sequential and ordered**: one item at a time, in
order, with exclusive access to the process's own `var` state. (Parallel
workers were designed and cut; see log entry 27.)

### 4.2 Channels

Implicit and unbuffered (rendezvous): an `emit` blocks until the downstream
stage receives. No channel is ever declared or wired by hand. A process's
output stream closes automatically when its body ends; there is no manual
`close()`.

### 4.3 Pipelines — `|>`

```
source |> double |> double |> sink
```

`|>` feeds one process's output stream into the next's input stream; the
runtime creates the channel between them.

### 4.4 Sources and sinks — structural

- **Source** — a `spawn` with no input parameter: its body runs once, emits
  whatever it likes, stream auto-closes when the body ends.
- **Sink** — a `spawn` with no return slot: consumes its input, emits
  nothing.

```
spawn nums() Stream<Int> {
    for i in 0..100 { emit i }
}

spawn show(input Stream<Int>) {
    input -> item { print(item) }
}

nums |> double |> show      // complete, runnable pipeline
```

A pipeline expression ending in a sink is a **runnable statement**: it runs
to completion (all streams drained and closed) before the next statement.

### 4.5 Processes as values

Spawns are first-class, same as functions:

- Process types mirror declarations: `spawn(Stream<Int>) Stream<Int>` is a
  transducer, `spawn() Stream<Int>` a source, `spawn(Stream<Int>)` a sink,
  `spawn()` a complete pipeline.
- Anonymous spawn literals work inline, exactly like anonymous `fn`.
- **Partial pipelines are process values**: `|>` composes two compatible
  processes into a new one; the open ends are the new type.

```
let quad spawn(Stream<Int>) Stream<Int> = double |> double
let job spawn() = nums |> quad |> show
job                          // statement: runs the pipeline
```

A spawn literal's captures are copied into the process at creation, the same
copy rule as everything crossing a process boundary (§6).

---

## 5. Control flow and panics

- `if` / `else` — conditional.
- `for` — iteration: `for item in xs { ... }`, `for i in 0..10 { ... }`.
  Ranges are exclusive: `0..10` yields 0 through 9.
- `while` — pre-condition loop.
- `return` — return from a function.

**Panics.** Anything unexpected kills the program with a message: integer
division by zero, array index out of bounds, missing map key, or an explicit
`panic("msg")` (a builtin function, like `print`). There are no error
values, no try/catch, and no recovery in v1. A `(T, Err)` convention can be
layered on later; nothing in the core blocks it.

---

## 6. Value semantics

Assignment, parameter passing, construction, and `emit` all copy. No
pointers, no references, no aliasing. Because nothing aliases, the
implementation is free to refcount and copy lazily (copy-on-write), so
copies cost nothing until a shared value is mutated.

This is an implementation posture, not a doctrine: it exists because
copy-on-write refcounting is the *simplest* correct memory story for a
tree-walking interpreter, and because copy-on-emit means stages never share
mutable state.

---

## 7. Reference

Keywords: `type` `let` `var` `fn` `spawn` `emit` `if` `else` `for` `while`
`in` `return`.

Builtin functions: `print`, `panic`.

Built-in types: `Int`, `Float`, `Str`, `Bool`; `Stream<T>`, `Array<T>`,
`Map<K, V>`; function types `fn(T, ...) R`; process types
`spawn(Stream<T>) Stream<U>` (either side omissible — §4.5).

| Token  | Meaning                                              |
|--------|------------------------------------------------------|
| `<>`   | Generics (built-ins only in v1)                      |
| `->`   | Reactive-body arrow in a `spawn` (`input -> item`)   |
| `\|>`  | Pipeline composition                                 |
| `..`   | Range, exclusive upper bound                         |
| `=`    | Assignment / binding                                 |
| `[]`   | Array literal / indexing                             |
| `{}`   | Blocks; struct construction; Map literals            |

No `:` anywhere. No `->` as a return separator. No `=>`.

**Static rules (the whole checker):**

1. `emit` only inside a `spawn` body.
2. Every binding, parameter, and return slot annotated; types match
   nominally.
3. Pipeline composition: each stage's output stream type matches the next
   stage's input; only complete `spawn()` values run as statements.
4. No construction literals in condition/iterable position (the `IDENT {`
   ambiguity, resolved Go-style).

---

## 8. Decision log

Append-only. Entries 1–22 are the first design (most now superseded — kept
as history); entries 23+ are the simplification pass.

Resolved (second design session, 2026-07-01):

1. **Struct construction** — braced field-value by juxtaposition, mirroring
   declaration; order-independent.
2. **Stream type** — `Vein<T>`. *(Superseded by 26: now `Stream<T>`.)*
3. **Primitives** — `Int`, `Float`, `Str`, `Bool`, all 64-bit where sized.
4. **Returns nothing** — omit the return-type slot.
5. **Closures** — anonymous `fn`; no `=>`.
6. **Ranges** — exclusive upper bound.
7. **Parallel ordering** — `spawn<N>`/`spawn<auto>` unordered; bare `spawn`
   ordered. *(Superseded by 27: parallelism cut.)*
8. **Error model** — signatures tell the whole truth; the program cannot
   crash. *(Superseded by 28: panics.)*
9. **Multi-return** — signature feature only; no first-class tuples.
   *(Superseded by 29: multi-return cut.)*
10. **Collections** — `Array<T>`, `Set<T>`, `Map<K, V>`. *(Amended by 30:
    `Set` cut.)*
11. **Pipeline ends** — structural: no-input spawn = source, no-output
    spawn = sink; sink-terminated pipeline is a runnable statement.
12. **entomb** — no new instances. *(Superseded by 26: cut.)*
13. **ossify** — reserved. *(Superseded by 26: cut.)*
14. **Generics** — built-ins only in v1.

Resolved (naming and memory sessions, 2026-07-01):

15. **Names** — language and CLI are **ichor**; GC is **Autophage**.
    *(Amended by 31: ichor stays, Autophage cut as a named design.)*
16. **Memory model** — value semantics everywhere; per-process cells.
    *(Amended by 31: value semantics stays, cell doctrine cut.)*
17. **Recursive types** — legal; cycles leak until process death.
    *(Amended by 31: cycles leak until program exit; acceptable for v1.)*
18. **Implementation language** — Rust. *(Stands; see ADR 0008 for the
    weakened but sufficient rationale.)*

Resolved (first-class values session, 2026-07-01):

19. **Fn and spawn types** — mirror the declaration shapes minus name and
    body.
20. **Full spawn first-classness** — anonymous literals, partial pipelines
    as composable values, complete pipelines runnable as statements.
21. **Annotations stay mandatory** for fn/spawn-valued bindings.
22. **Closures capture by copy.**

Resolved (simplification session, 2026-07-01 — ADR 0008):

23. **The language survives; the size doesn't.** Keep a real language built
    around the spawn mechanic; cut everything that isn't load-bearing for
    it.
24. **What the spawn mechanic means** — transducer + pipelines (variable
    emit count, implicit channels, `|>`, structural sources/sinks) and
    first-class processes. Explicitly *not* chosen as essential:
    parallelism, mortality.
25. **Machine faults panic the program.** No process mortality, no `cide`,
    no drain-and-report, no supervision. Divide by zero, index out of
    bounds, missing map key, `panic()` — all kill the program with a
    message.
26. **Theme cut entirely.** `genesis`→`type`, `Vein<T>`→`Stream<T>`;
    `entomb`, `ossify`, `cide`, `clean` deleted. The repo/CLI name ichor
    stays.
27. **Parallelism cut.** No `spawn<N>`/`spawn<auto>`, no parallel-state
    checker rule. Spawn is sequential and ordered, full stop. The
    thread-per-stage runtime (entry 32) keeps the door open.
28. **Errors are panics.** No `Err` type, no `clean`, no error values in
    v1. `(T, Err)` can be layered on later.
29. **Multi-return cut.** Its only surviving job was comma-ok; `m.has(k)` +
    panicking `m[k]` replaces it. Deletes destructuring from grammar and
    checker. Flagged as the one hard-to-retrofit cut; accepted.
30. **`Set<T>` cut.** `do`/`while` cut; plain `while` restored.
31. **Memory model demoted to a footnote.** Value semantics and
    copy-on-emit stay (they are the simple option, not a doctrine);
    Autophage as a named per-process-cell GC design is cut. Refcount + COW;
    cycles leak until program exit.
32. **Runtime is thread-per-stage CSP.** One OS thread per spawn,
    `sync_channel(0)` rendezvous. Chosen over pull-based iterators to keep
    real CSP semantics and a zero-semantic-change path to parallelism
    later. A panic in any stage thread takes the program down.

Deferred (recorded, not blocking):

- `(T, Err)` error values and propagation sugar.
- User-defined generics; bounds.
- `spawn<N>` parallelism (would reuse the first design's checker rule).
- Fan-out / fan-in stdlib helpers.
- `=>` expression-bodied lambda sugar; buffered channels; `Set<T>`;
  inclusive ranges; spread/functional update; string interpolation.

---

## 9. Implementation path

Rust stable, edition 2021, zero dependencies. Lexer → recursive-descent +
Pratt parser → AST → checker (the four rules, §7) → tree-walking evaluator.

Runtime: one OS thread per spawn process, `std::sync::mpsc::sync_channel(0)`
as the CSP rendezvous, `Arc` + copy-on-write (`Arc::make_mut`) for value
semantics. A panic in any stage aborts the program with the panic message.

The prior 17-task plan (`docs/plans/2026-07-01-ichor-v1-interpreter.md`)
targets the pre-simplification spec and is superseded; a new plan should be
written against this document.
