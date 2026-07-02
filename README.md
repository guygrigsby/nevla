# ichor

A small statically-typed language where **CSP stream processes are the
central primitive**. A `spawn` is a stream transducer: it consumes a
`Stream<T>`, emits zero, one, or many values per input item, and composes
into pipelines with `|>`. Everything else is deliberately plain.

Status: **design complete (simplified core, ADR 0008), implementation not
started.** Tree-walking interpreter (v1, Rust, zero deps) first;
compilation is v1.1.

```
type Vessel {
    name Str
    capacity Int
}

spawn vessels() Stream<Vessel> {
    emit Vessel { name "cup", capacity 100 }
    emit Vessel { name "jug", capacity 500 }
}

spawn big(input Stream<Vessel>) Stream<Vessel> {
    input -> v {
        if v.capacity >= 200 {
            emit v
        }
    }
}

spawn show(input Stream<Vessel>) {
    input -> v {
        print(v.name)
    }
}

vessels |> big |> show
```

## The load-bearing idea

Variable emit count makes one primitive cover filter, map, flatMap, scan,
batch, and window. Channels are implicit rendezvous; sources and sinks are
structural (no input param / no return slot); processes are first-class
values, so partial pipelines compose like functions.

A first, larger design added themed keywords, process mortality ("the
program cannot crash"), opt-in parallelism, and a named per-process GC.
All cut — see `docs/adr/0008-simplify-to-minimal-core.md`. Anything
unexpected now just panics.

## Repo map

| Path | What |
|---|---|
| `language-spec.md` | The spec — source of truth, includes the append-only decision log |
| `docs/adr/` | Architectural decision records (append-only) |
| `docs/design-rationale.md` | Why the simplified core looks the way it does; known weaknesses |
| `docs/plans/` | Implementation plans (the 17-task plan is superseded; new plan pending) |
| `possible-names.md` | Naming candidates from the first design |

CLI: `ichor run file.ich`.
