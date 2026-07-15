# 23. Semantic model API for agent tooling

Status: accepted (2026-07-14)

## Context

agent-python (`~/projects/agent-python`, separate repo) is a semantic control
plane for agents working nevla code: query a resolved model (find a symbol,
find its callers) and mutate through a transactional envelope gated by
`nevla check` and the test runner. Its ADR 0003 picked nevla as the first
target because our front end already computes a sound, whole-program model
and confines dynamism to the `py` type.

The front end computes that model and discards it: the checker returns only
diagnostics, AST identifiers are bare strings, and no def-to-uses index
survives. agent-python needs symbols, references, call edges, and py
boundaries as data.

## Decision

- Add `nevla::model`, a public module exposing
  `analyze(entry: &Path) -> Result<Model, Vec<Diag>>` where `Model` carries
  symbols (with stable IDs `kind:file:qualified` and source positions),
  references (tagged by form: ident, module-member call, struct literal),
  call edges, and py boundaries. All types serde-serializable so a JSON or
  gRPC surface later is serialization, not redesign.
- The module is a standalone AST walk with its own lightweight, deliberately
  over-approximate scope tracking (function-scoped locals). It shares no code
  with and does not modify the typechecker: the model is a public surface,
  not a window into checker internals. Consumers own soundness recovery: a
  missed reference surfaces as a typecheck failure in their mutation gate.
- No language-semantics change. `language-spec.md` is untouched; the module
  is additive tooling. Serde becomes a regular dependency.
- Contract evolution: agent-python links nevla by Cargo path dependency and
  compiles against these types. Breaking the shape means fixing both repos in
  the same sitting; the compiler enforces the pairing.

## Consequences

- nevla grows a second public consumer surface beyond the CLI. API changes
  to `model` now have an external dependent.
- The resolver duplicates scope logic conceptually (not textually) with the
  checker. Acceptable for the over-approximate skeleton; if the two drift
  painfully, factoring the checker's binding rules into a shared walk is the
  follow-up, per the original design intent.
- `TypeExpr` carries no spans, so type-annotation positions cannot be emitted
  as references. Struct renames in consumers must name-match type positions
  (sound: nothing shadows a struct name in type position). Adding spans to
  `TypeExpr` is future work if sub-symbol precision starts to matter.
- Runtime tracing (the dynamic evidence layer in agent-python's design) will
  later want hooks in the interpreter and bridge; nothing here precludes it.
