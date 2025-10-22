# CONFIG-002: Quote Variable Expansions

**Category**: Security / Reliability
**Severity**: Warning
**Since**: v6.0.0
**Fixable**: Yes (automatic)

## Problem

Unquoted variable expansions can lead to:
- **Word splitting**: Spaces in values break arguments
- **Glob expansion**: Wildcards in values expand unexpectedly
- **Security vulnerabilities**: Injection attacks through unquoted paths

### Example Problem

```bash
# Unquoted variable
export PROJECT_DIR=$HOME/my projects

# Causes issues when used
cd $PROJECT_DIR  # Fails! Splits into: cd /home/user/my projects
```

The space in "my projects" causes the shell to interpret this as two arguments.

## Detection

Rash analyzes variable usage and detects unquoted expansions:

```bash,no_run
bashrs config analyze messy.bashrc
```

Output:

```text
[CONFIG-002] Unquoted variable expansion
  → Line: 1
  → Variable: $HOME
  → Column: 18
  → Can cause word splitting and glob expansion
  → Suggestion: Quote the variable: "${HOME}"
```

## Automatic Fix

Rash automatically adds quotes and converts to brace syntax:

```bash,no_run
bashrs config purify messy.bashrc --output clean.bashrc
```

**Before:**

```bash
export PROJECT_DIR=$HOME/my projects
cd $PROJECT_DIR
cp $SOURCE $DEST
```

**After:**

```bash
export PROJECT_DIR="${HOME}/my projects"
cd "${PROJECT_DIR}"
cp "${SOURCE}" "${DEST}"
```

## Why Quotes Matter

### Word Splitting Example

```bash
# Without quotes
FILE=$HOME/my document.txt
cat $FILE
# Error: cat: /home/user/my: No such file or directory
#        cat: document.txt: No such file or directory

# With quotes (correct)
FILE="${HOME}/my document.txt"
cat "${FILE}"
# Success!
```

### Glob Expansion Example

```bash
# Without quotes
PATTERN="*.txt"
echo $PATTERN
# Expands to: file1.txt file2.txt file3.txt

# With quotes (literal)
PATTERN="*.txt"
echo "${PATTERN}"
# Outputs: *.txt
```

### Security Example

```bash
# Vulnerable
USER_INPUT="file.txt; rm -rf /"
cat $USER_INPUT  # DANGER! Executes: cat file.txt; rm -rf /

# Safe
USER_INPUT="file.txt; rm -rf /"
cat "${USER_INPUT}"  # Safe: cat 'file.txt; rm -rf /'
```

## Implementation

The quoting algorithm:

```rust,no_run
use std::collections::HashMap;

/// Quote all unquoted variables in source
pub fn quote_variables(source: &str) -> String {
    let variables = analyze_unquoted_variables(source);

    if variables.is_empty() {
        return source.to_string();
    }

    let mut lines_to_fix = HashMap::new();
    for var in &variables {
        lines_to_fix.entry(var.line).or_insert_with(Vec::new).push(var);
    }

    let mut result = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if lines_to_fix.contains_key(&line_num) {
            if line.contains('=') {
                // Assignment: quote RHS
                result.push(quote_assignment_line(line));
            } else {
                // Command: quote individual variables
                result.push(quote_command_line(line));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

// Helper functions (part of actual implementation)
fn analyze_unquoted_variables(source: &str) -> Vec<UnquotedVariable> { vec![] }
fn quote_assignment_line(line: &str) -> String { line.to_string() }
fn quote_command_line(line: &str) -> String { line.to_string() }
struct UnquotedVariable { line: usize }
```

## Special Contexts

CONFIG-002 is smart about when NOT to quote:

### 1. Already Quoted

```bash
# Already safe - no change
export DIR="${HOME}/projects"
echo "Hello $USER"
```

### 2. Arithmetic Context

```bash
# Arithmetic - no quotes needed
result=$((x + y))
((counter++))
```

### 3. Array Indices

```bash
# Array index - no quotes needed
element="${array[$i]}"
```

### 4. Export Without Assignment

```bash
# Just exporting, not assigning - no change
export PATH
```

## Testing

Comprehensive test coverage for CONFIG-002:

```rust
#[test]
fn test_config_002_quote_simple_variable() {
    // ARRANGE
    let source = "export DIR=$HOME/projects";

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, r#"export DIR="${HOME}/projects""#);
}

#[test]
fn test_config_002_preserve_already_quoted() {
    // ARRANGE
    let source = r#"export DIR="${HOME}/projects""#;

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, source, "Should not change already quoted");
}

#[test]
fn test_config_002_idempotent() {
    // ARRANGE
    let source = "export DIR=$HOME/projects";

    // ACT
    let quoted_once = quote_variables(source);
    let quoted_twice = quote_variables(&quoted_once);

    // ASSERT
    assert_eq!(quoted_once, quoted_twice, "Quoting should be idempotent");
}
```

## Real-World Example

Common ~/.bashrc scenario:

```bash
# Before purification
export PROJECT_DIR=$HOME/my projects
export BACKUP_DIR=$HOME/backups

# Aliases with unquoted variables
alias proj='cd $PROJECT_DIR'
alias backup='cp $PROJECT_DIR/file.txt $BACKUP_DIR/'

# Functions
deploy() {
    cd $PROJECT_DIR
    ./build.sh
    cp result.tar.gz $BACKUP_DIR
}
```

After purification:

```bash
# After purification
export PROJECT_DIR="${HOME}/my projects"
export BACKUP_DIR="${HOME}/backups"

# Aliases with quoted variables
alias proj='cd "${PROJECT_DIR}"'
alias backup='cp "${PROJECT_DIR}/file.txt" "${BACKUP_DIR}/"'

# Functions
deploy() {
    cd "${PROJECT_DIR}" || return 1
    ./build.sh
    cp result.tar.gz "${BACKUP_DIR}"
}
```

## Configuration

Control CONFIG-002 behavior:

```bash
# Dry-run to preview changes
bashrs config purify ~/.bashrc --dry-run

# Apply with backup (default: ~/.bashrc.backup.TIMESTAMP)
bashrs config purify ~/.bashrc --fix

# JSON output for tooling
bashrs config analyze ~/.bashrc --format json
```

## Exceptions

CONFIG-002 intelligently skips:

1. **Comments**: Variables in comments are ignored
2. **Strings**: Variables already in double quotes
3. **Arithmetic**: Variables in `$((...))` or `(( ))`
4. **Arrays**: Variables used as array indices

## Related Rules

- [CONFIG-001](./config-001.md): PATH deduplication
- [CONFIG-007](./config-007.md): Validate source paths (security)
- [SEC-003](../../linting/security.md): Command injection prevention

## Performance

CONFIG-002 is highly optimized:

- **Regex-based**: O(n) scanning with compiled regex
- **Incremental**: Only processes lines with variables
- **Idempotent**: Safe to run multiple times
- **Fast**: ~1ms for typical .bashrc files

## FAQ

**Q: Why convert $VAR to ${VAR}?**

A: Brace syntax is more explicit and prevents issues like `$VARname` ambiguity.

**Q: What about single quotes?**

A: Variables in single quotes don't expand. CONFIG-002 focuses on double-quote contexts.

**Q: Can this break my scripts?**

A: Very rarely. Quoting variables is almost always safer. Test with `--dry-run` first.

**Q: What about $0, $1, $2, etc.?**

A: Positional parameters are quoted too: `"${1}"`, `"${2}"`, etc.

## See Also

- [Quote Everything by Default](https://mywiki.wooledge.org/Quotes)
- [ShellCheck SC2086](https://www.shellcheck.net/wiki/SC2086)
- [Bash Pitfalls](https://mywiki.wooledge.org/BashPitfalls)
