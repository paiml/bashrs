# Rust-to-Shell Transpiler

Rash can transpile a subset of Rust into safe, deterministic POSIX shell scripts. Write real Rust code, test it with standard Rust tooling (`cargo test`, `cargo clippy`), then transpile to a shell script that runs anywhere.

## Why Transpile from Rust?

- **Type safety at write time**: Catch errors before generating shell
- **Standard tooling**: Use `cargo test` to verify logic
- **Safe output**: Generated scripts use `set -euf`, proper quoting, and pass `shellcheck`
- **Zero runtime**: Output is plain POSIX `sh` with no dependencies

## Quick Start

Write a Rust file using the supported subset:

```rust,ignore
// install.rs
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    let user = env_var_or("USER", "world");
    greet(&user);
}
```

Transpile it:

```bash
bashrs build install.rs -o install.sh
```

The output is a self-contained POSIX shell script:

```sh
#!/bin/sh
set -euf
IFS='
'
export LC_ALL=C

greet() {
    name="$1"
    printf '%s\n' "Hello, $name!"
}

main() {
    user="${USER:-world}"
    greet "$user"
}

trap 'rm -rf "${TMPDIR:-/tmp}/rash.$$"' EXIT
main "$@"
```

## Supported Rust Constructs

| Construct | Rust | Shell Output |
|-----------|------|--------------|
| Functions | `fn add(a: u32, b: u32) -> u32` | `add() { a="$1"; b="$2"; ... }` |
| Variables | `let x = 42;` | `x='42'` |
| Arithmetic | `x + y * 2` | `$((x + y * 2))` |
| If/else | `if x > 0 { ... } else { ... }` | `if [ "$x" -gt 0 ]; then ... fi` |
| While loops | `while i < n { ... }` | `while [ "$i" -lt "$n" ]; do ... done` |
| For loops | `for i in 0..10 { ... }` | `for i in $(seq 0 9); do ... done` |
| Match | `match x { 0 => ..., _ => ... }` | `case "$x" in 0) ... ;; *) ... ;; esac` |
| Return | `return x + 1;` | `echo $((x + 1)); return` |
| Recursion | `fn fib(n) { fib(n-1) + fib(n-2) }` | Recursive shell function with `$(...)` |
| Nested calls | `f(g(h(x)))` | `"$(f "$(g "$(h x)")")"` |
| println! | `println!("{}", x)` | `printf '%s\n' "$x"` |

## Supported Types

- `u32`, `u16` -- integers (shell arithmetic)
- `bool` -- booleans (`true`/`false` strings)
- `&str`, `String` -- strings (shell strings)
- `()` (void) -- functions with no return value

## Match Expressions

Match can be used as a statement or in a let binding:

```rust,ignore
// Match as let binding -- generates case with per-arm assignment
let tier = match level % 3 {
    0 => level * 10,
    1 => level + 5,
    _ => level,
};
```

Generates:

```sh
case "$level" in
    0) tier=$((level * 10)) ;;
    1) tier=$((level + 5)) ;;
    *) tier="$level" ;;
esac
```

## Functions and Return Values

Functions with return types use `echo` + `return` for output capture:

```rust
fn double(x: u32) -> u32 {
    return x * 2;
}

fn main() {
    let result = double(21);  // Captured via $(double 21)
    println!("{}", result);   // Prints: 42
}
```

Nested function calls are supported:

```rust,ignore
let result = double(add_ten(square(3)));
// Shell: result="$(double "$(add_ten "$(square 3)")")"
```

## If-Else as Expressions

If-else can be used in let bindings and return statements, including nested else-if chains:

```rust
fn classify(n: i32) -> &'static str {
    if n > 0 {
        "positive"
    } else if n < 0 {
        "negative"
    } else {
        "zero"
    }
}
```

Generates:

```sh
classify() {
    n="$1"
    if [ "$n" -gt 0 ]; then
        echo positive
    elif [ "$n" -lt 0 ]; then
        echo negative
    else
        echo zero
    fi
}
```

## Makefile Transpilation

Rust code using `println!()` and `exec()` can transpile to Makefile output. The emitter detects raw output mode automatically and emits resolved lines directly:

```rust
fn main() {
    let project = "myapp";
    println!("{}: build test", project);
}
```

Transpiles to:

```makefile
myapp: build test
```

## Limitations

The transpiler supports a **restricted subset** of Rust designed for shell-compatible operations:

- No heap allocation (`Vec`, `HashMap`, `Box`)
- No traits, generics, or lifetimes
- No closures (lambda expressions are simplified)
- No async/await
- No pattern destructuring beyond match literals and wildcards
- Integer arithmetic only (no floating point)
- Arrays are simulated via indexed variables (`arr_0`, `arr_1`, ...)

## Running the Demo

```bash
cargo run --example transpiler_demo
```

This runs 7 demonstrations covering basic functions, nested calls, match expressions, loops with return, match inside loops, recursion, and multi-function programs.

## What a method call lowers to

bashrs transpiles a method call only when POSIX shell has an honest spelling for it. Where it does not, the transpile **fails and names the method**: an unlowerable call is never emitted as a no-op, because a script that runs and does nothing is worse than one that does not build.

| Rust | Shell | Note |
|---|---|---|
| `s.len()` | `${#s}` | computed at runtime; a variable known to be an array takes the element-count path instead |
| `s.to_string()` | the value | identity on a string |
| `x.unwrap_or(d)` | `${x-d}` | **unset only**, not `${x:-d}` |
| `x.unwrap_or_else(\|\| d)` | `${x-d}` | only when the closure body has no side effects |
| `std::env::var("X")` | `"${X}"` | the same read as `env("X")`; under the script's `set -u` an unset name aborts |
| `std::env::var("X").unwrap_or(d)`, `.unwrap_or_else(\|_\| d)` | `"${X-d}"` | on the env name itself, so the default really applies when `X` is unset (v7.4.0) |
| `std::env::var("X").unwrap_or_default()` | `"${X-}"` | unset reads as empty |
| `std::env::var("X").unwrap()`, `.expect("m")` | `"${X?}"`, `"${X?m}"` | the script aborts when `X` is unset, as the panic would |
| `s.push_str(t)`, `s.push(c)`, `v.insert(..)`, `s.rev()` | none | the transpile fails naming the method |

The default `d` is spelled for the double-quoted expansion it sits in: a plain word goes in bare (`"${PORT-8080}"`); anything with a brace, a space, a quote, a dollar or a backslash becomes a nested double-quoted word with backslash escapes (`"${X-"{brace}"}"`), which bash, dash and busybox all read the same way. v7.3.0 rendered such defaults through the single-quoting escaper, which printed the quotes literally under bash and was a syntax error under dash; v7.4.0 fixed it and pinned it (`F-TCORE-028`).

The `unwrap_or` spelling is the one worth understanding. Rust returns the default for `None` alone, so a value that is present but empty must survive:

```sh
name=          # set, empty
echo "${name-fallback}"    # prints nothing: the value is present
echo "${name:-fallback}"   # prints fallback: wrong for unwrap_or
```

That distinction is pinned by a test and by a falsification entry in `contracts/transpiler-core-v1.yaml`, so a future change to the spelling fails the contract gate rather than silently changing what programs mean.

