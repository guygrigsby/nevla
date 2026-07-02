# 5. Implementation language: Rust

Status: accepted (2026-07-01); rationale weakened by 0008, decision stands

## Context

Go was the default: author's primary language, the reference book (*Writing
an Interpreter in Go*) matches, goroutines map 1:1 onto spawn processes. But
in Go, ichor values ride the host GC — Autophage never exists as owned code,
and v1.1 compilation would require rewriting the runtime out from under the
language (or shipping two collectors).

## Decision

Rust, stable toolchain, edition 2021, **zero external dependencies** in v1.
Autophage must be built, so we own it; the same runtime (values, cells,
scheduler) carries into v1.1. Concurrency maps to stdlib: one OS thread per
process, `std::sync::mpsc::sync_channel(0)` is the CSP rendezvous.

## Consequences

- v1 takes longer than it would in Go; the interpreter's value model must be
  `Send` (`Arc` throughout).
- No dependency risk surface; the build is `cargo build`, nothing else.
- The v1.1 migration is an execution-layer swap, not a rewrite (ADR 0006).
- Rationale §4; plan global constraints.
