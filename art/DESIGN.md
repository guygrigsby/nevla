# nevla design

The single source for brand and visual decisions. Append decisions here;
don't scatter them through code comments.

## Mascot

Nevli the mongoose (`logo.png`; editable source `logo.af`, Affinity
Designer). The mascot is the brand: it appears in the README (top right,
200px, below the title so GitHub's h1 rule doesn't cut her) and the
playground header. Licensed CC BY 4.0 (see `art/LICENSE`); the code is MIT
and stays that way.

Named Rikki until 2026-07-10; renamed with the language (ADR 0014).
Same artwork.

## Color

Revised 2026-07-10: lighter overall, and a brand-tinted band across the
top of every page (go.dev-style). White text on `#c86fb9` was 3.3:1 and
failed AA on the primary buttons, so filled controls sit on the deep
purple instead; the bright purple exists because the deep one fails on
dark grounds. `tests/brand.rs` gates every hex on a site surface against
this list.

- Brand: `#c86fb9` (decided 2026-07-09), the mascot's purple. Use it for
  borders, focus rings, and accents; never for body text and never as a
  ground for white text (3.3:1, fails AA).
- Filled controls (primary buttons, run buttons): ground `#a94f9a`
  (4.9:1 with white), hover/active `#8d3f81`.
- Links and keywords: `#a94f9a` on light, `#d98fcd` on dark.
- Band (page-top wash): `#f6e7f2` light, `#382b34` dark.
- Light surfaces: background `#ffffff`, panels `#fbf7fa`, text `#2a2028`,
  muted `#6b5f68`, borders `#eadfe7`.
- Dark surfaces are plum-tinted, not gray: background `#231c21`, panels
  `#2e242b`, text `#ece5ea`, muted `#b1a1ac`, borders `#453842`.
- Search marks (mdBook): `#f3d9ee` light, `#5c3355` dark.
- Errors: `#d4526e`, one color for compile and runtime; the label carries
  the distinction.
- Both themes ship; `prefers-color-scheme` decides.

## Type

System stacks only, no webfonts: UI in the platform sans, code in the
platform mono. Code is the point of every nevla surface; it gets the
visual priority.

## Playground (decided 2026-07-09)

- Frontend-only WASM on GitHub Pages; no backend, ever. Sharing encodes
  the program into the URL fragment.
- A plain textarea editor until it hurts; no editor framework, no CDN
  dependencies. The page is fully self-contained. It hurt once already:
  compile errors were unfindable, so the textarea gained a synced line
  gutter that highlights diagnostic lines in the error color (wrap off,
  so gutter and text can never drift).
- The python bridge is honestly absent: the py example runs and shows the
  real "python is not available in this build" error rather than hiding
  the boundary.
