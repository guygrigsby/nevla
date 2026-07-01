# ichor

A statically-typed, garbage-collected programming language where **CSP stream
processes are the central primitive** and **the program cannot crash**.
Processes are cells: they're born (`spawn`), they transform streams
(`Vein<T>`), and they die (`cide`) without taking the organism down. The
garbage collector is **Autophage**: per-process heaps, reclaimed whole when
the cell dies.

Status: **design complete, implementation not started.** The tree-walking
interpreter (v1, Rust) is fully planned; compilation is v1.1.

```
genesis Vessel {
    name Str
    capacity Int
}

spawn vessels() Vein<Vessel> {
    emit Vessel { name "cup", capacity 100 }
    emit Vessel { name "jug", capacity 500 }
}

spawn fill(input Vein<Vessel>) Vein<Vessel> {
    input -> v {
        if v.capacity >= 200 {
            emit v
        }
    }
}

spawn show(input Vein<Vessel>) {
    input -> v {
        print(v.name)
    }
}

vessels |> fill |> show
```

## The three load-bearing ideas

1. **Signatures tell the whole truth.** No exceptions, no panic, no invisible
   control flow. A `fn` that can fail returns `(T, Err)`; a `fn` that doesn't
   cannot fail, period.
2. **Death is process-scoped.** `cide` (and machine faults like divide by
   zero) kill the enclosing `spawn` process — its stream closes, the pipeline
   drains, the program reports and continues. Nothing crashes the program
   from inside the language.
3. **The memory model is the concurrency model.** Value semantics everywhere,
   no pointers, no aliasing. Each process owns its heap; `emit` copies across
   the rendezvous; process death reclaims the whole cell (apoptosis).

## Repo map

| Path | What |
|---|---|
| `language-spec.md` | The spec — source of truth, includes the append-only decision log |
| `docs/adr/` | Architectural decision records (append-only) |
| `docs/design-rationale.md` | Why each major decision went the way it did, what was rejected, known weaknesses |
| `docs/plans/2026-07-01-ichor-v1-interpreter.md` | The v1 implementation plan (17 TDD tasks, Rust, zero deps) |
| `possible-names.md` | Naming candidates; ichor and Autophage chosen |

## Naming

*Ichor* is the blood of the gods — what flows through a `Vein<T>`. *Autophage*
is the cell that consumes its own dead parts. Reference program to eventually
build: eviscerOS. CLI: `ichor run file.ich`.
