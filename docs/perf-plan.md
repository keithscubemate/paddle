# Perf plan

Learning project: hand-write the changes, use this as a map.
Stay a tree-walker. Bytecode VM stays a "some day" item.

## Habits

- Predict, measure, record. Write the predicted top-3 costs before profiling.
- One change per measurement.
- Keep a log: change, prediction, result.
- Integration tests are the guard rail. Run after every step.
- Do work once: push decisions earlier (lexer, parser, lower, closure creation).
- Cost ladder: allocator call > cache miss > string hash > bad branch > `Rc` inc/dec > arithmetic.

## Step 0: measure

- Bench set: `fib 25`/`fib 30`, 1M-iteration counting loop, `nqueens`, `sudoku`.
- `hyperfine`, `--release`.
- Cheap experiments: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `mimalloc`.
- Profilers: `valgrind --tool=callgrind`, `dhat`, or `samply`/`cargo flamegraph` (needs `perf`, awkward on WSL2).
- Clone counter: static counter bumped in `Value::clone`; report clones per eval step.
- Reference points: native Rust loop (floor), CPython, a small Scheme (chibi/tinyscheme).
- Express results as ratios (x slower than native, x slower than CPython).

## Tasks

- 1. Eager work on the happy path
    + `.context(format!(...))` in `eval_step` builds a `String` (and `Display`s a form) on every call
    + Where else is a `String`/`Vec`/list built but only needed on error?
- 2. Ownership audit of `eval`
    + `eval(Value, Rc<RefCell<Env>>)` by value, call sites `eval(v.clone(), env.clone())`
    + `Done(ast.clone())` on an already-owned `ast`
    + Which clones can go with `&Value`/`&Rc<..>`? Which can't (trampoline `Continue`)?
- 3. Per-call environment
    + Count allocs per call: `Rc<RefCell<Env>>`, `HashMap`, `String` per `define`, cloned body
    + `ends_with("...")` scan of arg names per call: decide at definition time
    + Minimal structure for "a frame of N values"?
- 4. Lexical addressing (biggest learning item)
    + `resolve` hashes a string and walks parents; make it (depth, slot)
    + Open questions: internal `define`, macros (see unexpanded code), `set!`, REPL top level
    + Read: SICP lexical addressing, Crafting Interpreters "Resolving and Binding"
- 5. Specialise hot primitives
    + `(+ a b)` today: eval head, collect args into a cons list, builtin walks it
    + Emit a dedicated node in lower for known builtins
    + What if the user redefines `+`? How to detect?
- 6. Lists vs `Value::Vec` (last)
    + Only after 1-5; re-profile and decide from data

## Knowing when to stop

- Compare to reference ratios, not a guess. No claimed ceiling.
- Spiky profile: easy wins left. Flat profile: structural change needed.
- Gain per change: three in a row under ~5% means diminishing returns.
- Tie to a goal: slowest thing actually wanted (paddle.pd, an AoC day) and the x needed.
- Learning: stop when the next step stops teaching something new.

## Unrelated bug to hunt

- `Value::set_span` (`core/src/value.rs`): `Func` arm vs `Macro` arm. When does `eval_step` call it on a function body?
