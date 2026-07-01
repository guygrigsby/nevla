# ichor v1 Interpreter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A complete tree-walking interpreter for ichor per `language-spec.md`, runnable as `ichor run file.ich`.

**Architecture:** Classic pipeline — lexer → Pratt parser → AST → nominal type checker → tree-walking evaluator — plus a CSP runtime mapping each spawn process to an OS thread with `sync_channel(0)` rendezvous channels. The checker guarantees all safety rules statically; the evaluator trusts it. Values are `Arc`-based with copy-on-write, implementing the spec's value semantics (§7).

**Tech Stack:** Rust stable, edition 2021, **zero external dependencies** (`[dependencies]` stays empty). Test with built-in `cargo test`.

## Global Constraints

- Spec is `language-spec.md` at repo root — the source of truth. Section refs (§N) below point into it.
- No external crates, ever, in v1. Stdlib covers everything (`std::thread`, `std::sync::mpsc::sync_channel(0)` for CSP rendezvous, `Arc::make_mut` for COW).
- Every value type is `Send` (use `Arc`, never `Rc`, inside `Value`). Per-thread evaluator state may use `Rc`/`RefCell`.
- No `:` in ichor source anywhere; no `->` as return separator; no `=>` (spec §8 punctuation table).
- Machine faults (Int div/mod by zero, Array/Str index out of bounds) are process death, never a Rust panic and never a wrapped value (§6.3).
- Set elements and Map keys are restricted to `Int`, `Str`, `Bool` (hashable prims; Float excluded). Checker-enforced.
- Commit after every task: terse, verb-first, module prefix (`lexer: ...`), no attribution trailers.
- Gate per task: `cargo test` green, `cargo fmt --check` clean, `cargo clippy -- -D warnings` clean.
- File extension: `.ich`.

## File Structure

```
Cargo.toml
src/lib.rs        # module declarations only
src/main.rs       # CLI (Task 17)
src/token.rs      # Token, TokenKind (Task 1)
src/lexer.rs      # Lexer (Task 2)
src/ast.rs        # AST + TypeExpr (Task 3)
src/parser.rs     # Pratt parser (Tasks 4-7)
src/types.rs      # checker's Type language (Task 8)
src/checker.rs    # nominal checker + safety rules (Tasks 8-10)
src/value.rs      # runtime Value, Env (Task 11)
src/interp.rs     # evaluator (Tasks 11-13)
src/runtime.rs    # processes, pipelines, death (Tasks 14-16)
examples/*.ich    # golden programs (Task 17)
tests/e2e.rs      # end-to-end golden tests (Task 17)
```

---

### Task 1: Scaffold + token module

**Files:**
- Create: `Cargo.toml`, `src/lib.rs`, `src/main.rs` (stub), `src/token.rs`

**Interfaces:**
- Produces: `token::TokenKind` (enum, all variants below), `token::Token { kind, literal: String, line: u32, col: u32 }`, `token::lookup_keyword(&str) -> Option<TokenKind>`

- [ ] **Step 1: Scaffold the crate**

```bash
cargo init --name ichor
```

`Cargo.toml`:
```toml
[package]
name = "ichor"
version = "0.1.0"
edition = "2021"

[dependencies]
```

`src/lib.rs`:
```rust
pub mod token;
```

`src/main.rs`:
```rust
fn main() {
    eprintln!("usage: ichor run <file.ich>");
    std::process::exit(2);
}
```

- [ ] **Step 2: Write the failing test** (bottom of `src/token.rs` in `#[cfg(test)] mod tests`)

```rust
#[test]
fn keywords_resolve() {
    assert_eq!(lookup_keyword("spawn"), Some(TokenKind::Spawn));
    assert_eq!(lookup_keyword("cide"), Some(TokenKind::Cide));
    assert_eq!(lookup_keyword("ossify"), Some(TokenKind::Ossify));
    assert_eq!(lookup_keyword("banana"), None);
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test` — Expected: compile error, `TokenKind` not defined.

- [ ] **Step 4: Implement `src/token.rs`**

The complete inventory (later tasks depend on these exact names):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // literals / identifiers
    Ident, Int, Float, Str,
    // keywords (§8)
    Let, Var, Fn, Return, If, Else, For, In, Do, While,
    Genesis, Spawn, Emit, Cide, Entomb, Clean, Auto, Ossify, True, False,
    // operators / punctuation
    Assign,            // =
    Plus, Minus, Star, Slash, Percent,
    Eq, NotEq, Lt, Gt, Le, Ge,          // == != < > <= >=
    And, Or, Bang,                      // && || !
    Arrow,             // ->  (reactive body only)
    Pipe,              // |>
    DotDot,            // ..
    Dot, Comma,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semi,              // statement terminator: auto-inserted at newline (see Task 2), or literal ';'
    Eof, Illegal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
    pub line: u32,
    pub col: u32,
}

pub fn lookup_keyword(ident: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match ident {
        "let" => Let, "var" => Var, "fn" => Fn, "return" => Return,
        "if" => If, "else" => Else, "for" => For, "in" => In,
        "do" => Do, "while" => While,
        "genesis" => Genesis, "spawn" => Spawn, "emit" => Emit,
        "cide" => Cide, "entomb" => Entomb, "clean" => Clean,
        "auto" => Auto, "ossify" => Ossify,
        "true" => True, "false" => False,
        _ => return None,
    })
}
```

- [ ] **Step 5: Verify pass and commit**

Run: `cargo test && cargo fmt --check && cargo clippy -- -D warnings` — Expected: 1 test passes.
```bash
git add -A && git commit -m "token: keyword and token inventory"
```

---

### Task 2: Lexer

**Files:**
- Create: `src/lexer.rs`; Modify: `src/lib.rs` (add `pub mod lexer;`)

**Interfaces:**
- Consumes: `token::{Token, TokenKind, lookup_keyword}`
- Produces: `lexer::Lexer::new(src: &str) -> Lexer`, `Lexer::next_token(&mut self) -> Token` (returns `Eof` forever at end)

**Automatic statement termination (Go-style ASI):** ichor has no written
semicolons, so the lexer inserts a `Semi` token at a newline when the previous
token can end a statement: `Ident, Int, Float, Str, True, False, Clean,
RParen, RBracket, RBrace, Return` (bare return). A literal `;` also lexes to
`Semi`. All other newlines are whitespace. This is what makes `let a Int = 1\nlet b Int = 2`
two statements while `let a Int =\n    1` still works.

Tricky spots the tests must pin down:
- `..` vs `.` vs float: `0..10` → `Int(0) DotDot Int(10)`; `0.5` → `Float`; `xs.len` → `Ident Dot Ident`. Rule: after digits, `.` followed by a digit → float; `.` followed by `.` → emit Int then DotDot.
- Two-char operators: `== != <= >= && || -> |>`. Single `|` or `&` → `Illegal`.
- `//` comments skipped to end of line. Strings: `"..."` with escapes `\"` `\\` `\n`. Unterminated string → `Illegal`.
- Line/col tracked for every token (1-based).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn lexes_representative_source() {
    let src = r#"
spawn double(input Vein<Int>) Vein<Int> {
    input -> item {
        emit item * 2   // per item
    }
}
let r Float = 0.5
for i in 0..10 { }
a |> b
let e Err = clean
"#;
    use crate::token::TokenKind::*;
    let expected = [
        Spawn, Ident, LParen, Ident, Ident, Lt, Ident, Gt, RParen,
        Ident, Lt, Ident, Gt, LBrace,
        Ident, Arrow, Ident, LBrace,
        Emit, Ident, Star, Int,
        RBrace, RBrace,
        Let, Ident, Ident, Assign, Float,
        For, Ident, In, Int, DotDot, Int, LBrace, RBrace,
        Ident, Pipe, Ident,
        Let, Ident, Ident, Assign, Clean,
        Eof,
    ];
    let mut lx = Lexer::new(src);
    for want in expected {
        // skip auto-inserted statement terminators; ASI has its own test
        let mut t = lx.next_token();
        while t.kind == crate::token::TokenKind::Semi {
            t = lx.next_token();
        }
        assert_eq!(t.kind, want);
    }
}

#[test]
fn asi_inserts_semi_after_statement_enders() {
    use crate::token::TokenKind::*;
    let kinds: Vec<_> = std::iter::from_fn({
        let mut lx = Lexer::new("let a Int = 1\nlet b Int = 2");
        move || match lx.next_token().kind { Eof => None, k => Some(k) }
    }).collect();
    assert_eq!(kinds, vec![Let, Ident, Ident, Assign, Int, Semi, Let, Ident, Ident, Assign, Int]);
    // no Semi after Assign: continuation across the newline
    let mut lx = Lexer::new("let a Int =\n1");
    let no_semi: Vec<_> = std::iter::from_fn(move || match lx.next_token().kind { Eof => None, k => Some(k) }).collect();
    assert!(!no_semi.contains(&Semi));
}

#[test]
fn tracks_position_and_illegal() {
    let mut lx = Lexer::new("let\n  x");
    let t = lx.next_token();
    assert_eq!((t.line, t.col), (1, 1));
    let t = lx.next_token();
    assert_eq!((t.line, t.col), (2, 3));
    assert_eq!(Lexer::new("&").next_token().kind, crate::token::TokenKind::Illegal);
}
```

Also add (full code in the same style): a test for string escapes (`"a\"b\n"` → literal `a"b` + newline) and one asserting `"0.5.3"` lexes `Float(0.5) Dot Int(3)`.

- [ ] **Step 2: Run to verify failure** — `cargo test lexer` — Expected: compile error, no `Lexer`.

- [ ] **Step 3: Implement the lexer**

Byte-walking struct; standard shape:

```rust
pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: u32,
    col: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self { Lexer { src: src.as_bytes(), pos: 0, line: 1, col: 1 } }

    fn peek(&self) -> u8 { *self.src.get(self.pos).unwrap_or(&0) }
    fn peek2(&self) -> u8 { *self.src.get(self.pos + 1).unwrap_or(&0) }

    fn bump(&mut self) -> u8 {
        let c = self.peek();
        self.pos += 1;
        if c == b'\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        c
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws_and_comments();
        let (line, col) = (self.line, self.col);
        let mk = |kind, lit: &str| Token { kind, literal: lit.to_string(), line, col };
        use TokenKind::*;
        match self.peek() {
            0 => mk(Eof, ""),
            b'=' if self.peek2() == b'=' => { self.bump(); self.bump(); mk(Eq, "==") }
            b'=' => { self.bump(); mk(Assign, "=") }
            b'!' if self.peek2() == b'=' => { self.bump(); self.bump(); mk(NotEq, "!=") }
            b'!' => { self.bump(); mk(Bang, "!") }
            b'-' if self.peek2() == b'>' => { self.bump(); self.bump(); mk(Arrow, "->") }
            b'|' if self.peek2() == b'>' => { self.bump(); self.bump(); mk(Pipe, "|>") }
            b'&' if self.peek2() == b'&' => { self.bump(); self.bump(); mk(And, "&&") }
            b'|' if self.peek2() == b'|' => { self.bump(); self.bump(); mk(Or, "||") }
            b'<' if self.peek2() == b'=' => { self.bump(); self.bump(); mk(Le, "<=") }
            b'>' if self.peek2() == b'=' => { self.bump(); self.bump(); mk(Ge, ">=") }
            b'.' if self.peek2() == b'.' => { self.bump(); self.bump(); mk(DotDot, "..") }
            b'"' => self.read_string(line, col),
            c if c.is_ascii_digit() => self.read_number(line, col),
            c if c.is_ascii_alphabetic() || c == b'_' => self.read_ident(line, col),
            _ => { /* single-char table: + - * / % < > . , ( ) { } [ ] else Illegal */
                   let c = self.bump(); self.single(c, line, col) }
        }
    }
}
```

`read_number`: consume digits; if `peek() == b'.' && peek2().is_ascii_digit()` consume the dot and digits → `Float`; else `Int` (leaving `..` intact). `read_ident`: consume `[A-Za-z0-9_]`, then `lookup_keyword` else `Ident`. Write `single`, `read_string` (escapes `\" \\ \n`; EOF before close → `Illegal`), and `skip_ws_and_comments` (`//` to newline) fully.

- [ ] **Step 4: Verify pass** — `cargo test lexer` — Expected: all lexer tests pass.

- [ ] **Step 5: Commit** — `git add -A && git commit -m "lexer: tokenize full ichor surface"`

---

### Task 3: AST module

**Files:**
- Create: `src/ast.rs`; Modify: `src/lib.rs`

**Interfaces:**
- Produces the exact node vocabulary every later task uses:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq)]
pub struct Binding { pub name: String, pub ty: TypeExpr }

#[derive(Debug, Clone, PartialEq)]
pub enum Workers { Seq, Fixed(u32), Auto }

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,               // "" for anonymous fn literals
    pub params: Vec<Binding>,
    pub ret: Vec<TypeExpr>,         // 0 = returns nothing; >1 = multi-return
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnDecl {
    pub name: String,               // "" for anonymous spawn literals
    pub workers: Workers,
    pub input: Option<Binding>,     // None = source
    pub output: Option<TypeExpr>,   // None = sink
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { mutable: bool, bindings: Vec<Binding>, value: Expr, line: u32 },
    Assign { target: Expr, value: Expr, line: u32 },   // target: Ident | Dot | Index
    Return { values: Vec<Expr>, line: u32 },
    Expr(Expr),
    Genesis { name: String, fields: Vec<Binding>, entombed: bool, line: u32 },
    Fn(FnDecl),
    Spawn(SpawnDecl),
    If { cond: Expr, then: Block, els: Option<Block> },
    ForIn { var: String, iter: Expr, body: Block, line: u32 },
    DoWhile { body: Block, cond: Expr },
    Emit { value: Expr, line: u32 },
    Cide { msg: Expr, line: u32 },
    Reactive { stream: String, item: String, body: Block, line: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Int(i64), Float(f64), Str(String), Bool(bool), CleanLit,
    Prefix { op: PrefixOp, rhs: Box<Expr> },
    Infix { op: InfixOp, lhs: Box<Expr>, rhs: Box<Expr>, line: u32 },
    Call { callee: Box<Expr>, args: Vec<Expr>, line: u32 },
    Index { obj: Box<Expr>, idx: Box<Expr>, line: u32 },
    Dot { obj: Box<Expr>, field: String, line: u32 },
    FnLit(Box<FnDecl>),
    SpawnLit(Box<SpawnDecl>),
    Construct { ty: String, fields: Vec<(String, Expr)>, line: u32 },
    ArrayLit(Vec<Expr>),
    SetLit(Vec<Expr>),
    MapLit(Vec<(Expr, Expr)>),
    EmptyBraces { id: u32 },        // `{}` — checker resolves Set vs Map by annotation
    Range { lo: Box<Expr>, hi: Box<Expr> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixOp { Neg, Not }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp { Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, Gt, Le, Ge, And, Or, Pipe }

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named(String),                             // Int, Str, Vessel, ...
    Generic(String, Vec<TypeExpr>),            // Vein<Int>, Map<Str, Int>
    Fn(Vec<TypeExpr>, Vec<TypeExpr>),          // fn(Int) (Int, Err)
    Spawn(Option<Box<TypeExpr>>, Option<Box<TypeExpr>>), // spawn(Vein<Int>) Vein<Int>
}
```

- [ ] **Step 1: Write the failing test** — a smoke test that constructs `Stmt::Let` for `let count Int = 0` by hand and asserts equality with itself after `.clone()` (locks `PartialEq + Clone` derives).

- [ ] **Step 2: Run to verify failure** — `cargo test ast` — compile error.

- [ ] **Step 3: Implement** exactly the code above.

- [ ] **Step 4: Verify pass** — `cargo test ast`.

- [ ] **Step 5: Commit** — `git add -A && git commit -m "ast: node vocabulary for full language"`

---

### Task 4: Parser core — Pratt expressions + let/var/assign/return

**Files:**
- Create: `src/parser.rs`; Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `lexer::Lexer`, `ast::*`, `token::*`
- Produces: `parser::Parser::new(Lexer) -> Parser`, `Parser::parse_program(&mut self) -> Result<Program, Vec<ParseError>>`, `pub struct ParseError { pub msg: String, pub line: u32, pub col: u32 }`
- Internal (used by Tasks 5-7): `parse_stmt`, `parse_block`, `parse_expr(prec: Prec)`, `parse_type_expr`, and the `no_construct` flag (see below)

Precedence (low→high):
```rust
enum Prec { Lowest, Pipe, Or, And, Equals, Compare, Range, Sum, Product, Prefix, Call }
// Pipe: |>   Equals: == !=   Compare: < > <= >=   Range: ..
// Call covers call/index/dot postfix
```

**Construction ambiguity rule (Go's solution):** `IDENT {` parses as `Construct` only when the parser is *not* in a condition/iterable position. `parse_expr` takes the flag through a parser field `no_construct: bool`, set while parsing `if` conditions, `do/while` conditions, and `for ... in` iterables, cleared inside any parenthesized/braced subexpression.

**Statement separation:** statements end at `Semi` (auto-inserted or literal),
`}` or `Eof`; the parser skips redundant `Semi`s between statements. Construct
and genesis field lists accept `Comma` or `Semi` as separators (that's the
"one per line, no trailing separators" spec behavior).

Statement dispatch in `parse_stmt`:
- `let`/`var` → Let (multi-binding: `let a Int, b Err = expr` — bindings comma-separated, one `=`, one value expr)
- `return` → Return (comma-separated exprs until end of line/`}`)
- IDENT followed by `->` → Reactive (Task 7)
- otherwise parse an expression; if next token is `=` → Assign (validate target is Ident/Dot/Index, else error) else ExprStmt

- [ ] **Step 1: Write the failing tests**

```rust
fn parse_ok(src: &str) -> crate::ast::Program {
    let p = Parser::new(crate::lexer::Lexer::new(src));
    p.parse_program().expect("parse failed")
}

#[test]
fn parses_let_with_annotation() {
    let prog = parse_ok("let count Int = 40 + 2");
    use crate::ast::*;
    let Stmt::Let { mutable, bindings, value, .. } = &prog.0[0] else { panic!() };
    assert!(!mutable);
    assert_eq!(bindings[0], Binding { name: "count".into(), ty: TypeExpr::Named("Int".into()) });
    assert!(matches!(value, Expr::Infix { op: InfixOp::Add, .. }));
}

#[test]
fn parses_multi_binding_let() {
    let prog = parse_ok("let r Int, err Err = divide(10, 0)");
    let crate::ast::Stmt::Let { bindings, .. } = &prog.0[0] else { panic!() };
    assert_eq!(bindings.len(), 2);
}

#[test]
fn precedence() {
    // (1 + (2 * 3)) < 10  &&  x == y
    let prog = parse_ok("1 + 2 * 3 < 10 && x == y");
    // assert the root is And, its lhs is Lt, Lt's lhs is Add whose rhs is Mul
}

#[test]
fn postfix_chain() {
    let prog = parse_ok("v.field[0].len()");
    // root: Call{ callee: Dot{ obj: Index{ obj: Dot{...} } } }
}
```

Plus (full code, same pattern): assign to dot-path `v.capacity = v.capacity - amount`; range `0..xs.len()` parses `Range{lo, hi: Call{..}}`; parse error carries line/col.

- [ ] **Step 2: Run to verify failure** — `cargo test parser`.

- [ ] **Step 3: Implement**

Standard two-token-lookahead Pratt parser:

```rust
pub struct Parser<'a> {
    lx: Lexer<'a>,
    cur: Token,
    peek: Token,
    errors: Vec<ParseError>,
    no_construct: bool,
    brace_id: u32,          // for Expr::EmptyBraces
}
```

`parse_expr(prec)`: prefix dispatch on `cur.kind` (Int/Float/Str literals, `true/false/clean`, Ident → Ident-or-Construct (respecting `no_construct`), `(` grouped, `-`/`!` prefix, `[` array literal (Task 5), `{` set/map literal (Task 5), `fn` literal (Task 5), `spawn` literal (Task 7)); then loop while `prec < infix_prec(peek)`: `..` → Range, `(` → Call, `[` → Index, `.` → Dot, else Infix (including `|>` as `InfixOp::Pipe`).

`parse_type_expr`: Ident, optionally `<` type-list `>` → Generic; `fn` `(` types `)` [ret-types] → `TypeExpr::Fn`; `spawn` `(` [type] `)` [type] → `TypeExpr::Spawn`. Return-type slot detection after `)`: a return type is present iff the next token starts a type (`Ident`, `fn`, `spawn`, `(` for multi-return list).

- [ ] **Step 4: Verify pass** — `cargo test parser`.

- [ ] **Step 5: Commit** — `git add -A && git commit -m "parser: pratt core, let/assign/return"`

---

### Task 5: Parser — declarations and literals

**Files:**
- Modify: `src/parser.rs`

**Interfaces:**
- Produces parsing for: `genesis`/`entomb` decls, named `fn` decls, `fn` literals, `Construct`, `ArrayLit`/`SetLit`/`MapLit`/`EmptyBraces`, `ossify` (reserved → parse error "ossify is reserved")

Grammar notes:
- `genesis IDENT { (IDENT TypeExpr)* }` — fields newline- or comma-separated. `entomb` identical, sets `entombed: true`.
- `fn IDENT ( params ) [ret] Block`, params `IDENT TypeExpr` comma-separated; ret slot per Task 4 rule; multi-return `( T1, T2 )`.
- Construct: `IDENT { (IDENT expr) , ... }` — field name followed by expression (juxtaposition), comma or newline separated.
- Brace literal in expression position: `{}` → `EmptyBraces{id}`; `{ e1, e2 }` → SetLit; `{ e1 e2, e3 e4 }` → MapLit (after first element expr, if next token is `,` or `}` it's a Set, else parse the value expr — Map).

- [ ] **Step 1: Write the failing tests** — full code, one per form:

```rust
#[test]
fn parses_genesis() {
    let prog = parse_ok("genesis Vessel {\n name Str\n capacity Int\n sealed Bool\n}");
    let crate::ast::Stmt::Genesis { name, fields, entombed, .. } = &prog.0[0] else { panic!() };
    assert_eq!(name, "Vessel");
    assert_eq!(fields.len(), 3);
    assert!(!entombed);
}

#[test]
fn parses_construction() {
    let prog = parse_ok(r#"let v Vessel = Vessel { name "cup", capacity 100, sealed false }"#);
    // Let value is Construct with 3 fields in written order
}

#[test]
fn parses_fn_literal_with_fn_type() {
    let prog = parse_ok("let square fn(Int) Int = fn(x Int) Int { return x * x }");
    // binding ty is TypeExpr::Fn([Named Int],[Named Int]); value is FnLit
}

#[test]
fn parses_collection_literals() {
    parse_ok(r#"let a Array<Int> = [1, 2, 3]"#);
    parse_ok(r#"let s Set<Str> = {"a", "b"}"#);
    parse_ok(r#"let m Map<Str, Int> = { "ada" 36, "linus" 55 }"#);
    parse_ok(r#"let e Set<Int> = {}"#);   // EmptyBraces
}

#[test]
fn ossify_is_reserved() {
    let p = Parser::new(crate::lexer::Lexer::new("ossify x Int = 1"));
    assert!(p.parse_program().is_err());
}
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement** the parse functions per the grammar notes; multi-return ret list: after `)` if peek is `(` parse parenthesized comma-separated type list.
- [ ] **Step 4: Verify pass** — `cargo test parser`.
- [ ] **Step 5: Commit** — `git add -A && git commit -m "parser: genesis, fn, construction, collection literals"`

---

### Task 6: Parser — control flow

**Files:**
- Modify: `src/parser.rs`

**Interfaces:**
- Produces parsing for: `If` (no parens, `else` optional incl. `else if` via nested If in a one-stmt Block), `ForIn`, `DoWhile` (`do Block while ( expr )` — parens required per spec §5)

`no_construct` is set while parsing: if condition, do-while condition, for-in iterable.

- [ ] **Step 1: Write the failing tests** — full code for each:

```rust
#[test]
fn parses_if_else_without_parens() {
    parse_ok("if item % 2 == 0 { emit item } else { }");
}

#[test]
fn if_condition_does_not_eat_construction() {
    // `ok` here must parse as Ident condition, `{` opens the block
    parse_ok("if ok { return 1 }");
}

#[test]
fn parses_for_in_range_and_collection() {
    parse_ok("for i in 0..10 { }");
    parse_ok("for item in xs { }");
}

#[test]
fn parses_do_while() {
    parse_ok("do { x = x + 1 } while (x < 10)");
}
```

- [ ] **Step 2: Run to verify failure.** — new tests fail.
- [ ] **Step 3: Implement.** `while` outside `do` → parse error ("standalone while was dropped; use do/while or for").
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "parser: if, for-in, do-while"`

---

### Task 7: Parser — spawn, reactive, emit, cide, pipelines

**Files:**
- Modify: `src/parser.rs`

**Interfaces:**
- Produces parsing for: named `SpawnDecl` (with `spawn<8>`/`spawn<auto>` worker clause), anonymous `SpawnLit` expressions, `Reactive`, `Emit`, `Cide`, and `|>` chains (already an infix — test coverage here)

Grammar notes:
- `spawn [< INT|auto >] IDENT ( [param] ) [TypeExpr] Block` — 0 or 1 params (source has none). Anonymous form: `spawn [<...>] ( [param] ) [TypeExpr] Block` in expression position (distinguished from named decl by `(` after `spawn`/worker clause instead of IDENT).
- Reactive statement (inside any block; checker enforces spawn-only): `IDENT -> IDENT Block`.
- `emit expr` and `cide ( expr )` — cide takes a parenthesized Str expr per spec examples.

- [ ] **Step 1: Write the failing tests** — full code:

```rust
#[test]
fn parses_spawn_decl_sequential() {
    let prog = parse_ok("spawn double(input Vein<Int>) Vein<Int> { input -> item { emit item * 2 } }");
    let crate::ast::Stmt::Spawn(d) = &prog.0[0] else { panic!() };
    assert_eq!(d.workers, crate::ast::Workers::Seq);
    assert!(d.input.is_some() && d.output.is_some());
    assert!(matches!(d.body.0[0], crate::ast::Stmt::Reactive { .. }));
}

#[test]
fn parses_worker_clauses() {
    parse_ok("spawn<8> f(input Vein<Int>) Vein<Int> { }");
    parse_ok("spawn<auto> f(input Vein<Int>) Vein<Int> { }");
}

#[test]
fn parses_source_sink_and_pipeline() {
    let prog = parse_ok("spawn nums() Vein<Int> { emit 1 }\nspawn show(input Vein<Int>) { input -> item { } }\nnums |> double |> show");
    // last stmt: Expr(Infix{op: Pipe, lhs: Infix{op: Pipe, ..}, ..})
}

#[test]
fn parses_anonymous_spawn_in_pipeline() {
    parse_ok("nums |> spawn(input Vein<Int>) { input -> item { emit item } }");
}

#[test]
fn parses_cide() {
    parse_ok(r#"spawn f(input Vein<Str>) Vein<Int> { input -> item { cide("bad") } }"#);
}
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Verify pass** — the whole parser suite green.
- [ ] **Step 5: Commit** — `git add -A && git commit -m "parser: spawn, reactive, emit, cide, pipelines"`

---

### Task 8: Type language + checker core (bindings, expressions, nominal rules)

**Files:**
- Create: `src/types.rs`, `src/checker.rs`; Modify: `src/lib.rs`

**Interfaces:**
- Produces `types::Type`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int, Float, Str, Bool, ErrT, Void,
    Struct(String),                                  // nominal: identity is the name
    Array(Box<Type>), Set(Box<Type>), Map(Box<Type>, Box<Type>), Vein(Box<Type>),
    Fn(Vec<Type>, Vec<Type>),
    Spawn(Option<Box<Type>>, Option<Box<Type>>),     // stream element types
}
```

- Produces `checker::check(prog: &Program) -> Result<CheckInfo, Vec<CheckError>>` where:

```rust
pub struct CheckError { pub msg: String, pub line: u32 }
pub struct CheckInfo {
    pub empty_braces: std::collections::HashMap<u32, BraceKind>, // id → Set or Map
}
pub enum BraceKind { SetK, MapK }
```

- Internal structure both later checker tasks extend:

```rust
struct Checker {
    structs: HashMap<String, StructDef>,      // fields + entombed, collected in pass 1
    scopes: Vec<HashMap<String, VarInfo>>,    // VarInfo { ty: Type, mutable: bool }
    ctx: Vec<Ctx>,
    errors: Vec<CheckError>,
    info: CheckInfo,
}
enum Ctx {
    Fn { rets: Vec<Type> },
    Spawn { out: Option<Type>, parallel: bool, in_reactive: bool,
            pre_reactive_vars: HashSet<String> },
}
```

Two passes: (1) collect all `genesis`/`entomb`, named `fn`, named `spawn` signatures into globals; (2) check every body with `expr_type(&mut self, e: &Expr, expected: Option<&Type>) -> Type` (the `expected` drives EmptyBraces resolution and literal element checks).

Rules in this task:
- every binding annotated; initializer type must equal annotation (nominal, no Int/Float mixing, no implicit conversions anywhere)
- assignment requires `var` binding (or field/index place rooted at a `var`); type must match
- arithmetic ops: Int×Int→Int, Float×Float→Float; `+` also Str×Str→Str; comparisons on Int/Float/Str; `==`/`!=` on Int, Float, Str, Bool, and ErrT (for `err != clean`); `&&`/`||` on Bool; unknown identifier, redeclaration in same scope → errors
- Set elements / Map keys restricted to Int, Str, Bool
- `Range` only legal as a `for ... in` iterable

- [ ] **Step 1: Write the failing tests**

```rust
fn check_src(src: &str) -> Result<crate::checker::CheckInfo, Vec<crate::checker::CheckError>> {
    let prog = Parser::new(Lexer::new(src)).parse_program().unwrap();
    crate::checker::check(&prog)
}

#[test]
fn accepts_well_typed_bindings() {
    assert!(check_src("let a Int = 1 + 2\nvar b Str = \"x\"\nb = b + \"y\"").is_ok());
}

#[test]
fn rejects_type_mismatch_and_let_mutation() {
    assert!(check_src("let a Int = \"nope\"").is_err());
    assert!(check_src("let a Int = 1\na = 2").is_err());
    assert!(check_src("let a Int = 1\nlet b Float = 2.0\nlet c Float = a + b").is_err()); // no mixing
}

#[test]
fn nominal_not_structural() {
    let src = "genesis A { x Int }\ngenesis B { x Int }\nlet a A = A { x 1 }\nlet b B = a";
    assert!(check_src(src).is_err());
}

#[test]
fn resolves_empty_braces_from_annotation() {
    let info = check_src("let s Set<Int> = {}").unwrap();
    assert_eq!(info.empty_braces.len(), 1);
}

#[test]
fn float_set_elements_rejected() {
    assert!(check_src("let s Set<Float> = {}").is_err());
}
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement** per the rules above.
- [ ] **Step 4: Verify pass** — `cargo test checker`.
- [ ] **Step 5: Commit** — `git add -A && git commit -m "checker: nominal core, bindings, expressions"`

---

### Task 9: Checker — functions, multi-return, construction, collections, methods

**Files:**
- Modify: `src/checker.rs`

**Interfaces:**
- Extends `expr_type` / `check_stmt` with: calls (incl. fn-typed values and the `Err(Str) -> ErrT` builtin and `print(...)` accepting any args, returning Void), `return` arity/type checks against `Ctx::Fn`, construction rules, dot-method signatures, comma-ok map indexing, fn literals (params/rets from the literal; body checked in a fresh fn ctx; **capture rule: closures see outer `let`s and `var`s as immutable copies — assignment to a captured outer name inside a literal is an error**)

Hardcoded method table (checker AND interp use these exact names):

| Receiver | Method | Signature |
|---|---|---|
| `Array<T>` | `len()` | `() Int` |
| `Array<T>` | `push(v T)` | mutates receiver (requires var-rooted place) |
| `Array<T>` | `map(f fn(T) U)` | `Array<U>` |
| `Array<T>` | `fold(init U, f fn(U, T) U)` | `U` |
| `Str` | `len()` | `() Int` |
| `Set<T>` | `add(v T)` | mutates receiver |
| `Set<T>` | `has(v T)` | `() Bool` |
| `Set<T>` | `len()` | `() Int` |
| `Map<K,V>` | `len()` | `() Int` |
| `Err` | `.msg` field | `Str` |

Construction rules: type must exist, not entombed, every field present exactly once, field types match. Map indexing: **only** legal as the value of a two-binding let (`let v V, ok Bool = m[k]`) — anywhere else is an error "map access requires comma-ok destructure". Map index *assignment* (`m[k] = v`) is legal. Array indexing: single-value, Int index.

- [ ] **Step 1: Write the failing tests** — full code following the `check_src` pattern; cases:
  - fn decl + correct call OK; wrong arity/arg type/return use → errors
  - `fn f(a Int, b Int) (Int, Err)` with `return a / b, clean` OK; `let r Int, e Err = f(1,2)` OK; single-binding destructure of multi-return → error
  - `return` with wrong arity vs `rets` → error; missing return type usage `let x Int = log("hi")` → error (Void)
  - entombed construction → error; missing/extra field → error
  - `let v Int, ok Bool = m["k"]` OK; `let v Int = m["k"]` → error; `m["k"] = 3` OK (var map)
  - `nums.map(fn(n Int) Int { return n * 2 })` → `Array<Int>`; wrong fn shape → error
  - closure capture: `let a Int = 1` then literal body reading `a` OK; literal body assigning outer `var` → error
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "checker: fns, multi-return, construction, methods"`

---

### Task 10: Checker — spawn, streams, pipelines, safety rules

**Files:**
- Modify: `src/checker.rs`

**Interfaces:**
- Extends checking with everything §4/§6 requires statically:

Rules (each gets a test):
1. `emit` only inside spawn; emitted type matches the spawn's output elem type; `emit` in a sink (no output) → error
2. `cide` only inside spawn; argument must be Str
3. Reactive stmt only inside spawn; `stream` name must be the spawn's input param; item binding gets elem type; reactive in a source (no input) → error
4. **Parallel state rule (§4.5):** in a `spawn<N>`/`spawn<auto>` body, assignment inside the reactive block to a `var` declared in the spawn body *outside* the reactive block → error (track `pre_reactive_vars`)
5. `|>` composition: lhs must be a spawn-typed value with an output, rhs with an input, elem types equal; result type joins outer ends (`Spawn(lhs.in, rhs.out)`)
6. A pipeline/process value used as an expression statement must be complete (`Spawn(None, None)`) → else error "pipeline has open ends"; complete process values may be bound to `spawn()`-annotated lets
7. Named spawn declarations register as values of their `Spawn` type; anonymous `SpawnLit` exprs get their type from their signature; spawn literal bodies check under `Ctx::Spawn`
8. `return` inside spawn → error ("processes emit, they don't return")

- [ ] **Step 1: Write the failing tests** — full code; one focused test per rule above, e.g.:

```rust
#[test]
fn parallel_cross_item_state_rejected() {
    let src = "spawn<8> f(input Vein<Int>) Vein<Int> {\n var sum Int = 0\n input -> item {\n  sum = sum + item\n  emit sum\n }\n}";
    assert!(check_src(src).is_err());
    let seq = src.replace("spawn<8>", "spawn");
    assert!(check_src(&seq).is_ok());
}

#[test]
fn pipeline_type_mismatch_rejected() {
    let src = "spawn ints() Vein<Int> { emit 1 }\nspawn strs(input Vein<Str>) { input -> s { } }\nints |> strs";
    assert!(check_src(src).is_err());
}

#[test]
fn open_pipeline_statement_rejected() {
    let src = "spawn ints() Vein<Int> { emit 1 }\nspawn dbl(input Vein<Int>) Vein<Int> { input -> i { emit i } }\nints |> dbl";
    assert!(check_src(src).is_err()); // no sink: open end
}
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Verify pass** — full checker suite green.
- [ ] **Step 5: Commit** — `git add -A && git commit -m "checker: spawn safety rules and pipeline typing"`

---

### Task 11: Value model + evaluator core (expressions, bindings)

**Files:**
- Create: `src/value.rs`, `src/interp.rs`; Modify: `src/lib.rs`

**Interfaces:**
- Produces `value::Value` (everything `Send + Clone`; clones are cheap — COW):

```rust
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64), Float(f64), Str(Arc<String>), Bool(bool),
    Clean, ErrV(Arc<String>),
    StructV { ty: Arc<String>, fields: Arc<HashMap<String, Value>> },
    ArrayV(Arc<Vec<Value>>),
    SetV(Arc<HashSet<Key>>),
    MapV(Arc<HashMap<Key, Value>>),
    FnV(Arc<Closure>),
    ProcV(Arc<Vec<Stage>>),          // a (partial) pipeline: ordered stages
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key { Int(i64), Str(String), Bool(bool) }

#[derive(Debug)]
pub struct Closure {
    pub params: Vec<String>,
    pub body: crate::ast::Block,
    pub captured: HashMap<String, Value>,   // capture-by-copy snapshot (§3.3)
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub name: String,                        // for death reports; "" if anonymous
    pub workers: crate::ast::Workers,
    pub input: Option<String>,               // input param name; None = source
    pub has_output: bool,
    pub body: crate::ast::Block,
    pub captured: HashMap<String, Value>,
}
```

- Produces `interp::Interp`:

```rust
pub struct Globals {
    pub info: crate::checker::CheckInfo,
    // named fns/spawns/structs collected from the Program at load
}
pub struct Interp { pub globals: Arc<Globals> }

pub enum Signal { Return(Vec<Value>), Death { msg: String, line: u32 } }
pub type Eval<T> = Result<T, Signal>;

impl Interp {
    pub fn eval_expr(&self, e: &Expr, env: &mut Env) -> Eval<Value>;
    pub fn eval_block(&self, b: &Block, env: &mut Env) -> Eval<()>;
}
```

- `Env`: per-thread scope stack `Vec<HashMap<String, Value>>` with `get`, `declare`, `assign_place`. (Plain `HashMap`s — no `Rc<RefCell>` needed because closures capture by copy, not by reference.)
- **Place resolution** (shared by Assign, `push`, `add`, map-index-assign): resolve target to a mutable slot via `Arc::make_mut` at each hop — root Ident slot in `Env`, then `StructV` field hops / `MapV` key hop / `ArrayV` index hop. Index OOB during place walk → `Signal::Death`.

Machine faults produced here (as `Signal::Death`, message text exact): `"division by zero"`, `"modulo by zero"` (Int `/`/`%` with 0 rhs), `"index N out of bounds (len L)"`. Float division by zero is IEEE (∞), not a fault.

- [ ] **Step 1: Write the failing tests**

```rust
fn eval_program(src: &str) -> Result<Env, Signal> { /* parse, check, run top-level stmts in fresh Env */ }

#[test]
fn arithmetic_and_bindings() {
    let env = eval_program("let a Int = 6 * 7").unwrap();
    assert!(matches!(env.get("a"), Some(Value::Int(42))));
}

#[test]
fn cow_value_semantics() {
    // b copies a; mutating b leaves a intact
    let src = "genesis P { x Int }\nlet a P = P { x 1 }\nvar b P = a\nb.x = 99";
    let env = eval_program(src).unwrap();
    // a.x == 1, b.x == 99
}

#[test]
fn int_division_by_zero_is_death() {
    let err = eval_program("let a Int = 1 / 0").unwrap_err();
    assert!(matches!(err, Signal::Death { .. }));
}
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement** `eval_expr` for: literals, Ident, Prefix, Infix (arith/compare/logic with short-circuit `&&`/`||`), Construct, ArrayLit/SetLit/MapLit/EmptyBraces (via `globals.info.empty_braces`), Index (array/str read), Dot (struct field, `Err.msg`), Range (only reached via ForIn — represent as evaluated bounds there, no `Value` variant needed); `eval_block` for Let/Assign/Expr.
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "interp: value model, env, expression eval"`

---

### Task 12: Evaluator — functions, closures, control flow

**Files:**
- Modify: `src/interp.rs`

**Interfaces:**
- Extends eval with: Call (user fns by name, fn values, multi-return producing `Vec<Value>` consumed by multi-binding Let), FnLit (snapshot visible env into `Closure.captured`), Return signal, If, ForIn (over Array elements, Set keys, Range bounds; Str yields 1-char Strs), DoWhile.

- [ ] **Step 1: Write the failing tests** — full code; cases: recursive `fn` (factorial via program that stores result in a binding); closure capture-by-copy (`var n Int = 1`, make closure reading n, set `n = 2`, closure still sees 1); multi-return destructure end-to-end (`divide(10, 2)` → 5, clean); `for i in 0..5` sums to 10; do-while runs at least once.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.** Call semantics: bind params as immutable copies in a fresh env whose base is the closure's `captured` map; execute body; `Signal::Return(vals)` unwinds to the call.
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "interp: calls, closures, control flow"`

---

### Task 13: Evaluator — builtins, methods, Err/clean

**Files:**
- Modify: `src/interp.rs`

**Interfaces:**
- Produces: `print(...)` (writes each arg's display form + newline to a `Write` sink held by `Interp` — injectable for tests, stdout in `main`); `Err(msg)` constructor; the full method table from Task 9 (`len/push/map/fold/add/has`); comma-ok map read; map index assign; display formatting for every `Value` variant (`Int` → `42`, `Str` → raw, `ErrV` → `Err("msg")`, `Clean` → `clean`, arrays as `[1, 2]`, structs as `Vessel { name "cup", ... }`).

- [ ] **Step 1: Write the failing tests** — full code; cases: `print` output captured and asserted; `push` grows a `var` array but a copy taken before push is unchanged (COW proof through a method); `map`/`fold` with fn literals; `has`/`add` on sets; comma-ok present/absent; `err.msg`; `err != clean` both ways.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "interp: builtins, collection methods, err semantics"`

---

### Task 14: Runtime — sequential processes and pipelines

**Files:**
- Create: `src/runtime.rs`; Modify: `src/lib.rs`, `src/interp.rs` (Pipe infix + spawn decls/literals eval to `ProcV`; complete `ProcV` as expression statement invokes the runtime)

**Interfaces:**
- Produces:

```rust
pub struct DeathReport { pub stage: String, pub msg: String, pub line: u32 }
pub fn run_pipeline(stages: &[Stage], interp: &Interp) -> Vec<DeathReport>;
```

- Consumes: `value::{Stage, Value}`, `interp::{Interp, Signal}`

Mechanics (this is the heart — implement exactly):
- `stages[0]` has no input; last stage has no output (checker guaranteed).
- Between consecutive stages: `std::sync::mpsc::sync_channel::<Value>(0)` — rendezvous (§4.4).
- One `std::thread` per sequential stage. Thread body: fresh `Env` seeded from `stage.captured`; bind an emit sink (the stage's `SyncSender`) into the interp context for `Stmt::Emit`; execute `stage.body` statements in order. A `Reactive` stmt loops `for item in rx.iter()` binding the item name each iteration. When the body ends, the sender drops → downstream sees close (**auto-close, §4.4**).
- `emit` when the receiver has hung up (`send` returns `Err`): stop emitting, keep consuming input (drain mode) so upstream never deadlocks.
- Pipeline composition (`|>` eval): concatenate stage vectors into a new `ProcV` — no threads until a complete pipeline runs as a statement.
- `run_pipeline` joins all threads, collects `DeathReport`s (none yet in this task), returns.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn sequential_pipeline_end_to_end() {
    let src = r#"
spawn nums() Vein<Int> {
    for i in 0..5 { emit i }
}
spawn double(input Vein<Int>) Vein<Int> {
    input -> item { emit item * 2 }
}
spawn total(input Vein<Int>) Vein<Int> {
    var sum Int = 0
    input -> item { sum = sum + item; emit sum }
}
spawn show(input Vein<Int>) {
    input -> item { print(item) }
}
nums |> double |> total |> show
"#;
    let out = run_capture(src);          // helper: run program, capture print sink
    assert_eq!(out, "0\n2\n6\n12\n20\n");
}

#[test]
fn partial_pipeline_is_a_value() {
    let src = "spawn a() Vein<Int> { emit 1 }\nspawn b(input Vein<Int>) Vein<Int> { input -> i { emit i + 1 } }\nspawn c(input Vein<Int>) { input -> i { print(i) } }\nlet mid spawn(Vein<Int>) Vein<Int> = b |> b\na |> mid |> c";
    assert_eq!(run_capture(src), "3\n");
}
```

- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.** Ordering guarantee for sequential stages must hold (rendezvous + single worker ⇒ deterministic output — the first test asserts exact order).
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "runtime: sequential pipelines over rendezvous channels"`

---

### Task 15: Runtime — death, drain-and-report

**Files:**
- Modify: `src/runtime.rs`, `src/interp.rs`

**Interfaces:**
- `Stmt::Cide` eval → `Signal::Death { msg, line }`; machine faults already produce the same signal (Task 11).
- Stage thread behavior on `Signal::Death`: record `DeathReport { stage, msg, line }`, drop the sender (stream auto-closes), then **drain** remaining input (`for _ in rx {}`) so upstream finishes (§6.4 drain-and-report).
- Root process: top-level statements run under the same signal discipline; a root `Death` stops remaining top-level statements.
- `run_program` (new top entry in `interp.rs`): returns `ProgramResult { deaths: Vec<DeathReport>, root_death: Option<DeathReport> }`. Report lines are formatted exactly: `process 'NAME' died: MSG (line N)`, root: `root process died: MSG (line N)`.

- [ ] **Step 1: Write the failing tests** — full code; cases:
  - mid-stage `cide`: upstream source of 5 items, stage 2 cides on the 3rd; downstream receives the first 2, pipeline completes, one `DeathReport` with the stage name and message; nothing deadlocks (test has a 5s watchdog via `std::thread` + channel timeout)
  - machine fault in a stage (`emit 1 / (item - item)`) → death report with `"division by zero"`
  - fault in a `fn` called from a stage kills that stage (not the program)
  - root death: `let a Int = xs[9]` on a 2-element array at top level → `root_death` set, subsequent statement did not run (assert via print output)
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Verify pass.**
- [ ] **Step 5: Commit** — `git add -A && git commit -m "runtime: process death, drain and report"`

---

### Task 16: Runtime — parallel workers

**Files:**
- Modify: `src/runtime.rs`

**Interfaces:**
- `Workers::Fixed(n)` → n threads for the stage; `Workers::Auto` → `std::thread::available_parallelism()` threads.
- Shared input: `Arc<Mutex<Receiver<Value>>>` — each worker loops `{ lock, recv, unlock, process }`. All workers share one `SyncSender` clone each; output closes when the last worker's sender drops (join via the natural drop, no explicit close).
- Worker death: first death records the report and sets an `Arc<AtomicBool>` dead flag for the stage; all workers check it per item and switch to discard mode (consume without processing) so the stage dies as a unit and upstream drains.
  `// ponytail: first worker death kills the whole stage; per-worker survival is a supervision-era feature`
- Each worker gets its own `Env` seeded from `stage.captured` (checker already banned cross-item mutable state in parallel stages).

- [ ] **Step 1: Write the failing tests** — full code; cases:
  - `spawn<4>` doubling stage over 100 source items → collect via a sequential accumulator sink into a sorted list; assert the multiset (order-insensitive: parallel is unordered, §4.5)
  - `spawn<auto>` parses, runs, same multiset assertion
  - death in one worker (cide when item == 50) → exactly one death report; all other items ≤ the death point may or may not appear (assert only: pipeline terminates, report exists, no deadlock under watchdog)
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Verify pass** — run the whole suite; also `cargo test -- --test-threads=1` once to shake out timing assumptions.
- [ ] **Step 5: Commit** — `git add -A && git commit -m "runtime: parallel workers, unordered emit"`

---

### Task 17: CLI + golden end-to-end tests

**Files:**
- Modify: `src/main.rs`; Create: `examples/hello.ich`, `examples/pipeline.ich`, `examples/death.ich`, `tests/e2e.rs`

**Interfaces:**
- CLI: `ichor run <file.ich>`. Exit codes: `0` clean, `1` any process death (reports printed to stderr in the exact Task 15 format), `2` usage/lex/parse/check errors (printed to stderr as `line N: MSG`).

- [ ] **Step 1: Write the example programs and failing e2e test**

`examples/pipeline.ich`:
```
genesis Vessel {
    name Str
    capacity Int
}

fn describe(v Vessel) Str {
    return v.name + " holds " + "?"
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
        print(describe(v))
    }
}

vessels |> fill |> show
```

`examples/death.ich` — a 3-stage pipeline whose middle stage `cide`s on a specific item; expected: partial output on stdout, one `process '...' died: ...` line on stderr, exit code 1.

`tests/e2e.rs`:
```rust
use std::process::Command;

fn run(example: &str) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_ichor"))
        .args(["run", &format!("examples/{example}")])
        .output()
        .unwrap();
    (String::from_utf8(out.stdout).unwrap(),
     String::from_utf8(out.stderr).unwrap(),
     out.status.code().unwrap())
}

#[test]
fn pipeline_example() {
    let (stdout, stderr, code) = run("pipeline.ich");
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "jug holds ?\n");
}

#[test]
fn death_example() {
    let (_, stderr, code) = run("death.ich");
    assert_eq!(code, 1);
    assert!(stderr.contains("died"), "stderr: {stderr}");
}

#[test]
fn compile_error_exits_2() {
    // write a temp file with `let a Int = "no"` and assert exit 2 + line number on stderr
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test --test e2e`.
- [ ] **Step 3: Implement `main.rs`** — arg parsing (only `run <path>`), read file, lex/parse/check/run via the library, map outcomes to exit codes.
- [ ] **Step 4: Verify pass** — full `cargo test`; then run `target/debug/ichor run examples/pipeline.ich` by hand and confirm the real invocation works (test-through-the-real-path rule).
- [ ] **Step 5: Commit** — `git add -A && git commit -m "cli: ichor run, golden examples, e2e gate"`

---

## Post-plan notes

- **Not in v1** (spec-deferred, do not build): supervision/restarts, `?` propagation sugar, user generics, fan-out/fan-in helpers, buffered channels, arena-backed cells (v1 approximates with Arc+COW per §7.3 note), `=>` lambdas, inclusive ranges, `ossify` semantics.
- **Known ceilings to leave comments on:** Arc cycles leak until program exit (§7.3); `Mutex<Receiver>` per-item lock in parallel stages (upgrade path: sharded queues); first-worker-death kills the stage.
- Spec drift found during implementation goes back into `language-spec.md` + decision log in the same commit as the code that surfaced it.
