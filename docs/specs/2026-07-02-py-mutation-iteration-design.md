# py assignment, py iteration, captured-write guard

Date: 2026-07-02. Status: approved. Source: training-project friction list.

Three changes, ordered. A fourth item (context managers) and the real
closure-capture redesign are recorded in docs/backlog.md, not here.

## 1. Assignment into py targets

`x.attr = v` and `x[i] = v` where the target chain is py stop being
compile errors (spec 13.2 currently bans them). `py` is the documented
reference exception; mutating a referent is what references are for.
`param.requires_grad = false` and `batch["labels"] = x` become direct
syntax; the `builtins.setattr` / `operator.setitem` workaround dies.

- Checker: an assignment target that checks as a py chain is a py
  assignment. The assigned value is checked as an expression but not
  against the target type (any value; the bridge's inbound conversion
  table 13.5 governs at runtime). Deep native paths that reach py
  mid-chain (`m["k"].attr = v` for `map[str]py`) follow existing rules on
  the native prefix; the runtime supports py at any depth of the descent.
- Runtime: the assign descent, on reaching a py value with steps left,
  hands the rest to the bridge: getattr/getitem for intermediate steps,
  setattr/setitem for the last.
- Fallibility: assignment is a statement with no error slot. A python
  exception (or an unconvertible value) faults with
  `py assignment: <exception>`. These are deliberate hyperparameter
  pokes; if one raises, the run is broken. Spec 13.3's "python exceptions
  are never faults" sentence gains this exception alongside the existing
  unhandled-chain fault.
- Bridge additions: `setattr(h, name, &Value)`, `setitem(h, &Value,
  &Value)`, both `Result<(), ErrVal>`.

## 2. `for range` over py iterables

`for i, batch := range loader` calls `iter()` once, then `__next__` per
round; StopIteration ends the loop silently. Manual iter/next with the
per-epoch error value dance dies.

- Bindings keep Go's shape so a py iterable ranges like a list: zero
  names runs the body per item, one name binds the iteration index
  (`int`, from 0), two names bind index and item (`py`). More is the
  usual compile error.
- The range operand may be a py chain; ranging absorbs it (expr_pyish).
- Any exception other than StopIteration, including from `iter()`
  itself, faults (`py range: <exception>`). A DataLoader raising
  mid-epoch is fatal; per-iteration error plumbing would tax every loop
  for an unhandleable case.
- Spec 8.7 table gains a `py` row; the "including `py`" exclusion
  sentence goes.
- Bridge additions: `iter(h) -> Result<PyHandle, ErrVal>`,
  `next(h) -> Result<Option<Value>, ErrVal>` (None = StopIteration).

## 3. Compile error on writes to captured names

Interim guard until the capture redesign (backlog): closures capture by
value, so assignment to a captured name is silently lost today — it has
eaten training metrics twice. The checker makes it loud:

- Inside a function literal, an assignment whose target's base identifier
  resolves outside the literal is a compile-time error:
  `<name> is captured by value; writes inside a function literal do not
  escape`. Applies to plain, field, and index assignments on native
  values (mutating a captured copy is equally lost).
- Exception: py-chain targets stay legal — captured py values share the
  handle, so `d["k"] = v` on a captured py dict genuinely escapes. This
  is also the documented accumulator pattern until reference capture
  lands.
- Nested literals check against the innermost boundary. `:=` still
  shadows freely. The repl stays unchecked. Spec 7.3 records the error.

## Verification

Golden tests per item (`py/` cases gated on RIKKI_TEST_PY use json and
builtins, no numpy): py setattr/setitem round-trip, py range over
`json.loads("[1,2,3]")` with index and item, StopIteration terminating,
captured-write compile error, and captured-py-append still legal. Then
the real path: lmtk quickstart must stay green byte for byte.
