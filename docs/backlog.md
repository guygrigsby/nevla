# Backlog

Not commitments, just recorded intent. Ordered roughly by expected pain.

## v1.1

- Keyword arguments for py calls. Positional-only today; torch APIs lean hard
  on kwargs, this bites first.
- `bytes` type. Unblocks binary file and http bodies. Touches literals,
  indexing, conversions.
- `mongoose fmt`. One true style, needs a lossless formatter.
- Repl typechecking (currently unchecked).

## v2

- Matrix operations. First-class operators or a core type for numeric work,
  so the common tensor/linear-algebra shapes do not have to round-trip
  through `py` calls one method at a time. Design open: native type vs
  operator sugar over the bridge (a `py` tensor already supports `+ * @`
  on the Python side; `@` matmul operator is not in the grammar yet).
- Concurrency. Bridge module is the single place GIL work lands.
- Bytecode VM if pure-mongoose loops ever hurt (recorded in ADR 0001).
- Copy-on-write values if profiling demands (ADR 0004).
- `==` structural equality for containers (contains already compares
  structurally; the operator stays scalar-only in v1).
