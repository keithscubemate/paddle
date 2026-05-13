# Plan: Rich Error Context (line/col + call chain)

## Context

Paddle errors currently show bare variant debug output (`ERROR: SymbolUndefined("foo")`) with no location info and no call context. The goal is to surface:
1. **Source location** — which line:col the failing expression is at
2. **Call context** — what form/call site triggered the error (lambda, define, function call)

The existing infrastructure is already good: `Span` (line + col) lives in lexer tokens and propagates through `Expr` nodes. It's just dropped at `lower()` and never reattached. `anyhow` is already a dep, so `.with_context()` is free to use.

---

## Phase 1 — `anyhow` context chains (no structural changes to Value)

This gives the most value with the least churn.

### Step 1a: Add `Expr::span()` helper (`core/src/parser.rs`)

```rust
impl<'a> Expr<'a> {
    pub fn span(&self) -> Span {
        match self { Expr::Atom(_, s) | Expr::List(_, s) => *s }
    }
}
```

### Step 1b: Top-level location in `process()` (`core/src/cursor.rs`)

In the eval loop, annotate failures with the top-level expression span:

```rust
let span = ast.span();
let val = eval(&expr, env)
    .with_context(|| format!("at {span}"))?;
```

This is the cheapest way to get a source location on every error.

### Step 1c: Call-site context in `apply()` (`core/src/eval/eval.rs`)

After `let head = eval(head, env)?;`, wrap the rest of apply in a closure and attach context based on what `head` resolved to:

```rust
let context = match &head {
    Value::Func { name, .. } | Value::Macro { name, .. } => format!("in call to `{name}`"),
    Value::Lambda { .. }                                  => "in anonymous lambda call".to_string(),
    Value::Builtin(_, name)                               => format!("in builtin `{name}`"),
    _                                                     => return Ok(head.clone()),
};
apply_resolved(&head, tail, env).with_context(|| context)
```

Extract the body of apply (after head is resolved) into `apply_resolved()` to make wrapping clean.

### Step 1d: Form context in `eval_form()` (`core/src/eval/eval.rs`)

Wrap the inner match arms that can fail with context:

- `Form::Define` / `Form::DefineMacro` → `"in define"`
- `Form::Lambda` → `"in lambda"`
- `Form::If` → `"in if"`

These are lower-value so keep them terse — one `.with_context()` call per arm, wrapping the `define(...)` / `eval(...)` call.

### Step 1e: Fix error display (`core/src/cursor.rs::display_results`)

```rust
Err(err) => {
    eprintln!("error: {err}");
    for cause in err.chain().skip(1) {
        eprintln!("  caused by: {cause}");
    }
}
```

anyhow's `{:#}` also works but the manual chain gives more control over formatting.

**Result after Phase 1:**
```
error: undefined symbol: foo
  caused by: in call to `bar`
  caused by: at 5:1
```

---

## Phase 2 — Span propagation into `Value::Symbol`

This adds the exact location of the failing symbol reference (not just the top-level call site).

### Step 2a: Extend `Value::Symbol` (`core/src/eval/value.rs`)

```rust
// before
Symbol(String),
// after
Symbol(String, Option<Span>),
```

Import `Span` from `crate::lexer`. Update all match arms that destructure `Symbol` — add `_` or bind the span as needed. Key sites:
- `Display` impl — render `s` only, ignore span
- `Debug` impl — show `s` only (span is noise in debug)
- `truthy()` — add `Symbol(_, _)` arm  
- `eval.rs` — `resolve(atom, env)` → `resolve(atom, span, env)`
- `define()` — `Value::Symbol(atom)` → `Value::Symbol(atom, _)`
- `eval_form` Lambda arm — symbol extraction in args list

### Step 2b: Propagate span through `lower()` (`core/src/eval/lower.rs`)

```rust
// classify gains a span parameter
fn classify(atom: &str, span: Option<Span>) -> Value { ... Value::Symbol(atom.to_owned(), span) }

// quote_eval passes the span from the Expr
Expr::Atom(atom, span) => classify(atom, Some(*span)),
```

Symbols constructed at runtime (quasiquote expansion, etc.) pass `None`.

### Step 2c: Use span in `resolve()` (`core/src/eval/eval.rs`)

```rust
Value::Symbol(atom, span) => resolve(atom, *span, env),

fn resolve(atom: &str, span: Option<Span>, env: ...) -> Result<Value> {
    env.borrow().resolve(atom).ok_or_else(|| {
        let err = anyhow::Error::from(EvalError::SymbolUndefined(atom.to_string()));
        match span {
            Some(s) => err.context(format!("at {s}")),
            None    => err,
        }
    })
}
```

**Result after Phase 2:**
```
error: undefined symbol: foo
  caused by: at 3:17          ← exact symbol reference site
  caused by: in call to `bar`
  caused by: at 5:1           ← top-level call site
```

### Test updates

- `lower.rs` tests: `sym("foo")` helper needs updating → `Value::Symbol("foo".into(), None)` (tests construct via `lower_str` which goes through the real lexer, so spans will be `Some` — update test helper to accept any span or restructure)
- `eval_errors.rs` / `builtin_errors.rs`: downcast tests don't care about Symbol internals, should be unaffected
- Integration tests: check `SymbolUndefined` by downcasting — unaffected since error type is unchanged

---

## Files to modify

| File | Changes |
|---|---|
| `core/src/parser.rs` | Add `Expr::span()` |
| `core/src/cursor.rs` | Top-level context, fix display |
| `core/src/eval/eval.rs` | apply context, eval_form context, resolve span |
| `core/src/eval/value.rs` | `Symbol(String, Option<Span>)`, update impls |
| `core/src/eval/lower.rs` | Pass span through classify |
| `core/src/eval/env.rs` | Update Symbol matches in builtins |

---

## Verification

```
cargo test -p paddle-core          # all existing tests still pass
cargo build -p paddle-cli
echo '(define (foo x) (bar x)) (foo 5)' | ./target/debug/paddle
# expect: error with "in call to foo" and location
```
