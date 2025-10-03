# Rash Error Guide

This guide helps you understand and troubleshoot common Rash errors.

## Table of Contents

- [Error Types](#error-types)
- [Common Errors](#common-errors)
- [Validation Errors](#validation-errors)
- [Parse Errors](#parse-errors)
- [Stdlib Errors](#stdlib-errors)
- [Debugging Tips](#debugging-tips)
- [Getting Help](#getting-help)

---

## Error Types

Rash errors are categorized into several types:

### 1. Parse Errors
Errors during Rust syntax parsing.

**Error Format:**
```
Error: Parse error: <syn error message>
```

**Cause**: Invalid Rust syntax in your source file.

### 2. Validation Errors
Errors during AST validation (unsupported features).

**Error Format:**
```
Error: AST validation error: <validation message>
Error: Validation error: <message>
```

**Cause**: Using unsupported Rust features or invalid constructs.

### 3. IR Generation Errors
Errors during intermediate representation generation.

**Error Format:**
```
Error: IR generation error: <message>
```

**Cause**: Issues converting AST to shell IR.

### 4. Emission Errors
Errors during shell code generation.

**Error Format:**
```
Error: Code emission error: <message>
```

**Cause**: Problems generating final shell script.

### 5. Verification Errors
Errors during ShellCheck verification.

**Error Format:**
```
Error: Verification error: <message>
Error: ShellCheck validation error: <details>
```

**Cause**: Generated shell script fails ShellCheck validation.

---

## Common Errors

### Unsupported Feature

**Error:**
```
Error: Unsupported feature: <feature>
```

**Common Examples:**
```
Error: Unsupported feature: mutable variables
Error: Unsupported feature: for loops (not yet implemented)
Error: Unsupported feature: match expressions (not yet implemented)
```

**Solution:**
- Check [ROADMAP.md](../ROADMAP.md) for feature status
- Use workarounds with supported features
- File a feature request on GitHub

**Example:**
```rust
// ❌ Not supported: mutable variables
let mut count = 0;
count += 1;

// ✅ Supported: immutable rebinding
let count = 0;
let count = count + 1;
```

---

### Invalid println! Arguments

**Error:**
```
Error: Validation error: Invalid println! arguments
```

**Cause**: Incorrect usage of `println!` macro.

**Solution:**
```rust
// ❌ Wrong: Complex format strings not yet supported
println!("Value: {:?}", some_struct);

// ✅ Correct: Simple interpolation
println!("Value: {}", value);

// ✅ Correct: Multiple arguments
println!("x: {}, y: {}", x, y);
```

---

### Undefined Function

**Error:**
```
Error: AST validation error: Undefined function: <name>
```

**Cause**: Calling a function that doesn't exist or isn't a stdlib function.

**Solution:**
```rust
// ❌ Wrong: typo in function name
let result = string_uppre("hello");

// ✅ Correct: proper function name
let result = string_to_upper("hello");

// Check stdlib functions:
// - string_trim, string_contains, string_len
// - string_replace, string_to_upper, string_to_lower
// - fs_exists, fs_read_file, fs_write_file
// - fs_copy, fs_remove, fs_is_file, fs_is_dir
```

---

### Reserved Identifier

**Error:**
```
Error: AST validation error: Reserved identifier: <name>
```

**Cause**: Using a shell builtin or reserved word as a function name.

**Solution:**
```rust
// ❌ Wrong: 'test' is a shell builtin
fn test() {
    echo("testing");
}

// ✅ Correct: use a different name
fn run_test() {
    echo("testing");
}
```

**Reserved identifiers include**: `test`, `export`, `source`, `true`, `false`, `cd`, `pwd`, etc.

---

## Validation Errors

### Type Mismatch

**Error:**
```
Error: Validation error: Type mismatch in <context>
```

**Solution:**
```rust
// ❌ Wrong: passing wrong types
if "string" {  // strings are not booleans
    echo("hello");
}

// ✅ Correct: use boolean expressions
if fs_is_file("/etc/passwd") {
    echo("file exists");
}

// ✅ Correct: use comparisons
if count > 0 {
    echo("positive");
}
```

---

### Recursive Function Calls

**Error:**
```
Error: Validation error: Recursive function calls not supported
```

**Cause**: Rash doesn't support recursion (shell limitation).

**Solution:**
```rust
// ❌ Wrong: recursive factorial
fn factorial(n: i32) -> i32 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)  // recursion not supported
    }
}

// ✅ Correct: use loops instead
fn factorial(n: i32) -> i32 {
    let result = 1;
    let i = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    result
}
```

---

## Parse Errors

### Syntax Errors

**Error:**
```
Error: Parse error: expected `;`, found `}`
```

**Cause**: Missing semicolons or braces.

**Solution:**
```rust
// ❌ Wrong: missing semicolon
fn main() {
    let x = 42
    echo(x);  // syntax error here
}

// ✅ Correct: add semicolon
fn main() {
    let x = 42;
    echo(x);
}
```

---

### Invalid Literal

**Error:**
```
Error: Parse error: invalid literal
```

**Solution:**
```rust
// ❌ Wrong: invalid number format
let x = 1_000_000_000_000;  // might overflow i32

// ✅ Correct: use valid i32 range
let x = 1000000;

// ❌ Wrong: float literals not supported
let x = 3.14;

// ✅ Correct: use integers only
let x = 3;
```

---

## Stdlib Errors

### Function Not Found

**Error:**
```
Error: Validation error: Unknown stdlib function: <name>
```

**Solution:** Check the function exists and spelling is correct.

**Available stdlib functions (v0.9.3):**

**String operations:**
- `string_trim(s)` - Remove leading/trailing whitespace
- `string_contains(haystack, needle)` - Check substring
- `string_len(s)` - Get string length
- `string_replace(s, old, new)` - Replace substring
- `string_to_upper(s)` - Convert to uppercase
- `string_to_lower(s)` - Convert to lowercase

**File system operations:**
- `fs_exists(path)` - Check if path exists
- `fs_read_file(path)` - Read file contents
- `fs_write_file(path, content)` - Write to file
- `fs_copy(src, dst)` - Copy file
- `fs_remove(path)` - Remove file
- `fs_is_file(path)` - Check if regular file
- `fs_is_dir(path)` - Check if directory

**Example:**
```rust
// ❌ Wrong: function doesn't exist
let upper = to_uppercase("hello");

// ✅ Correct: use proper function name
let upper = string_to_upper("hello");
```

---

### Wrong Number of Arguments

**Error:**
```
Error: Validation error: Wrong number of arguments
```

**Solution:**
```rust
// ❌ Wrong: missing argument
let result = string_replace("hello", "world");  // needs 3 args

// ✅ Correct: provide all arguments
let result = string_replace("hello", "world", "rust");
// Replaces "world" with "rust" in "hello"
```

---

## Debugging Tips

### 1. Check Generated Shell Script

Use `--output` to save the generated script and inspect it:

```bash
bashrs build input.rs --output output.sh
cat output.sh  # inspect generated code
```

### 2. Enable Verbose Mode

```bash
bashrs build input.rs --verbose
```

### 3. Validate Incrementally

Build your script step by step:

```rust
// Start simple
fn main() {
    echo("hello");
}

// Add complexity gradually
fn main() {
    let name = "world";
    echo("hello {name}");
}

// Add stdlib functions one at a time
fn main() {
    let name = "world";
    let upper = string_to_upper(name);
    echo("hello {upper}");
}
```

### 4. Use ShellCheck

Run ShellCheck on generated scripts:

```bash
bashrs build input.rs -o output.sh
shellcheck -s sh output.sh
```

### 5. Test in Different Shells

Test generated scripts in various shells:

```bash
# Test in sh
sh output.sh

# Test in dash
dash output.sh

# Test in bash
bash output.sh
```

---

## Common Gotchas

### 1. String Interpolation Syntax

```rust
// ❌ Wrong: Rust format syntax not supported
let msg = format!("Hello, {}!", name);

// ✅ Correct: Use string interpolation
let msg = "Hello, {name}!";
echo(msg);
```

### 2. Function Return Values

```rust
// Functions return via echo
fn add(a: i32, b: i32) -> i32 {
    a + b  // This is echoed to stdout
}

fn main() {
    let sum = add(1, 2);  // Captured via $()
    echo("Sum: {sum}");
}
```

### 3. Variable Scope

```rust
// Variables have function scope
fn main() {
    let x = 42;
    if true {
        let y = x + 1;  // y is local to function, not block
    }
    echo(y);  // ❌ This works (shell scoping) but avoid
}
```

### 4. Boolean vs. Exit Codes

```rust
// Function success = exit code 0 (true)
fn check_file() -> bool {
    fs_is_file("/etc/passwd")  // Returns true/false
}

// Can use in conditions
if check_file() {
    echo("File exists");
}
```

---

## Performance Tips

### 1. Minimize File Operations

```rust
// ❌ Slow: reading file multiple times
if fs_exists(path) {
    let content = fs_read_file(path);
    // ...
}

// ✅ Better: read once and check
let content = fs_read_file(path);
if content != "" {
    // use content
}
```

### 2. Avoid Nested String Operations

```rust
// ❌ Slower: nested operations
let result = string_to_upper(string_trim(text));

// ✅ Better: separate steps (more readable, easier to debug)
let trimmed = string_trim(text);
let upper = string_to_upper(trimmed);
```

---

## Getting Help

### Documentation

- **API Docs**: https://docs.rs/bashrs
- **User Guide**: [docs/user-guide.md](user-guide.md)
- **STDLIB Spec**: [docs/specifications/STDLIB.md](specifications/STDLIB.md)
- **Examples**: [examples/](../examples/)

### Community

- **GitHub Issues**: https://github.com/paiml/bashrs/issues
- **Feature Requests**: https://github.com/paiml/bashrs/issues/new?labels=enhancement
- **Bug Reports**: https://github.com/paiml/bashrs/issues/new?labels=bug

### Troubleshooting Steps

1. **Check error message** - Read the full error including "Caused by" chains
2. **Verify syntax** - Ensure valid Rust syntax
3. **Check feature support** - Verify feature is supported (see ROADMAP.md)
4. **Review examples** - Look at working examples in `examples/`
5. **Inspect generated code** - Use `--output` to see shell script
6. **Search issues** - Check if others have reported similar errors
7. **Create minimal reproduction** - Reduce to smallest failing example
8. **File issue** - Report bug with reproduction case

---

## Error Reporting

When reporting errors, please include:

1. **Rash version**: `bashrs --version`
2. **Input code**: Minimal reproduction case
3. **Error message**: Full error output
4. **Expected behavior**: What you expected to happen
5. **Environment**: OS, shell version, Rust version

**Example bug report:**

```
## Environment
- Rash version: 0.9.3
- OS: Ubuntu 22.04
- Shell: dash 0.5.11
- Rust: 1.75.0

## Input Code
```rust
fn main() {
    let x = string_to_upper("hello");
    echo(x);
}
```

## Error Output
```
Error: Validation error: Unknown function: string_to_upper
```

## Expected
Should compile and output "HELLO"

## Notes
Function is listed in STDLIB.md as available in v0.9.3
```

---

**Last Updated**: 2025-10-03
**Version**: 0.9.3
**Status**: Complete
