# 7. Rename to rikki

Status: accepted

## Context

The language shipped as mongoose. The name collided with the MongoDB ODM in
spirit and with the mg editor in practice: `.mg` sources and a `mongoose`
binary invite the wrong associations on both sides. The runner had already
been renamed once for adjacent reasons (mg → tavi → tk). With the runner
settled as `tk`, naming the language rikki completes the pair: language and
runner together form Rikki-Tikki, from the same Kipling lore the project
drew on from the start.

## Decision

Rename the language and everything it names; the runner `tk` keeps its name.

- Language name: mongoose → rikki (all prose, all case variants).
- Setup binary: `mongoose` → `rikki` (crate and package name, `[[bin]]`,
  CLI about-text).
- Runner binary: `tk` (unchanged).
- Manifest: `mongoose.toml` → `rikki.toml`; lock: `mongoose.lock` →
  `rikki.lock`; hidden env dir: `.mongoose/` → `.rikki/`.
- Source extension: `.mg` → `.rk` (goldens, examples, scaffolding, loader
  import suffix, harness discovery).
- Test gate env var: `MONGOOSE_TEST_PY` → `RIKKI_TEST_PY`.
- Internals: Rust crate paths `mongoose::` → `rikki::`, thread name
  `rikki-interp`, http User-Agent `rikki/0.1`, error text `rikki py add x`.

## Consequences

- Source files use the `.rk` extension; the manifest is `rikki.toml`.
- Existing projects migrate by renaming the manifest, lock, and sources
  (`mongoose.toml` → `rikki.toml`, `mongoose.lock` → `rikki.lock`,
  `*.mg` → `*.rk`); the `.mongoose/` venv is disposable and regenerates
  as `.rikki/` on the next run.
- Historical documents (ADRs 0001–0006, `docs/specs/`, `docs/plans/`)
  keep the old name; they are records, not references.
