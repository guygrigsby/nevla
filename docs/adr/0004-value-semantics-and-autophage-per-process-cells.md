# 4. Value semantics and Autophage per-process cells

Status: superseded by 0008 (2026-07-01); value semantics survives as an implementation posture

## Context

The GC (Autophage) needed a real design. Pure refcounting leaks cycles
(mutable collections can close loops: `n.children.push(n)`); tracing GC in
Rust is the classic hard problem. The checker already forbids shared mutable
state between processes, which makes a third option structural.

## Decision

- **Everything is a value.** Copy semantics for assignment, parameters,
  construction, and `emit`. No pointers, no references, no aliasing.
  Closures and spawn literals capture by copy. Shared mutable state lives in
  a stateful sequential `spawn` process — processes replace pointers.
- **Autophage = per-process cells** (Erlang's memory model, themed): each
  process owns its heap; `emit` copies across the rendezvous so no
  cross-process references exist; refcounting within a living cell; on
  process death the whole cell is reclaimed at once — apoptosis is arena
  reclamation, cycles included.
- Implementation is refcount + copy-on-write (`Arc::make_mut`): copies are
  free until a shared value is mutated.

## Consequences

- GC is process-local: no global pauses; parallel workers share nothing.
- Recursive genesis types stay legal; in-cell cycles leak only until the
  process dies.
- Copy-on-emit costs real copies for mutated-per-stage large values;
  unmeasured (rationale §9.3).
- V1 interpreter approximates cells with global Arc+COW — cycle reclamation
  at process death only truly arrives with the owned v1.1 runtime.
- Spec §7; rationale §3.
