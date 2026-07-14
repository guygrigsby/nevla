# 22. []byte append always copies; the lent flag is gone

Status: accepted 2026-07-14 (supersedes part of ADR 0021)

## Context

ADR 0021 accepted a lent flag on `BytesBuf`: a buffer never lent across
the bridge could grow in place on `append` (amortized O(1) for
build-by-append loops), a buffer ever lent never would, "exactly
`append`'s rebinding semantics." Task 3 implemented this as
`Rc::strong_count(&buf) <= 2 && !buf.borrow().lent`, verified empirically
against two cases: the reassignment idiom (`b = append(b, x)`, reuses one
allocation) and a real alias taken before any append (`a := b; b =
append(b, ...)`, correctly forces a copy).

A Task 3 review round found a third case neither check covered: binding
`append`'s result to a *different* name while the original name is kept:

```
b := []byte{1, 2}
c := append(b, 3)
print(b)   // printed [1, 2, 3]; should print [1, 2]
```

`eval(Ident)` clones the variable's `Rc` into the argument list
(`cell.borrow().clone()`), so evaluating a bare `b` argument always yields
`strong_count == 2` (the cell's own reference plus this temporary) whether
the caller is about to rebind `b` to the result (safe: one name, no
observable mutation) or bind the result elsewhere while keeping `b`
untouched (not safe: growing in place mutates the buffer `b`'s cell still
points to). The two shapes are indistinguishable at the refcount check;
telling them apart needs move semantics or escape analysis the
interpreter doesn't have. The lent flag never entered into this bug (it
starts and stays `false` until a bridge crossing that doesn't exist yet);
the refcount half of the check was unsound on its own.

This directly broke the contract chapter 11 and section 14.7 state for
every `[]T`, byte or not: "`append(xs, v)` stays pure... growth becomes
visible to other names only by rebinding."

## Decision

`append` on `[]byte` always copies, unconditionally, like the `[]T` (List)
arm right next to it. `BytesBuf` drops the `lent` field entirely: with no
in-place growth of any kind, there is nothing left for it to gate.
Index-assignment (`b[i] = v`) is unaffected — it writes one element in
place and never reallocates, so it was never part of this hazard and
still gives the bridge a stable address for the lifetime of a buffer that
isn't appended to.

Correctness over the optimization: a refcount check that cannot
distinguish "the caller is discarding this binding" from "the caller is
keeping it" is not a valid soundness proof, no matter how it tested
against the cases it was checked against.

## Consequences

- `[]byte` append is O(n) per call, same as every other `[]T`; the
  amortized-O(1) shard-build case ADR 0021 sized this feature for no
  longer gets special-cased at the interpreter level. A future
  reintroduction needs a mechanism that can actually tell the two call
  shapes apart (move semantics, an escape analysis pass, or a
  compiler-visible `xs = append(xs, ...)` pattern match) — none of which
  exist today; re-entry path, not a promise.
- The lent flag's stated purpose in ADR 0021's consequences ("the seam a
  future concurrency story hooks into") is gone with it. Task 7's
  buffer-protocol bridge crossing starts from no private/shared
  distinction; `docs/proposals/concurrency.md` is updated to stop
  pointing at a mechanism that no longer exists.
- ADR 0021's bullet "a buffer never lent across the bridge may grow in
  place on `append`... exactly `append`'s rebinding semantics" is
  superseded by this record; that sentence was the bug.
