# Standard Library Specification (v0.9.0)

## Overview

The Rash standard library provides essential utilities for building production-grade shell scripts. All functions follow POSIX compliance and are transpiled to safe, verified shell code.

## Design Principles

1. **POSIX Compliance**: All stdlib functions must work in `sh`, `dash`, `ash`, `busybox sh`
2. **Safety First**: No user input can escape quoting, all edge cases handled
3. **Determinism**: Same inputs produce identical outputs
4. **Zero Dependencies**: No external tools required (except POSIX builtins)
5. **Idempotence**: Where applicable, functions are safe to call multiple times

## Architecture

### Module Structure
```
rash::stdlib
  ├── string   - String manipulation
  ├── array    - Array/list operations
  ├── fs       - File system utilities
  └── prelude  - Auto-imported essentials
```

### Runtime Emission Strategy

Stdlib functions are emitted as shell functions in the runtime section:
- Only emit functions that are actually used (dead code elimination)
- Functions are prefixed with `rash_` to avoid naming conflicts
- All functions use local variables to avoid pollution

---

## String Module (`rash::stdlib::string`)

### `trim(s: &str) -> String`
Remove leading and trailing whitespace.

**Rust Signature**:
```rust
use rash::stdlib::string::trim;

fn main() {
    let text = "  hello  ";
    let result = trim(text);
    println!("{}", result); // "hello"
}
```

**Shell Implementation**:
```sh
rash_string_trim() {
    s="$1"
    # Remove leading whitespace
    s="${s#"${s%%[![:space:]]*}"}"
    # Remove trailing whitespace
    s="${s%"${s##*[![:space:]]}"}"
    printf '%s' "$s"
}
```

**Properties**:
- Idempotent: `trim(trim(s)) == trim(s)`
- Empty safe: `trim("") == ""`
- Whitespace: Handles spaces, tabs, newlines

---

### `contains(haystack: &str, needle: &str) -> bool`
Check if string contains substring.

**Rust Signature**:
```rust
use rash::stdlib::string::contains;

fn main() {
    if contains("hello world", "world") {
        println!("Found!");
    }
}
```

**Shell Implementation**:
```sh
rash_string_contains() {
    haystack="$1"
    needle="$2"
    case "$haystack" in
        *"$needle"*) return 0 ;;
        *) return 1 ;;
    esac
}
```

**Properties**:
- Empty needle always matches: `contains(s, "") == true`
- Case sensitive
- No regex, literal match only

---

### `split(s: &str, delimiter: &str) -> Vec<String>`
Split string by delimiter.

**Rust Signature**:
```rust
use rash::stdlib::string::split;

fn main() {
    let parts = split("a,b,c", ",");
    // parts = ["a", "b", "c"]
}
```

**Shell Implementation**:
```sh
rash_string_split() {
    s="$1"
    delimiter="$2"

    # Save IFS
    old_ifs="$IFS"
    IFS="$delimiter"

    # Split into array (POSIX compatible using set --)
    set -- $s

    # Restore IFS
    IFS="$old_ifs"

    # Return count for iteration
    echo "$#"
}
```

**Note**: Returns array via positional parameters due to POSIX limitations.

---

### `len(s: &str) -> usize`
Get string length.

**Rust Signature**:
```rust
use rash::stdlib::string::len;

fn main() {
    let length = len("hello");
    println!("{}", length); // 5
}
```

**Shell Implementation**:
```sh
rash_string_len() {
    s="$1"
    printf '%s' "$s" | wc -c
}
```

**Properties**:
- Empty string: `len("") == 0`
- Multi-byte: Counts bytes, not UTF-8 characters
- No newline counting

---

## Array Module (`rash::stdlib::array`)

### `len(arr: &[T]) -> usize`
Get array length.

**Rust Signature**:
```rust
use rash::stdlib::array::len;

fn main() {
    let items = vec!["a", "b", "c"];
    let count = len(&items);
    println!("{}", count); // 3
}
```

**Shell Implementation**:
```sh
rash_array_len() {
    # Assumes array is passed as space-separated positional params
    echo "$#"
}
```

---

### `join(arr: &[String], separator: &str) -> String`
Join array elements with separator.

**Rust Signature**:
```rust
use rash::stdlib::array::join;

fn main() {
    let items = vec!["a", "b", "c"];
    let result = join(&items, ", ");
    println!("{}", result); // "a, b, c"
}
```

**Shell Implementation**:
```sh
rash_array_join() {
    separator="$1"
    shift

    result=""
    first=1
    for item in "$@"; do
        if [ "$first" = 1 ]; then
            result="$item"
            first=0
        else
            result="${result}${separator}${item}"
        fi
    done

    printf '%s' "$result"
}
```

---

## File System Module (`rash::stdlib::fs`)

### `exists(path: &str) -> bool`
Check if file/directory exists.

**Rust Signature**:
```rust
use rash::stdlib::fs::exists;

fn main() {
    if exists("/etc/passwd") {
        println!("File exists");
    }
}
```

**Shell Implementation**:
```sh
rash_fs_exists() {
    path="$1"
    test -e "$path"
}
```

**Properties**:
- Returns true for files, directories, symlinks
- Follows symlinks
- Safe with special characters in paths

---

### `read_file(path: &str) -> Result<String>`
Read entire file to string.

**Rust Signature**:
```rust
use rash::stdlib::fs::read_file;

fn main() {
    match read_file("/etc/hostname") {
        Ok(content) => println!("{}", content),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Shell Implementation**:
```sh
rash_fs_read_file() {
    path="$1"
    if [ ! -f "$path" ]; then
        echo "ERROR: File not found: $path" >&2
        return 1
    fi
    cat "$path"
}
```

**Properties**:
- Fails if file doesn't exist
- Fails if not a regular file
- Preserves newlines and binary data

---

### `write_file(path: &str, content: &str) -> Result<()>`
Write string to file (overwrites).

**Rust Signature**:
```rust
use rash::stdlib::fs::write_file;

fn main() {
    write_file("/tmp/test.txt", "Hello, World!").unwrap();
}
```

**Shell Implementation**:
```sh
rash_fs_write_file() {
    path="$1"
    content="$2"
    printf '%s' "$content" > "$path"
}
```

**Properties**:
- Overwrites existing file
- Creates file if doesn't exist
- Atomic write (uses printf > redirection)

---

## Prelude Module (`rash::stdlib::prelude`)

Auto-imported essentials:
```rust
pub use crate::stdlib::string::{trim, contains, len};
pub use crate::stdlib::fs::{exists, read_file, write_file};
```

---

## Implementation Plan

### Phase 1: String Module (Sprint 22.1 - 2 hours)
- ✅ Implement `trim()`, `contains()`, `len()`
- ✅ RED-GREEN-REFACTOR for each function
- ✅ Property tests for each function
- ✅ Integration tests with real shell execution

### Phase 2: Array Module (Sprint 22.2 - 1 hour)
- ✅ Implement `len()`, `join()`
- ✅ Test edge cases (empty arrays, single element)
- ✅ Property tests

### Phase 3: File System Module (Sprint 22.3 - 2 hours)
- ✅ Implement `exists()`, `read_file()`, `write_file()`
- ✅ Test with tempfiles
- ✅ Security tests (path traversal, injection)

### Phase 4: Integration & Polish (Sprint 22.4 - 1 hour)
- ✅ Dead code elimination (only emit used functions)
- ✅ Update examples
- ✅ Update documentation
- ✅ Release v0.9.0

---

## Testing Strategy

### Unit Tests
Each function has comprehensive unit tests:
```rust
#[test]
fn test_string_trim_basic() {
    let input = "  hello  ";
    assert_eq!(trim(input), "hello");
}

#[test]
fn test_string_trim_empty() {
    assert_eq!(trim(""), "");
}

#[test]
fn test_string_trim_idempotent() {
    let s = "  test  ";
    assert_eq!(trim(trim(s)), trim(s));
}
```

### Property Tests
```rust
proptest! {
    #[test]
    fn prop_trim_idempotent(s: String) {
        assert_eq!(trim(&trim(&s)), trim(&s));
    }

    #[test]
    fn prop_contains_empty_needle(s: String) {
        assert!(contains(&s, ""));
    }
}
```

### Shell Execution Tests
```rust
#[test]
fn test_shell_execution_trim() {
    let code = transpile(r#"
        use rash::stdlib::string::trim;
        fn main() {
            println!("{}", trim("  hello  "));
        }
    "#);

    let output = execute_shell(&code);
    assert_eq!(output.trim(), "hello");
}
```

---

## Future Extensions (v1.0.0+)

- `string::replace(s, from, to)` - String replacement
- `string::to_upper(s)`, `to_lower(s)` - Case conversion
- `array::map(arr, fn)` - Map function (requires closures)
- `array::filter(arr, fn)` - Filter function
- `fs::mkdir(path)` - Create directory
- `fs::remove(path)` - Remove file
- `fs::copy(src, dst)` - Copy file

---

**Version**: 0.9.0-draft
**Status**: RFC / Planning
**Author**: Rash Development Team
**Date**: 2025-10-03
