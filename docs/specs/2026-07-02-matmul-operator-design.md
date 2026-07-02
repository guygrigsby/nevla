# `@` matmul operator

Date: 2026-07-02. Status: approved for implementation.

## Why

rikki exists to write ML code and experiments without authoring Python.
Matrix multiplication is the single most common operation in that code;
today it spells `torch.matmul(w, x)` or `w.matmul(x)` through the bridge.
This adds the `@` operator as sugar over the same bridge path, so training
code reads `y = w @ x + b`. Promoted from the v2 backlog (where it was
already decided) because the ML purpose is now the design center.

## Grammar and lexing

- New token `@`.
- Binary operator only: no unary form, no compound assignment.
- Multiplication precedence, left-associative, exactly where `*` sits:
  `y = w @ x + b` parses as `(w @ x) + b`; `a @ b @ c` as `(a @ b) @ c`.
- New `ast::BinOp::MatMul` variant. The checker's and interpreter's binop
  matches are exhaustive (no `_` arms), so the compiler enforces coverage.

## Checking

`@` has no native meaning. The rule mirrors the existing py behavior of the
arithmetic operators:

- If either operand is `py` (or a py chain), the result is `py`, and py
  chain absorption works exactly as it does for `+`.
- If neither operand is py: compile-time diagnostic
  `@ needs py operands; there is no native matrix type`.
- Mixed forms (`2 @ tensor`) typecheck as py and let Python decide at
  runtime, consistent with `tensor + 1` today.

## Runtime

`BinOp::MatMul` dispatches through `bridge::binop` like `+ - * / %`:

- Preferred: pyo3's number-protocol method on `Bound` (`matmul`), one arm
  like `add`.
- Fallback if that method is absent in pyo3 0.29: call Python's
  `operator.matmul`, which runs the full `__matmul__` / `__rmatmul__` /
  NotImplemented protocol.
- Python exceptions become error values through the chain, like every other
  py operation. A matmul reaching a statement unconsumed is the usual
  unhandled-py fault.

## Spec and tests

Same commit as the semantics, per CLAUDE.md:

- language-spec.md: token list, precedence table, py-operators section
  (13.x), and the compile-error wording above.
- Golden `check/matmul-native.err`: `@` on non-py operands is a
  compile-time diagnostic.
- Bridge unit test: define a Python class with `__matmul__` in the embedded
  interpreter, assert `bridge::binop(MatMul, ..)` round-trips (the test
  venv has no numpy/torch, so the runtime proof lives at the bridge layer).
- Golden `py/matmul.rk` gated on RIKKI_TEST_PY if a stdlib-only vehicle
  exists; otherwise the bridge test carries runtime coverage.

## Verification through the real path

- Run a matmul snippet against lmtk's venv torch (`tk` with the lmtk
  project), asserting `w @ x` equals `torch.matmul(w, x)`.
- Adopt `@` in lmtk where it currently spells matmul explicitly; lmtk's
  quickstart (seeded, byte-for-byte reproducible) must produce identical
  output before and after.

## Rejected: native `[][]float` matmul

Considered and rejected (2026-07-02). A native `@` over `Vec<Value>` is a
boxed O(n^3) loop: fine in a toy test, uselessly slow at real sizes, and an
invitation to write math in the wrong layer when the design is "objects and
speed stay PyTorch's". It cannot reach torch without conversion anyway, it
drags in shape/rectangularity validation against list types that may lie at
runtime (spec 7.7), and it makes `@` mean math on containers whose other
operators (`+` is concat) mean data. If native matrices ever earn their
keep, they arrive as a `math.matmul` stdlib function, not operator
semantics.
