# proc design: subprocesses with runtime-pumped pipes

Implementation plan for ADR 0016. Same routine as the script stdlib:
goldens, spec section 15.12, implementation, one commit.

## Decisions pinned here

- `struct Cmd { argv []str, dir str, env map[str]str, stdin str,
  log str }` and `struct Result { status int, stdout str, stderr str }`,
  plus the opaque handle `Proc` (reference semantics, the Ctx/Re
  pattern, cannot be constructed with a literal).
- `cmd.env` MERGES into the inherited environment, entries winning over
  inherited values. Deliberate deviation from Go's replace-everything
  `Cmd.Env`: scripts that lose PATH because they set one variable is
  the footgun, and merge is what every shell invocation does. An empty
  map inherits unchanged.
- `cmd.stdin` is written to the child and closed before reading output
  (from a writer thread, so a child that never reads cannot deadlock
  the write).
- Exit semantics (ADR 0016): nonzero exit fills Result AND sets the
  error ("exit status 3"). A child terminated by a signal reports
  `status -1` and the error names the signal when known. Failure to
  spawn returns the zero Result and an error. A ctx that is already
  done returns before spawning.
- A live ctx bounds `run`/`exec`/`wait`/`readline`: when the deadline
  passes or SIGINT fires, `run`/`exec` terminate the child (terminate,
  2s grace, kill), then return the ctx error with output captured so
  far and `status -1`. `wait`/`readline` return the ctx error and
  leave the child alone; the handle stays valid.
- `proc.start` merges stderr into stdout as one stream, interleaved at
  line granularity in arrival order (two pump threads feed one queue;
  the spec promises line-granular interleave, not byte order).
  `cmd.log` instead routes the merged stream to a file (append),
  for watchers where nobody reads it in-language.
- Handle methods: `pid() int`, `running() bool`,
  `readline(c Ctx) (str, error?)` (eof is an error value, msg `eof`,
  the `os.readline` contract), `wait(c Ctx) (int, error?)`,
  `stop(grace int) error?` (terminate, wait `grace` nanoseconds, kill,
  reap; idempotent, stopping a dead child is `none`).
- Waiting is slice-polling (the `time.sleep` pattern, ~20ms slices,
  checking the ctx between slices). No async machinery.
- Threads live inside the module and only Strings cross them
  (`Mutex<VecDeque<String>>` plus an eof flag per child). No `Value`
  ever crosses a thread; the concurrency ledger's constraint one
  holds.
- wasm: the whole module reports absence, the os module contract.

## Integration map

The 0015 pattern exactly: STD_MODULES + sigs rows; struct injection
(Cmd, Result; Proc opaque) in typecheck collect and interp; method
table arm for Proc next to Ctx and Re; `Value::Proc(Arc<ProcInner>)`;
`src/stdlib/proc.rs`; mod.rs dispatch; spec 15.12; goldens under
tests/golden/stdlib/ using /bin/echo, /bin/sh, and sleep (the project
already assumes unix: libc, flock).

## Golden sketch

- run echo: status 0, stdout captured, none error.
- sh -c "exit 3": Result filled AND error set, both visible.
- missing binary: zero Result, error names it.
- exec with dir, env merge, stdin roundtrip (cat).
- start + readline until eof + wait: lines arrive, eof is an error
  value, wait returns 0.
- stop on a sleeping child: prompt, none error, running() flips.
- ctx timeout on a sleeping child: run returns the ctx error promptly
  with status -1.
