# False Positive Testing

This chapter documents bashrs's comprehensive false positive testing framework, which ensures the linter doesn't flag valid bash patterns as errors.

## What the linter refuses to guess (v6.67.0)

Most false positives in bashrs came from one habit: a rule searching a physical
line for a substring, then reporting as if it had analysed shell. v6.67.0 replaces
that habit in five places. Each is worth knowing, because each also tells you what
the linter will *not* report.

### String literals are not code

Shell-syntax rules see a copy of your script in which the *text* inside quotes has
been replaced by inert filler. So this is clean:

```bash
export PATTERN="PMAT-[0-9]{4}"   # `]` is a regex, not a test expression
grep "^Diff in" file.txt         # `in` is a word, not the for/case keyword
```

Quote characters themselves, and every expansion — `$VAR`, `${VAR}`, `$(...)`,
backticks — stay visible, because an expansion is code. In
`echo "[$(date '+%H:%M')] started"` the brackets are text while `date` is a real
command, and both are treated accordingly.

Rules that are *about* quoting (SC1003, SC1078, SC2016, SC2086, …) are excluded
from this and keep seeing your literals — otherwise fixing a false positive would
buy a false negative. If a quote never closes, the whole mask is discarded from
that point rather than silently hiding the rest of the file.

### `[` must be its own word to be a test command

SC1020 and SC1140 now require what POSIX requires. These are all clean:

```bash
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then :; fi   # array subscript
if [[ "$line" =~ ^[[:space:]]*fn ]]; then :; fi      # character class
case "$mode" in [0-7][0-7][0-7]) :; ;; esac          # glob pattern
declare -A M; M["a|b"]="c"                           # associative array key
```

while `[ -f file.txt]` and `[ -f x ] extra` are still reported.

### SEC002 reads command position

An expansion supplying the *command word* is not reported — word splitting there
is the documented dispatcher idiom:

```bash
sh_c='sh -c'
[ "$(id -u)" -ne 0 ] && sh_c='sudo -E sh -c'
$sh_c 'docker version'     # not SEC002; SC2183 covers variable command names
```

An unquoted expansion in *argument* position still is, including inside a command
substitution — a context SEC002 previously got exactly backwards.

### SEC010/SEC014 require taint, and inspect validators

A path built only from literals is not a traversal risk, so nothing is reported.
A real guard that dominates the use clears the finding:

```bash
case "$1" in ""|*..*|/*) echo "bad name" >&2; exit 2 ;; esac
OUT_DIR="build/$1"
mkdir -p "$OUT_DIR"        # clean: the guard tests traversal and aborts
```

A function counts as a validator only if its **body** tests for traversal and
aborts. Naming a no-op `validate_path` no longer silences anything.

Severity is graded by provenance: proven external input reaching an unguarded path
is an `error`; a variable the file never assigns is a guess about the environment
and reports as a `warning`, so it cannot break your build.

### DET002 follows the timestamp to its sink

The defect is a timestamp reaching a reproducible *output*, not the calling of
`date`. These are clean:

```bash
echo "[$(date '+%F %T')] started" | tee -a "$LOG_FILE"     # a log line
STAMP=$(date -u -d "@${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}" +%Y%m%d)
```

Reading `SOURCE_DATE_EPOCH` is the reproducible-builds remedy and clears the rule,
including for whatever the value then feeds. A timestamp landing in an artifact
name, a hash input or a truncating redirect is still reported, and the message now
names the sink line.

## Overview

bashrs uses a **Popper Falsification** methodology - every valid bash pattern must pass the linter without triggering false positives. The test suite currently covers **230 structured tests** across two categories:

- **Falsification Tests (F-codes)**: 130 tests for valid patterns that must NOT trigger warnings
- **Simulation Tests (S-codes)**: 100 tests for edge cases that must NOT cause panics

## Running the Tests

```bash
# Run falsification tests (130 F-code tests)
cargo test -p bashrs --test falsification_probar_testing

# Run simulation tests (100 S-code tests)
cargo test -p bashrs --test simulation_probar_testing

# Run bug hunt tests (aggressive edge case discovery)
cargo test -p bashrs --test linter_bug_hunting -- --nocapture

# Run all
cargo test -p bashrs --test falsification_probar_testing --test simulation_probar_testing --test linter_bug_hunting
```

## Falsification Test Categories (F-codes)

### 6.1 Sudo and Permissions (F001-F010)

These tests ensure sudo patterns don't trigger SC2024 false positives:

```bash
# F001: Sudo redirect wrapped - MUST NOT trigger SC2024
sudo sh -c 'echo 1 > /etc/file'

# F002: Sudo tee pattern - MUST NOT trigger SC2024
echo 1 | sudo tee /etc/file

# F003: Sudo tee with output redirect
echo 1 | sudo tee /etc/file >/dev/null
```

### 6.2 Redirection and Pipes (F011-F020)

Valid redirect patterns that must not trigger SC2069:

```bash
# F011: Stdout/stderr order
cmd 2>&1 | other

# F012: Silence pattern
cmd >/dev/null 2>&1

# F013: Bash shorthand
cmd &> file
```

### 6.3 Quoting and Heredocs (F021-F030)

Quoting patterns that must not trigger SC2016 or SC2035:

```bash
# F021: Single quote heredoc is literal
cat << 'EOF'
$var  # This is literal, not expanded
EOF

# F029: Find name glob - MUST NOT trigger SC2035
find . -name '*.json'

# F030: Grep regex pattern
grep -r '*.c' .
```

### 6.4 Variables and Parameters (F031-F045)

Safe parameter expansion patterns:

```bash
# F031: Safe parameter expansion - MUST NOT trigger SC2086
echo "${var:-default}"

# F035: Array count is numeric
echo ${#arr[@]}

# F036: Arithmetic context
(( var++ ))

# F037: Test context - no word splitting in [[ ]]
[[ -n $var ]]
```

### 6.5 Control Flow (F046-F060)

Control structure patterns:

```bash
# F046: Inline if
if true; then echo yes; fi

# F047: Case with default - variable is defined in all branches
case $x in
  a) y=1 ;;
  *) y=2 ;;
esac
echo $y  # y is always defined

# F048: C-style for loop - i is numeric
for ((i=0; i<10; i++)); do
  echo $i  # MUST NOT trigger SC2086
done
```

### 6.6 Builtins and Environment (F061-F070)

Bash builtin variables that must not trigger SC2154:

```bash
# F061: EUID is a bash builtin
echo $EUID

# F063: BASH_VERSION is builtin
echo $BASH_VERSION

# F065: RANDOM is builtin
echo $RANDOM
```

### 6.7 Subshells and Command Substitution (F071-F080)

```bash
# F071: Subshell scope
( cd dir && cmd )

# F079: Block inside command substitution
x=$( { cmd; } )

# F080: Backticks in assignment context
x=`cmd`  # Legacy but valid in assignments
```

### 6.8 Traps and Signals (F081-F090)

```bash
# F081: Trap with single quotes (correct usage)
trap 'rm $tmpfile' EXIT

# F082: Trap with double quotes (intentional early expansion)
trap "echo $v" INT  # User chose double quotes intentionally
```

### 6.9 Parsing and Formatting (F091-F100)

```bash
# F096: Regex operator
[[ $x =~ ^[a-z]+$ ]]

# F098: Brace expansion
echo {1..10}

# F100: ANSI-C quoting
echo $'\t'
```

### 6.10 Arrays (F101-F110)

```bash
# F101: Array index access
arr=(a b c); echo ${arr[0]}

# F103: Associative array
declare -A map; map[key]=val

# F106: Array iteration (properly quoted)
for i in "${arr[@]}"; do echo "$i"; done

# F109: Array indices
echo ${!arr[@]}
```

### 6.11 String Operations (F111-F120)

```bash
# F111: Substring extraction
echo ${var:0:5}

# F112: Pattern substitution
echo ${var/old/new}

# F114: Lowercase transform
echo ${var,,}

# F116: String length
echo ${#var}
```

### 6.12 Arithmetic (F121-F130)

```bash
# F121: Basic arithmetic
echo $((1+2))

# F123: Increment operator
(( i++ ))

# F128: Hex literal
echo $((16#FF))

# F130: Ternary operator
echo $((x<y ? x : y))
```

## Simulation Test Categories (S-codes)

Simulation tests verify that bashrs handles edge cases without panicking.

### S1xx: Unicode and Encoding

```bash
# S101: Latin extended
echo 'héllo wörld'

# S103: Emoji
echo '🚀🔥💻'

# S104: RTL Arabic
var='مرحبا'; echo $var
```

### S2xx: Boundary Conditions

```bash
# S201: 10KB variable
x=aaaa...  # 10,000 characters

# S207: 500-line heredoc
cat << EOF
line 1
...
line 500
EOF

# S208: 20 nested expansions
echo ${x:-${x:-${x:-...}}}
```

### S3xx: Deep Nesting

```bash
# S301: 10 nested ifs
if true; then if true; then ... fi; fi

# S307: 10 nested command substitutions
$($($(...)))
```

### S5xx: Malformed Syntax (Graceful Errors)

```bash
# S501: Unclosed brace - should error gracefully
echo ${

# S503: Missing fi - should error gracefully
if true; then
```

## Adding New Tests

### Adding a Falsification Test

```rust,ignore
// In rash/tests/falsification_probar_testing.rs
// Add to the appropriate category test function
("F131", "your_code_here", "SC2086"),  // Must NOT trigger SC2086
```

### Adding a Simulation Test

```rust,ignore
// In rash/tests/simulation_probar_testing.rs
// Add to the appropriate category test function
("S1011", "code_here", "Description"),  // Must NOT panic
```

## Bug Hunt Tests

Bug hunt tests aggressively probe edge cases and **report** bugs without failing tests.
This allows continuous discovery of potential issues.

### Core Bug Hunt

```bash
cargo test -p bashrs --test linter_bug_hunting -- --nocapture
```

Categories:
- **Unicode Edge Cases**: Japanese, emoji, RTL, combining diacriticals
- **Extreme Nesting**: 5-100 levels of command/param substitution
- **Large Inputs**: 100-50000 character variables, arrays
- **False Positives**: Edge cases that might trigger false warnings
- **Malformed Syntax**: Graceful error recovery testing
- **Escape Sequences**: Hex, octal, ANSI-C escapes

### TUI/Pixel Bug Hunt

```bash
cargo test -p bashrs --test linter_tui_bug_hunting -- --nocapture
```

Categories:
- **Frame Rendering**: TUI frame construction and box drawing
- **Unicode Rendering**: Width consistency with unicode content
- **Snapshot Stability**: Deterministic output verification
- **Frame Sequences**: Multi-frame transition testing
- **Pixel Alignment**: Vertical character alignment
- **Diagnostic Formatting**: Warning/error message display
- **Frame Assertions**: expect_frame() validation
- **Content Truncation**: Long content handling

## Quality Gates

| Test Suite | Count | Target |
|------------|-------|--------|
| Falsification | 130 | 100% pass |
| Simulation | 100 | 100% pass |
| Core Bug Hunt | 6 categories | Report bugs |
| TUI Bug Hunt | 8 categories | Report bugs |
| **Total** | **230+** | **100%** |

All tests must pass before any release.

## See Also

- [Linting Rules Reference](../reference/rules.md)
- [Configuration Reference](../reference/configuration.md)
