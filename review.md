  Critique
  Critical: the “checked programs cannot crash the host process” claim is too broad with embedded CPython. The spec and README promise no host crash/core dump (language-spec.md:39, README.md:37), while ADR 0002 explicitly embeds real CPython/C extensions in-process (docs/adr/0002-embedded-cpython-pyo3-uv.md:7).
  PyErr conversion does not contain native extension segfaults or aborts. Scope the claim to pure Nevla/Python exceptions, or isolate Python work out-of-process.

  High: map.delete is statically unsound. The checker types it as returning the map (src/typecheck/sigs.rs:183), but runtime returns Unit (src/builtins.rs:183). The new delete-return mode demonstrates a checked program faulting later with len needs str, list, or map.

  High: ADR 0015 is accepted but only partially true. It says ctx.timeout migrates to int nanoseconds and adds file.glob, file.modified, regex, and flag (docs/adr/0015-script-stdlib.md:50, docs/adr/0015-script-stdlib.md:54), while code still types ctx.timeout as Float (src/typecheck/sigs.rs:50) and implements f64
  seconds (src/stdlib/ctx.rs:76). Either mark the ADR partial/proposed or finish the semantic change.

  Medium: the agent primer is stale enough to teach invalid code: it lists ord, chr, input, and says clone is deep (docs/nevla-primer.md:121), while the spec now has charcode/char, no input builtin, and one-level clone (language-spec.md:1775, language-spec.md:1816). That matters because new projects scaffold this
  for coding agents.
