# lmtk friction round two

Date: 2026-07-09. Status: approved. Source: lmtk agent's challenge list,
ordered by pain. Items 3 (math.exp/log, shipped earlier today as exp/ln/log)
and 5 (optional py imports; importlib through the bridge already expresses
it, backlog notes the possible first-class form) need no work here.

## 1. Dotted `import py "a.b"` binds the top-level segment

Today run_main inserts the module under the literal global name "a.b",
which no identifier can reference; the form is permitted and unusable.
Python's own semantics fix it: `import a.b` imports the submodule and binds
`a`.

- Runtime: import the full dotted path (loading the submodule and, per
  Python's import machinery, setting it as an attribute of the parent),
  then import and bind the top segment. `import py "os.path"` makes
  `os.path.join(...)` a working chain through the bound `os`.
- Checker: register the top segment as the py import name.
- Several dotted imports sharing a top segment collapse to one binding;
  a dotted and a bare import of the same top segment are the same binding.
- Manifest rule is already top-segment (17.5); unchanged.
- Spec 13.1 gains the binding rule.

## 2. `py(x)` conversion: py scalars and the None handle become writable

Spec 5.11 defines the py zero value (a None handle) but no expression
produces it, so a py-typed struct field cannot be initialized (struct
literals require every field; that rule stays). A conversion fits the
existing grammar: `py` is already a keyword, and `int(`/`str(`/`[]T(`
already mean conversion in expression position.

- `py(e)` accepts `int`, `float`, `bool`, `str`, and `none`; it yields
  plain `py` (no error slot; these inbound conversions cannot fail) via
  the bridge's existing inbound table. `py(none)` is the py zero value.
- Containers, structs, functions, and option-typed values are compile
  errors in v1 ("cannot convert X to py; pass it to a py call directly"):
  containers already convert at call boundaries, and the fallible cases
  (depth, cycles) stay out of an infallible expression.
- Spec: 7.7 gains the py target row; 5.8 cross-references it.

## 4. py-deps entries can map package name to module name

The manifest rule matches the import's top segment against declared
package names, so `mlflow-skinny` can never satisfy `import py "mlflow"`;
same wall for scikit-learn -> sklearn and pillow -> PIL. Declaration-side
override:

```toml
[py-deps]
mlflow-skinny = { version = "*", module = "mlflow" }
torch = "*"
```

- A py-dep value is a version string (unchanged) or a table with optional
  `version` (default "*") and optional `module` naming the import it
  satisfies.
- dep_declared: a dep with a module override satisfies exactly that
  module; without one, the normalized package name as today.
- uv still sees `name==version`; the module key is rikki-only.
- `rikki py add` keeps writing plain strings; the table form is a hand
  edit. save() round-trips it.
- Spec 17.5 documents the table form.

## Verification

Goldens: dotted import (`os.path.join`, stdlib so no manifest), py
conversion (zero-filled py struct field prints None, py(42)/py(true)
display through Python), container conversion compile error. Module
override is pinned by unit tests (dep_declared with and without override,
toml round-trip of the table form) since a positive runtime case would
need the package installed. Then the real path: lmtk drops its
os.environ.get("__miss__") workaround and takes mlflow-skinny.
