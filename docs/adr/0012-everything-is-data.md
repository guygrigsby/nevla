# 12. Everything is data

Status: accepted 2026-07-09

## Context

Named while designing `rikki test`, but the language has been obeying it
since v1: errors are struct-shaped values (msg, pytype, cause, traceback,
soon origin), not control flow; absence is an option value, not nil;
python exceptions cross the bridge as inspectable values; functions are
first-class; a test is a fallible function whose outcome is an error
value; a test table is a list of structs and its runner is a for loop.
Features keep getting simpler because their inputs and outcomes are
ordinary values that can be stored, compared, wrapped, and rendered.

## Decision

Everything is data is a design tenet. When designing a feature, the
outcome of an operation should be a value a program can hold and inspect,
and new concepts should be expressible as values plus existing machinery
before they may become new machinery. Applied consequences so far: errors
as values, test-as-fallible-function, error origins as readable fields,
tables over frameworks.

Boundaries, both deliberate:

- Faults are the anti-data exception: process death, uncatchable, never a
  value inside the program. This is what makes "a checked program cannot
  crash the host" provable. Hosts may reify faults at the process
  boundary (the test runner reports a faulted test as failure data).
- Functions are data as values (passable, storable, callable), not as
  structure: no reflection, no function comparison beyond what the spec
  grants.

## Consequences

- Structural `==` for containers is promoted from "v2 someday" to an open
  question with a bias toward yes; this is a spot where the tenet argues
  against Go (which refuses slice/map comparability). `contains` and
  `test.eq` already compare structurally; the operator decision waits for
  evidence from real code.
- Tool output is data too: test results and diagnostics should grow
  machine-readable forms (`--json`) since agents are first-class users.
- Any future feature whose failure story is "it prints something" or
  "it aborts" must justify why it cannot yield a value instead.
