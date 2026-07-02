# 1. Themed keywords rationed by semantic weight

Status: superseded by 0008 (2026-07-01)

## Context

ichor has a blood-and-bone theme. The tempting move is theming everything,
including control flow. Every renamed universal concept (`if`, loops,
`return`) taxes every future reader for zero information gain.

## Decision

A themed keyword must carry semantic weight a plain word doesn't: `genesis`,
`spawn`, `emit`, `cide`, `clean`, `entomb`, `Vein`, `Autophage`. Universal
concepts keep universal names: `if`, `else`, `for`, `do`, `while`, `fn`,
`let`, `var`, `return`. Corollaries: no colons anywhere, positional
`name Type` annotations, no `->` return arrow, no `=>` in v1.

## Consequences

- The theme lands where it means something (lifecycle, concurrency,
  mortality) and never obstructs reading.
- Bans are permanent surface commitments: reintroducing `:` or `->`-as-return
  later would break the language's one visual identity rule.
- Spec §1, §8; rationale §1, §7.
