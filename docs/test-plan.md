# Paddle test-improvement plan

Work top to bottom. Each phase should be one commit/PR. Run `cargo test --workspace` after every phase.

## Phase 0 — Confirm suspected bugs (write a failing test per item, then decide fix vs. pin)
- [ ] Lone `"` atom panics: `lower.rs:44` (unwrap on strip_prefix/suffix).
- [ ] `(quote)` and `(quasiquote)` panic via `unreachable!`: `eval/eval.rs:126,132`.
- [ ] `substring` panics on multibyte UTF-8 slice: `env/builtins/string.rs:74`.
- [ ] `Value::set_span` maps `Func` to `Macro`: `value.rs:73`.
- [ ] `dump()` prints literal `{{...}}` for Builtin: `value.rs:154`.
- [ ] `current_file` not restored when `require` errors: `eval/eval.rs:206-212`.
- [ ] String token has stale start column; newline inside a string doesn't bump line: `lexer.rs:44,51-58,75-79`.
- [ ] `(1 2 3)` silently returns `1` instead of erroring: `eval/eval.rs:66`.
- [ ] `inf`/`nan`/`infinity` lex and lower as numbers: `lower.rs:31`.
- [ ] `(+ 1 nil 2)` silently returns 1 (nil truncates args) in `math.rs`.

For each: assert the typed error (or the intended value) rather than `#[should_panic]`. Decide whether current behavior is a bug (fix code) or intended (pin it with a test and a comment).

## Phase 1 — Fix test infrastructure
- [ ] `core/src/lib.rs:99-121`: make `eval_str_env` fail on an error in any expression, not only the last. Make `eval_str` reject trailing input.
- [ ] Make the helpers return `Result` (or add `try_` variants) so parse errors can be asserted without panicking.
- [ ] Add a helper that keeps spans, and assert real spans in error tests. The `strip_*_span` helpers stay for value comparisons only.
- [ ] Replace the vacuous `else { assert_eq!(1, 1) }` in `parser.rs` `list_span_is_first_element`.
- [ ] `system/env.rs`: add real assertions to `if_only_evaluates_{true,false}_branch` (check the variable value).
- [ ] `integrations/errors.rs:38`: assert the error kind and that earlier expressions ran.
- [ ] `system/splice.rs:135`: replace `should_panic` with a typed-error assertion (this needs a code change in `Value::splice` callers).
- [ ] `system/set_bang.rs`: assert error variants instead of message substrings (needs typed variants for "Bad set! head" and "can't set! undefined").
- [ ] Make `print` testable (inject a writer, or capture output) so `print` and `wc.pd`-style programs can be asserted.

## Phase 2 — Spans end to end
- [ ] Run `examples/span_errors.pd` and `span_errors_nested.pd` through `process`, and assert the file/line/column on the resulting error.
- [ ] Span of `()`, list span (first element, not `(`), and multi-line expressions.
- [ ] Span of string tokens (after fixing the stale column).
- [ ] Error context chain text: "error evaling head of list", "error calling [:sym] at span".

## Phase 3 — Lexer / parser / lowering units
Lexer:
- [ ] `;` comments: to EOF without newline, mid-line, `;` inside a string.
- [ ] Unterminated string (and the lone-`"` lowering case).
- [ ] `` ` ``, `,`, `@` tokens; `,@x`; mixed `` `(a ,b @c) ``.
- [ ] Splitting: `a@b`, `a'b`, `abc"d e"`, `"a"b`.
- [ ] Unicode whitespace and column counting; multi-line string.

Parser:
- [ ] `unquote` / `unquotesplicing` desugaring.
- [ ] Prefix followed by `)`; `(' x)`; `(` alone (`EmptyInput` vs `UnexpectedEof` — decide intended).
- [ ] Spans for EOF errors.

Lowering:
- [ ] Number forms: `+5`, `.5`, `5.`, `1e3`, `-0`, `0x10`, `1_000`, `inf`, `nan`.
- [ ] `#f`, `#t`, `nil`.
- [ ] String escape handling (decide: decode or keep raw, then pin it).

## Phase 4 — `value.rs` and `env` unit tests (new `#[cfg(test)]` modules)
- [ ] `truthy`: `0`, `0.0`, NaN, `Char(0)`, `""`, `nil`, `#f`, symbol, cons, callables.
- [ ] `Display`: symbol (`:name`), strings, char escapes, bool, nil, NoPrint, nested lists, improper pair (decide: add dotted printing), Func/Macro/Builtin.
- [ ] `dump()` for all variants.
- [ ] `splice`: Nil self, non-list self, improper tail.
- [ ] `set_span` for every variant.
- [ ] `ConsIter` with improper tail; `into_cons_list`; `to_cons_list` of empty.
- [ ] `Form::try_parse` for every alias (`lamda`, `.\`, `defm`, `__name__`, etc.).
- [ ] Equality: Builtin vs itself; closures with cyclic envs must not hang.
- [ ] `Env`: define/resolve/shadowing, `set_bang` through the parent chain, builtin-shadowing behavior (`(define car 1)`), `new_child` shares builtins, `with_stdlib` failure.

## Phase 5 — Evaluator system tests (new files under `core/tests/system/`)
- [ ] `if.rs`: move the ~11 `if` tests out of `env.rs`; add wrong-arity (2-arg `if`).
- [ ] `quasi.rs`: nil-head cons (`` `(() ,x) ``), nested quasiquote levels, `,f` where `f` is callable, `unquotesplicing` outside quasi (wrong error variant today), splice of Nil, improper tail.
- [ ] `require.rs`: arg count 0 and 2, non-string arg, missing file, `current_file` restored after failure, `whoami`/`__name__` inside required file, a path-independent fixture (fix the `../examples/` cwd dependence).
- [ ] `progn.rs`: empty `(progn)`, tail position.
- [ ] `macros.rs`: variadic-last check at call time, bare `...`, zero extra variadic args binds Nil, macro expanding in non-tail position, macro defining macro.
- [ ] `tco.rs`: raise depth well above 2500; tail position in `progn`, `if`, and macro expansion; a test documenting non-tail recursion depth behavior.
- [ ] `eval_errors.rs`: remove or construct the dead variants (`EmptyPrognBody`, `BadLambdaArgs`, `BadLambdaArgsList`, `BadLambdaArgsListType`, `BadDefineFunctionHead`, `BadDefineFunctionHeadTypes`) and make tests match.
- [ ] Non-callable head `(1 2 3)` and `(() 1 2)`.
- [ ] Parse error mid-stream vs. eval error mid-stream: assert which earlier side effects persist (`cursor.rs` behavior).

## Phase 6 — Builtins
Math:
- [ ] `(- 5)`, `(/ 5)` semantics; divide/modulo by zero; `//` with negatives and zero; `%` with 1/2/3 args and negative operands; `<` with 0/1/many args, chained.
- [ ] `nil` argument handling for `+ * - /`.

Boolean:
- [ ] `&&`, `||` zero args; non-short-circuit behavior pinned; `not` arity.
- [ ] `=`: arity 1/3, `(= #t #t)`, `(= '(1) '(1))`, Str vs Symbol, Char vs 1-char Str, NaN.

List:
- [ ] `append`: extra args, improper second arg, non-list first; `list` with no args; `car`/`cdr` zero-arg errors (typed).

Predicates:
- [ ] Zero-arg and many-arg errors for all 8 predicates; `atom?` of nil.

String:
- [ ] `string-ref`: negative/NaN/fractional/out-of-range index.
- [ ] `substring`: `from == len`, `from > to`, empty string, multibyte (after fixing the panic), omitted `to`.
- [ ] `string->num`: `inf`, `nan`, whitespace; `list->string` with non-list; `string->symbol`.
- [ ] Byte-vs-char semantics for `string-length`, `string->list`, `char` with multibyte input (decide intended behavior).
- [ ] Fix the copy-pasted "string-length" error messages in `string-ref`/`substring`.

I/O:
- [ ] `getchar`/`getline` at EOF (note `getline` returns `""` for both EOF and blank line); needs stdin injection or CLI-level tests.

## Phase 7 — Stdlib
- [ ] Inventory every definition in `examples/base.pd` and `examples/macros.pd` and add at least one happy-path and one edge-case integration test each. Currently covered: `length`/`len`, `map`, `filter`, `range`, `^`, `cadr`/`caddr`/`cadddr`, `<= >= >`, `let`/`let*`/`letrec`/`letn`.
- [ ] Add `<` and `=` to `integrations/comparison.rs`, plus non-numeric argument cases.

## Phase 8 — Examples as smoke tests
Add to `core/tests/integrations/examples.rs` (or a new `examples_all.rs`): run each file and assert success (and a known output where stable).
- [ ] `forth.pd`, `paddle.pd` (highest priority — self-hosting goal)
- [ ] `y_combinator.pd`, `nqueens.pd`, `sudoku.pd`, `wc.pd` (needs stdin fixture), `dumb_macros.pd`
- [ ] `aoc_2025/day1.pd`, `aoc_2025/day2.pd` (need input fixtures)
- [ ] `span_errors.pd`, `span_errors_nested.pd` (expect a specific error; covered in Phase 2)
- [ ] Fix `examples.rs` copy-pasted comment and the wrong `.expect("fact.pd failed")` message.

## Phase 9 — CLI (`cli/tests/cli.rs`, `cli/src/repl.rs`)
- [ ] Exact exit codes: success, missing file, eval error in a file (likely exits 0 today — decide intended behavior), parse error.
- [ ] stderr content on error.
- [ ] Tighten `stdin_evaluates_expression` and the file tests to exact stdout (not `contains`).
- [ ] `--repl`, `file` plus `--repl`, `--version`, `--help`.
- [ ] `require` via the CLI (cwd-relative path behavior).
- [ ] Unit-test the pure helpers `handle_repl_cmd` and `make_prompt` (`:env`, `:e`, `:v`, `:version`, `:env_get <name>`).
- [ ] Unit-test `count_paren`/`is_ready_to_process`, including parens inside strings and comments (miscount today).
- [ ] `display_result`: error, NoPrint, normal value.
- [ ] Stdlib-load failure path.

## Phase 10 — Wasm (`wasm/src/lib.rs`)
- [ ] Tests for `Runner::run_code`, `Tag`, and error formatting (use `wasm-bindgen-test` or factor the logic into plain functions).

## Phase 11 — Cleanup
- [ ] Remove duplicates: `integrations/errors.rs` (three tests duplicated in `system/`), `integrations/programs.rs` `recursive_factorial_inline` vs `system/env.rs` `define_func_fact`, overlap among `cadr.rs`/`car.rs`/`cdr.rs`/`list.rs`.
- [ ] Move `caddr`/`cadddr` tests next to the `cadr` tests, or document the split (`system/` = builtins, `integrations/` = stdlib).
- [ ] Fix the stale comment in `builtin_errors.rs` (`car_arity`).

## Notes for the implementer
- Do Phase 0 first: several later phases depend on whether a suspected bug is real or intended.
- Where behavior is questionable but arguably intended (nil truncating math args, non-short-circuit `&&`/`||`, `getline` EOF, byte-based strings), add a test that pins the current behavior with a comment, so a future change is deliberate.
- Docs and README roadmap bullets should stay terse (`-` → `+` → `*`) if you reference this plan there.
