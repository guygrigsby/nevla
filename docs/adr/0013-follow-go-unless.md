# 13. Follow the Go way unless there is a compelling reason not to

Status: accepted 2026-07-10

## Context

Named after a day of design rounds kept landing on "it's Go muscle
memory": the copy model, capitalization visibility, same-package `_test`
files, one gofmt-style format with no configuration, tables over test
frameworks, errors as values. Go's designs are battle-tested, coherent
with each other, and already in the author's fingers. Re-deriving them
per feature wastes rounds and risks incoherence.

## Decision

Go is the taste reference. When a design question has a Go answer, take
it by default. Deviating requires a compelling, articulated reason,
recorded in the design doc or ADR at the moment of deviation — never a
silent drift.

Deviations so far, each with its recorded reason:

- Option types instead of nil and comma-ok: options subsume both, and
  one absence mechanism composing with flow narrowing beats two.
- `check` propagation: Go has no propagation operator; rikki's
  errors-everywhere culture needs one to stay writable.
- Lowercase stdlib (`math.sqrt`, not `math.Sqrt`): the stdlib sits
  beside lowercase builtins as the language's own vocabulary, keeping
  "capital = someone's rikki code, lowercase = the language" legible.
- Structural equality leaning yes (Go refuses): the everything-is-data
  tenet (ADR 0012) argues for it; still undecided, conflict on record.
- The py bridge itself: Go would say write it in Go; rikki exists
  because the ML ecosystem is python.

## Consequences

Design rounds start from "what does Go do"; docs aspire to Go's
standard (per-function sections with runnable examples, not tables);
future deviations must name their compelling reason in writing or they
don't ship.
