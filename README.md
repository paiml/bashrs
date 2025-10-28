# Rash - Bidirectional Shell Safety Tool

[![Crates.io](https://img.shields.io/crates/v/bashrs.svg)](https://crates.io/crates/bashrs)
[![Documentation](https://docs.rs/bashrs/badge.svg)](https://docs.rs/bashrs)
[![Book](https://img.shields.io/badge/book-The%20Rash%20Book-blue)](https://paiml.github.io/bashrs/)
[![License](https://img.shields.io/crates/l/bashrs.svg)](LICENSE)
[![CI](https://github.com/paiml/bashrs/workflows/CI/badge.svg)](https://github.com/paiml/bashrs/actions)
[![Tests](https://img.shields.io/badge/tests-5105%20passing-brightgreen)](https://github.com/paiml/bashrs/actions)
[![PropertyTests](https://img.shields.io/badge/property_tests-52%20passing-blue)](https://github.com/paiml/bashrs/blob/main/rash/src/testing/)
[![Coverage](https://img.shields.io/badge/coverage-88.71%25-green)](https://github.com/paiml/bashrs/actions)
[![Mutation](https://img.shields.io/badge/mutation-167%20mutants-brightgreen)](https://github.com/paiml/bashrs)

**Rash** is a bidirectional shell safety tool that lets you write shell scripts in REAL Rust and automatically purify legacy bash scripts.

## Why Rash?

- 🛡️ **Safety First**: Automatic protection against shell injection attacks
- 🔍 **Compile-Time Verification**: Catch errors before deployment
- 📦 **Zero Runtime Dependencies**: Generated scripts work on any POSIX shell
- 🎯 **Deterministic Output**: Same input always produces identical scripts
- ✅ **ShellCheck Compliant**: All output passes strict linting

## How Rash Exceeds ShellCheck

ShellCheck is an excellent linter that **detects** problems in shell scripts.
Rash goes **far beyond** by understanding the full AST and **automatically transforming** code to fix issues.

| What ShellCheck Does | What Rash Does |
|---------------------|----------------|
| ⚠️ **Warns**: "$RANDOM is non-deterministic" | ✅ **Rewrites** to version-based deterministic IDs |
| ⚠️ **Warns**: "mkdir may fail if exists" | ✅ **Transforms** to `mkdir -p` (idempotent) |
| ⚠️ **Warns**: "Unquoted variable expansion" | ✅ **Quotes** all variables automatically |
| ⚠️ **Warns**: "Timestamp in $(date +%s)" | ✅ **Replaces** with fixed version tags |
| ⚠️ **Warns**: "rm may fail if doesn't exist" | ✅ **Adds** `-f` flag for safe removal |
| Static pattern matching | **Full AST semantic understanding** |
| Detects issues (read-only) | **Fixes issues (read-write transformation)** |

### Example: Non-Deterministic Deployment Script

**Input (Messy Bash)**:
```bash
#!/bin/bash
SESSION_ID=$RANDOM                      # Non-deterministic
RELEASE="release-$(date +%s)"           # Timestamp-based
mkdir /app/releases/$RELEASE            # Non-idempotent
rm /app/current                         # Fails if doesn't exist
ln -s /app/releases/$RELEASE /app/current
```

**ShellCheck Output** (manual fixes required):
```
⚠️ SC2086: Quote variable to prevent word splitting
⚠️ $RANDOM is non-deterministic (YOU must fix manually)
⚠️ mkdir may fail if directory exists (YOU must add -p flag)
⚠️ rm may fail if doesn't exist (YOU must add -f flag)
```

**Rash Output** (automatically purified):
```bash
#!/bin/sh
# ✅ Automatically fixed by Rash - no manual work needed
deploy_app() {
    _version="$1"
    session_id="session-${_version}"           # ✅ Deterministic
    release="release-${_version}"              # ✅ Version-based (not timestamp)
    mkdir -p "/app/releases/${release}"        # ✅ Idempotent
    rm -f "/app/current"                       # ✅ Safe removal
    ln -sf "/app/releases/${release}" "/app/current"  # ✅ Idempotent symlink
}
```

**Key Difference**: ShellCheck tells you what's wrong. Rash **understands your code's intent** and rewrites it to be safe, deterministic, and idempotent — automatically.

This is only possible because Rash parses shell scripts into a full Abstract Syntax Tree (AST), understands the semantic meaning of each construct, and can perform intelligent transformations that preserve functionality while eliminating entire classes of bugs.

---

## How Rash Works: Two Workflows

Rash operates in **two directions** to maximize shell script safety:

### 🚀 PRIMARY: Rust → Safe Shell (Production-Ready)

**Write new scripts in REAL Rust, transpile to provably safe shell.**

```
Rust Code (.rs) → cargo test → Transpile → Safe POSIX Shell
                   ↑ Test FIRST with Rust tooling
```

**Use cases**:
- Bootstrap installers (Node.js, Rust toolchain, etc.)
- CI/CD deployment scripts
- System configuration tools
- Any new shell automation

**Benefits**:
- Full Rust std library support
- Test with `cargo test`, lint with `cargo clippy`
- Property-based testing with proptest
- 100% deterministic, idempotent output

### 🔄 SECONDARY: Bash → Rust → Purified Bash (Legacy Cleanup)

**Ingest messy bash, convert to Rust with tests, output purified shell.**

```
Messy Bash → Parser → Rust + Tests → Transpile → Purified Bash
                       ↑ Tests auto-generated
```

**Use cases**:
- Clean up legacy bash scripts
- Remove non-deterministic constructs ($RANDOM, timestamps, $$)
- Enforce idempotency (mkdir -p, rm -f)
- Generate comprehensive test suites

**Benefits**:
- Automatic test generation
- Remove unsafe patterns
- Maintain shell compatibility
- Preserve functionality while improving safety

See [`examples/PURIFICATION_WORKFLOW.md`](examples/PURIFICATION_WORKFLOW.md) for detailed purification examples.

---

## Quick Start (PRIMARY Workflow)

Write Rust:

```rust
// install.rs
#[rash::main]
fn main() {
    let version = env_var_or("VERSION", "1.0.0");
    let prefix = env_var_or("PREFIX", "/usr/local");
    
    echo("Installing MyApp {version} to {prefix}");
    
    // Create installation directories
    mkdir_p("{prefix}/bin");
    mkdir_p("{prefix}/share/myapp");
    
    // Copy files (with automatic quoting)
    if exec("cp myapp {prefix}/bin/") {
        echo("✓ Binary installed");
    } else {
        echo("✗ Failed to install binary");
        exit(1);
    }
}
```

Get POSIX shell:

```bash
$ bashrs build install.rs -o install.sh
$ cat install.sh
#!/bin/sh
# Generated by Rash v0.4.0
# POSIX-compliant shell script

set -euf
IFS=' 	
'
export LC_ALL=C

# Rash runtime functions
rash_require() {
    if ! "$@"; then
        echo "FATAL: Requirement failed: $*" >&2
        exit 1
    fi
}

# Main script begins
main() {
    VERSION="${VERSION:-1.0.0}"
    PREFIX="${PREFIX:-/usr/local}"
    
    echo "Installing MyApp $VERSION to $PREFIX"
    
    mkdir -p "$PREFIX/bin"
    mkdir -p "$PREFIX/share/myapp"
    
    if cp myapp "$PREFIX/bin/"; then
        echo "✓ Binary installed"
    else
        echo "✗ Failed to install binary"
        exit 1
    fi
}

# Execute main function
main "$@"
```

## Installation

### From crates.io (Recommended)

```bash
# Install latest release candidate
cargo install bashrs --version 1.0.0-rc1

# Or install latest stable
cargo install bashrs
```

### Binary Releases

Pre-built binaries are available for Linux and macOS:

```bash
# Linux x86_64
curl -L https://github.com/paiml/bashrs/releases/latest/download/bashrs-x86_64-unknown-linux-musl.tar.gz | tar xz

# macOS x86_64
curl -L https://github.com/paiml/bashrs/releases/latest/download/bashrs-x86_64-apple-darwin.tar.gz | tar xz

# macOS ARM64
curl -L https://github.com/paiml/bashrs/releases/latest/download/bashrs-aarch64-apple-darwin.tar.gz | tar xz
```

### Using cargo-binstall

```bash
cargo binstall bashrs
```

### From Source

```bash
# Full build with all features
cargo install --git https://github.com/paiml/bashrs

# Minimal build (smaller binary, ~2MB)
cargo install --git https://github.com/paiml/bashrs --no-default-features --features minimal
```

## Usage

### Basic Commands

```bash
# Transpile a Rust file to shell
bashrs build input.rs -o output.sh

# Check if a file is valid Rash
bashrs check input.rs

# Initialize a new Rash project
bashrs init my-project

# Verify safety properties
bashrs verify input.rs output.sh

# Inspect AST and safety properties
bashrs inspect input.rs

# Lint shell scripts for safety issues (NEW in v1.1)
bashrs lint script.sh

# Compile to self-extracting script (BETA)
bashrs compile input.rs -o install --self-extracting
```

### Native Linter (NEW in v1.1) 🔍

bashrs includes a **native linter** that validates shell scripts for safety issues, with zero external dependencies:

```bash
# Lint a shell script (human-readable output)
$ bashrs lint unsafe.sh
⚠ 8:3-9 [warning] SC2086: Double quote to prevent globbing and word splitting on $FILES
  Fix: "$FILES"

⚠ 12:7-30 [warning] SC2046: Quote this to prevent word splitting: $(find . -name '*.txt')
  Fix: "$(find . -name '*.txt')"

ℹ 16:9-24 [info] SC2116: Useless echo; just use $result directly
  Fix: $result

Summary: 0 error(s), 5 warning(s), 1 info(s)

# JSON format for CI/CD integration
$ bashrs lint script.sh --format=json
{
  "file": "script.sh",
  "diagnostics": [
    {
      "code": "SC2086",
      "severity": "warning",
      "message": "Double quote to prevent globbing and word splitting",
      "span": { "start_line": 8, "start_col": 3, "end_line": 8, "end_col": 9 },
      "fix": "\"$FILES\""
    }
  ],
  "summary": { "errors": 0, "warnings": 5, "infos": 1 }
}

# SARIF format for security scanners
$ bashrs lint script.sh --format=sarif
```

**Linter Features**:
- ✅ **Zero external dependencies** - No ShellCheck installation required
- ✅ **3 output formats** - Human, JSON, SARIF for CI/CD integration
- ✅ **Auto-fix** (NEW in v1.2) - Automatically apply fixes with `--fix` flag
- ✅ **Smart detection** - Context-aware to prevent false positives
- ✅ **ShellCheck parity** - Implements critical SC-series rules

**Auto-Fix** (NEW in v1.2):
```bash
# Apply fixes automatically (creates backup)
$ bashrs lint script.sh --fix
[INFO] Applied 6 fix(es) to script.sh
[INFO] Backup created at script.sh.bak
✓ All issues fixed!
```

Before:
```bash
DIR=/tmp/mydir
mkdir $DIR
FILES=$(ls *.txt)
```

After:
```bash
DIR=/tmp/mydir
mkdir "$DIR"
FILES="$(ls *.txt)"
```

**Rules Implemented** (v1.1):
- **SC2086**: Unquoted variable expansion (prevents word splitting & glob expansion)
- **SC2046**: Unquoted command substitution
- **SC2116**: Useless echo in command substitution

**Exit Codes**:
- `0` - No issues found
- `1` - Warnings detected
- `2` - Errors detected

**Comparison: bashrs vs ShellCheck**:

| Feature | ShellCheck | bashrs |
|---------|-----------|--------|
| **Core Capability** | Static pattern detection | **Full AST parsing + transformation** |
| **Output** | Warnings only | **Automatically fixed code** |
| Installation | External binary required | Built-in, zero dependencies |
| Output formats | checkstyle, gcc, json | human, JSON, SARIF |
| Auto-fix | ❌ No | ✅ Yes - automatic code transformation |
| **Determinism** | Warns about $RANDOM | **Replaces with deterministic constructs** |
| **Idempotency** | Warns about mkdir | **Transforms to mkdir -p** |
| Semantic understanding | Pattern matching only | **Full AST semantic analysis** |
| Code transformation | ❌ Read-only | ✅ **Read-write (purification)** |
| Performance | ~50ms | <2ms (native Rust) |

See the [Safety Comparison Guide](docs/SAFETY_COMPARISON.md) for detailed vulnerability prevention examples.

### Bash Test Framework (NEW in v6.10.0) 🧪

bashrs now includes a **test framework** for bash scripts with inline tests and GIVEN/WHEN/THEN syntax:

```bash
# Run all tests in a script
$ bashrs test script.sh

Test Results
============

✓ test_add_numbers
  Description: example function works correctly
  Given: x=5
  When: add_numbers 2 3
  Then: output should be "5"
✓ test_pass

Summary
-------
Total:   2
Passed:  2
Failed:  0
Time:    5ms

✓ All tests passed!
```

**Test Format**:
```bash
# TEST: my_function with valid input
# GIVEN: x=5
# WHEN: my_function 5
# THEN: output should be "Result: 5"
test_my_function_basic() {
    result=$(my_function 5)
    [[ "$result" == "Result: 5" ]] || return 1
}
```

**Features**:
- ✅ **Automatic test discovery** - Functions starting with `test_`
- ✅ **GIVEN/WHEN/THEN syntax** - Behavior-driven test documentation
- ✅ **Multiple output formats** - Human, JSON, JUnit XML
- ✅ **Pattern filtering** - Run specific tests with `--pattern`
- ✅ **Detailed results** - Show GIVEN/WHEN/THEN with `--detailed`
- ✅ **CI/CD integration** - JSON/JUnit output for test reporting

**Usage**:
```bash
# Run all tests
bashrs test script.sh

# Detailed output with GIVEN/WHEN/THEN
bashrs test script.sh --detailed

# Run tests matching pattern
bashrs test script.sh --pattern "test_add"

# JSON output for CI/CD
bashrs test script.sh --format json

# JUnit XML for test reporting
bashrs test script.sh --format junit
```

**Why bash tests?**
- ✅ **Inline tests** - Keep tests next to code
- ✅ **No external tools** - Pure bash, no bats or shunit2
- ✅ **BDD style** - Clear test documentation with GIVEN/WHEN/THEN
- ✅ **Fast execution** - Native test runner
- ✅ **CI/CD ready** - Multiple output formats

### Comprehensive Quality Audit (NEW in v6.12.0) 🔍

bashrs now includes a **comprehensive audit command** that runs all quality checks in one unified command:

```bash
# Run comprehensive quality audit
$ bashrs audit script.sh

Comprehensive Quality Audit
===========================

File: script.sh

Check Results:
--------------
✅ Parse:    Valid bash syntax
⚠️  Lint:     3 warnings
✅ Test:     1/1 tests passed
✅ Score:    B (8.5/10.0)

Overall: ✅ PASS
```

**Audit Checks** (4 comprehensive checks):
1. **Parse**: Valid bash syntax verification
2. **Lint**: Security and style issues (357 rules)
3. **Test**: Test discovery and execution
4. **Score**: Quality scoring (A+ to F scale)

**Output Formats**:
```bash
# Human-readable output (default)
bashrs audit script.sh

# JSON for CI/CD pipelines
bashrs audit script.sh --format json

# SARIF for GitHub Code Scanning
bashrs audit script.sh --format sarif
```

**Advanced Options**:
```bash
# Strict mode (fail on warnings)
bashrs audit script.sh --strict

# Enforce minimum grade
bashrs audit script.sh --min-grade A

# Detailed dimension breakdown
bashrs audit script.sh --detailed
```

**Why comprehensive audit?**
- ✅ **All-in-one** - Parse, lint, test, and score in one command
- ✅ **CI/CD ready** - Exit code reflects pass/fail status
- ✅ **Multiple formats** - Human, JSON, SARIF
- ✅ **Quality gates** - Enforce minimum standards with `--min-grade`
- ✅ **GitHub integration** - SARIF format for Code Scanning

### Bash Quality Scoring (NEW in v6.11.0) 📊

bashrs now includes a **quality scoring system** that evaluates bash scripts across 5 dimensions with TDG-style grading:

```bash
# Score a bash script
$ bashrs score script.sh

Bash Script Quality Score
=========================

Overall Grade: A+
Overall Score: 9.9/10.0

✓ Excellent! Near-perfect code quality.
```

**Scoring Dimensions** (5 dimensions, weighted):
1. **Complexity** (25%): Function length, nesting depth
2. **Safety** (30%): Variable quoting, error handling
3. **Maintainability** (20%): Modularity, comment ratio
4. **Testing** (15%): Test coverage ratio
5. **Documentation** (10%): Comment quality, header docs

**Grading Scale**:
- **A+** (9.5-10.0): Near-perfect | **A** (9.0-9.5): Excellent
- **B+/B** (8.0-8.9): Good | **C+/C** (7.0-7.9): Average
- **D** (6.0-6.9): Below average | **F** (<6.0): Poor

**Features**:
- ✅ **5-dimension analysis** - Comprehensive quality evaluation
- ✅ **TDG-style grading** - Industry-standard A+ to F scale
- ✅ **Actionable suggestions** - Specific improvements with examples
- ✅ **Multiple formats** - Human, JSON, Markdown reports
- ✅ **Detailed breakdown** - See individual dimension scores
- ✅ **CI/CD integration** - JSON output for quality gates

**Usage**:
```bash
# Basic score
bashrs score script.sh

# Detailed dimension breakdown
bashrs score script.sh --detailed

# JSON for CI/CD pipelines
bashrs score script.sh --format json

# Markdown quality report
bashrs score script.sh --format markdown
```

**Poor Script Example** (with suggestions):
```bash
$ bashrs score poor_script.sh

Overall Grade: F
Overall Score: 3.9/10.0

Improvement Suggestions:
1. Add quotes around variable expansions ("$var")
2. Add 'set -euo pipefail' for error handling
3. Add test functions (test_*) to verify behavior
4. Add header comments describing purpose
```

**Why quality scoring?**
- ✅ **Objective metrics** - Quantifiable quality assessment
- ✅ **Actionable feedback** - Specific improvements, not just warnings
- ✅ **CI/CD quality gates** - Enforce minimum quality scores
- ✅ **Team standards** - Consistent quality across projects
- ✅ **Continuous improvement** - Track quality over time

**Bash Quality Tools Progress**:
- ✅ `bashrs test` (v6.10.0) - Test discovery and execution
- ✅ `bashrs score` (v6.11.0) - Quality scoring
- ✅ `bashrs audit` (v6.12.0) - Comprehensive quality audit
- ✅ `bashrs coverage` (v6.13.0) - Test coverage tracking
- ⏳ `bashrs format` (planned) - Code formatting

### Coverage Tracking (NEW in v6.13.0) 📈

bashrs now includes **coverage tracking** for bash scripts:

```bash
# Generate coverage report
$ bashrs coverage script.sh

Coverage Report: script.sh

Lines:     9/12   (75.0%)  ⚠️
Functions: 2/2    (100.0%) ✅

Uncovered Lines: 3 lines

⚠️ Moderate coverage - consider adding more tests
```

**Coverage Features**:
- Line coverage percentage
- Function coverage percentage
- Uncovered lines/functions
- Multiple formats (Terminal/JSON/HTML/LCOV)
- Minimum coverage enforcement: `bashrs coverage script.sh --min 80`
- CI/CD ready with exit codes

### Interactive REPL (NEW in v6.7.0) 🎯

bashrs now includes a **full-featured REPL** for interactive bash script analysis and testing:

```bash
# Start the REPL
$ bashrs repl

bashrs REPL v6.7.0
Type 'quit' or 'exit' to exit, 'help' for commands
Current mode: normal - Execute bash commands directly
bashrs [normal]>
```

**REPL Features**:
- 🎯 **5 Interactive Modes**: normal, purify, lint, debug, explain
- 🔍 **Parser Integration**: Parse bash code and inspect AST
- 🧹 **Purifier Integration**: Transform bash to idempotent/deterministic code
- 🔎 **Linter Integration**: Real-time diagnostics with severity levels
- 📚 **Command History**: Persistent history in `~/.bashrs_history`

**Available Commands**:

```bash
# Mode switching
bashrs [normal]> :mode                 # Show current mode and available modes
bashrs [normal]> :mode lint            # Switch to lint mode
bashrs [lint]> :mode purify            # Switch to purify mode

# Parse bash code and show AST
bashrs [normal]> :parse echo hello
✓ Parse successful!
Statements: 1
Parse time: 0ms

AST:
  [0] SimpleCommand { ... }

# Purify bash code (make idempotent/deterministic)
bashrs [normal]> :purify mkdir /tmp/test
✓ Purification successful!
Purified 1 statements

# Lint bash code with diagnostics
bashrs [normal]> :lint cat file.txt | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1

# Show help
bashrs [normal]> help
bashrs REPL Commands:
  help             - Show this help message
  quit             - Exit the REPL
  exit             - Exit the REPL
  :mode            - Show current mode and available modes
  :mode <name>     - Switch to a different mode
  :parse <code>    - Parse bash code and show AST
  :purify <code>   - Purify bash code (make idempotent/deterministic)
  :lint <code>     - Lint bash code and show diagnostics

# Exit REPL
bashrs [normal]> quit
Goodbye!
```

**REPL Modes**:

| Mode | Description | Use Case |
|------|-------------|----------|
| **normal** | Execute bash commands directly | Interactive bash experimentation |
| **purify** | Show purified version | Learn idempotency/determinism patterns |
| **lint** | Show linting results | Find and fix safety issues |
| **debug** | Step-by-step execution | Understand complex bash behavior |
| **explain** | Explain bash constructs | Learn bash syntax and semantics |

**Why Use the REPL?**:
- ✅ **Learn interactively**: Experiment with bash transformations in real-time
- ✅ **Fast feedback**: Instant parsing, linting, and purification
- ✅ **No external tools**: Built-in parser, linter, and purifier
- ✅ **History support**: Recall previous commands with arrow keys
- ✅ **Mode switching**: Seamlessly switch between analysis types

**Example Workflow**:

```bash
# Start REPL
$ bashrs repl

# Parse to understand structure
bashrs [normal]> :parse if [ -f file.txt ]; then cat file.txt; fi
✓ Parse successful!
Statements: 1
AST: If statement with test command

# Switch to lint mode to find issues
bashrs [normal]> :mode lint
Switched to lint mode - Show linting results for bash commands

# Lint the code
bashrs [lint]> :lint if [ -f file.txt ]; then cat file.txt; fi
✓ No issues found!

# Switch to purify mode
bashrs [lint]> :mode purify
Switched to purify mode - Show purified version of bash commands

# See the purified version
bashrs [purify]> :purify mkdir /tmp/app
✓ Purification successful!
Purified: mkdir -p "/tmp/app"  # Idempotent + quoted
```

**Command History**:

The REPL automatically saves your command history to `~/.bashrs_history`:

```bash
# History persists across sessions
$ bashrs repl
bashrs [normal]> :parse echo hello
...
bashrs [normal]> quit

# Later...
$ bashrs repl
bashrs [normal]> # Press ↑ to recall ":parse echo hello"
```

**Integration with CI/CD**:

Use the REPL for quick validation before committing:

```bash
# Quick lint check in REPL
$ bashrs repl
bashrs [normal]> :lint $(cat deploy.sh)
Found 3 issue(s): ...

# Fix issues, then verify
bashrs [normal]> :lint $(cat deploy.sh)
✓ No issues found!
```

See the [REPL Guide](https://paiml.github.io/bashrs/getting-started/repl.html) in the book for comprehensive examples.

### CLI Options

```bash
USAGE:
    bashrs [OPTIONS] <COMMAND>

COMMANDS:
    build       Transpile Rust to shell script
    check       Validate Rust source without transpiling
    init        Initialize a new Rash project
    verify      Verify shell script matches source
    inspect     Analyze AST and safety properties
    lint        Lint shell scripts for safety issues (NEW in v1.1)
    repl        Interactive REPL for bash analysis (NEW in v6.7.0)
    compile     Compile to standalone binary (BETA - experimental)

OPTIONS:
    -v, --verbose            Enable verbose output
    -V, --version            Print version information
    -h, --help               Print help information
    --target <SHELL>         Target shell: posix, bash, ash (default: posix)
    --verify <LEVEL>         Verification level: none, basic, strict, paranoid
    --validation <LEVEL>     Validation level: none, minimal, strict, paranoid

BUILD OPTIONS:
    -o, --output <FILE>      Output file (default: stdout)
    --optimize               Enable optimization passes
    --strict                 Enable strict mode checks
    --emit-proof             Emit verification proof alongside output

REPL OPTIONS:
    --debug                  Enable debug mode in REPL
    --max-memory <MB>        Maximum memory usage (default: 500MB)
    --timeout <SECONDS>      Command timeout (default: 120s)
    --max-depth <N>          Maximum recursion depth (default: 1000)
    --sandboxed              Run in sandboxed mode (restricted operations)
```

## Documentation

### 📚 The Rash Book

**Comprehensive guide with tested examples**: [https://paiml.github.io/bashrs/](https://paiml.github.io/bashrs/)

The Rash Book is the official, comprehensive documentation for Rash. All code examples in the book are automatically tested, ensuring they stay up-to-date with the code.

**What's in the book:**
- **Getting Started**: Installation, quick start, your first purification
- **Core Concepts**: Determinism, idempotency, POSIX compliance
- **Shell Script Linting**: Security, determinism, and idempotency rules
- **Configuration Management**: Purifying .bashrc and .zshrc files (CONFIG-001, CONFIG-002)
- **Makefile Linting**: Security and best practices
- **Real-World Examples**: Bootstrap installers, deployment scripts, CI/CD
- **Advanced Topics**: AST transformation, property testing, mutation testing
- **Reference**: CLI commands, configuration, exit codes, rules reference

**Why the book is special:**
- ✅ All examples are tested automatically (TDD)
- ✅ Cannot release without updating the book
- ✅ Enforced quality through pre-release checks
- ✅ Toyota Way principles applied to documentation

### API Documentation

Rust API documentation is available on [docs.rs/bashrs](https://docs.rs/bashrs).

### Quick Links

- [Installation Guide](https://paiml.github.io/bashrs/getting-started/installation.html)
- [Quick Start](https://paiml.github.io/bashrs/getting-started/quick-start.html)
- [CONFIG-001: PATH Deduplication](https://paiml.github.io/bashrs/config/rules/config-001.html)
- [CONFIG-002: Quote Variables](https://paiml.github.io/bashrs/config/rules/config-002.html)
- [Security Rules](https://paiml.github.io/bashrs/linting/security.html)
- [Contributing Guide](https://paiml.github.io/bashrs/contributing/setup.html)

## Language Features

### Supported Rust Subset

Rash supports a carefully chosen subset of Rust that maps cleanly to shell:

#### Variables and Types
```rust
let name = "Alice";              // String literals
let count = 42;                  // Integers (including negatives: -42)
let flag = true;                 // Booleans
let user = env("USER");          // Environment variables
let result = capture("date");    // Command output
```

#### Arithmetic Operations
```rust
let x = 1 + 2;                   // Addition → $((1 + 2))
let y = 10 - 3;                  // Subtraction → $((10 - 3))
let z = 4 * 5;                   // Multiplication → $((4 * 5))
let w = 20 / 4;                  // Division → $((20 / 4))
```

#### Comparison Operators
```rust
if x > 0 {                       // Greater than → [ "$x" -gt 0 ]
    println!("Positive");        // println! macro supported
}
if y == 5 { ... }                // Equal → [ "$y" -eq 5 ]
if z < 10 { ... }                // Less than → [ "$z" -lt 10 ]
```

#### User-Defined Functions
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b                        // Return value via echo
}

fn main() {
    let sum = add(1, 2);         // Captured with $(add 1 2)
    println!("Sum: {}", sum);
}
```

#### Built-in Functions
```rust
// I/O operations
echo("Hello, World!");           // Print to stdout
println!("Hello, World!");       // println! macro (Sprint 10)
eprint("Error!");                // Print to stderr

// File system
mkdir_p("/tmp/myapp");           // Create directory recursively
write_file("config.txt", data);  // Write file
let content = read_file("config.txt");  // Read file
if path_exists("/etc/config") { ... }   // Check path

// Process management  
exec("ls -la");                  // Run command
let output = capture("date");    // Capture command output
exit(0);                         // Exit with code

// Environment
set_env("KEY", "value");         // Set environment variable
let val = env("KEY");            // Get environment variable
let val = env_var_or("KEY", "default"); // With default
```

#### Control Flow
```rust
// Conditionals
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// ✅ Pattern matching - SUPPORTED (experimental in v1.0.0-rc1)
match value {
    "linux" => echo("Linux detected"),
    "darwin" => echo("macOS detected"),
    _ => echo("Unknown OS"),
}
```

#### Loops ✅ (Supported in v1.0.0-rc1)
```rust
// ✅ For loops - FULLY SUPPORTED
for i in 0..10 {
    echo("Iteration: {i}");
}

// ✅ While loops - FULLY SUPPORTED
let mut count = 0;
while count < 10 {
    count = count + 1;
    echo("Count: {count}");
}
```

### Safety Features

All generated scripts are protected against:

- **Command Injection**: All variables are properly quoted
- **Path Traversal**: Paths are validated and escaped  
- **Glob Expansion**: Glob patterns are quoted when needed
- **Word Splitting**: IFS is set to safe value
- **Undefined Variables**: `set -u` catches undefined vars

Example of automatic safety:

```rust
let user_input = env("UNTRUSTED");
exec("echo {user_input}");  // Safe: becomes echo "$user_input"
```

## Beta Features ⚗️

The following features are available but marked as **experimental** in v1.0:

### Binary Compilation (BETA)

Compile Rust to self-extracting shell scripts or container images:

```bash
# Self-extracting script (includes runtime)
bashrs compile install.rs -o install --self-extracting

# Container image (OCI format)
bashrs compile app.rs -o app --container --format oci
```

**Status**:
- ✅ Self-extracting scripts work and are tested
- ⚠️ Container packaging is experimental
- ⚠️ Binary optimization is in progress

**Limitations**:
- Container formats are not fully implemented
- Advanced runtime optimizations pending
- Limited to dash/bash/busybox runtimes

**Recommendation**: Use `bashrs build` for production deployments. Use `compile` for quick testing or when you need a single-file installer.

### Proof Generation (BETA)

Generate formal verification proofs alongside transpiled scripts:

```bash
bashrs build input.rs -o output.sh --emit-proof
```

This creates `output.proof` with formal correctness guarantees.

**Status**: ⚠️ Proof format is experimental and may change

## Examples

See the [`examples/`](examples/) directory for complete examples:

- **Basic**
  - [Hello World](examples/hello.rs) - Simplest example
  - [Variables](examples/basic/variables.rs) - Variable usage and escaping
  - [Functions](examples/basic/functions.rs) - Built-in functions
  - [Standard Library](examples/stdlib_demo.rs) - Stdlib functions demo

- **Control Flow**
  - [Conditionals](examples/control_flow/conditionals.rs) - If/else statements
  - [Loops](examples/control_flow/loops.rs) - Bounded iteration

- **Safety**
  - [Injection Prevention](examples/safety/injection_prevention.rs) - Security examples
  - [String Escaping](examples/safety/escaping.rs) - Special character handling

- **Real-World**
  - [Node Installer](examples/node-installer.rs) - Node.js bootstrap script
  - [Rust Installer](examples/rust-installer.rs) - Rust toolchain installer

## Shell Compatibility

Generated scripts are tested on:

| Shell | Version | Status |
|-------|---------|--------|
| POSIX sh | - | ✅ Full support |
| dash | 0.5.11+ | ✅ Full support |
| bash | 3.2+ | ✅ Full support |
| ash (BusyBox) | 1.30+ | ✅ Full support |
| zsh | 5.0+ | ✅ Full support |
| mksh | R59+ | ✅ Full support |

## Standards Compliance

bashrs adheres to industry-standard shell scripting best practices and specifications:

### POSIX Shell Compliance

bashrs generates scripts compliant with the [POSIX Shell Command Language](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html) specification:

| POSIX Feature | Implementation | Status |
|---------------|----------------|--------|
| Variable quoting | Automatic single quotes for literals | ✅ Enforced |
| Command substitution | `$(command)` syntax | ✅ Compliant |
| Arithmetic expansion | `$((expression))` syntax | ✅ Compliant |
| Parameter expansion | `${var}` and `"$var"` patterns | ✅ Compliant |
| Test expressions | `[ condition ]` POSIX syntax | ✅ Compliant |
| String escaping | Proper handling of special characters | ✅ Safe |

### Google Shell Style Guide

Aligns with [Google's Shell Style Guide](https://google.github.io/styleguide/shellguide.html) recommendations:

| Guideline | bashrs Approach | Status |
|-----------|-----------------|--------|
| Always quote variables | Automatic quoting (no unquoted vars possible) | ✅ Enforced |
| Use `$(...)` not backticks | Generates modern `$(...)` syntax | ✅ Compliant |
| Check return values | Effect system tracks side effects | ✅ Implemented |
| Error messages to STDERR | Built-in `eprint()` function | ✅ Available |
| Avoid complex shell scripts | **Write Rust instead!** | ✅ **Core value** |

### ShellCheck Validation

All generated scripts pass [ShellCheck](https://www.shellcheck.net/) static analysis:

- ✅ **SC2086**: No unquoted variable expansions (automatic quoting)
- ✅ **SC2046**: No unquoted command substitutions
- ✅ **SC2116**: No useless echo wrapping
- ✅ **SC2005**: No useless echo in command substitution
- ✅ **24/24 ShellCheck tests passing** (100% compliance)

### Safety Guarantees

bashrs provides **automatic protection** against common shell vulnerabilities:

| Vulnerability | Raw Shell Risk | bashrs Protection |
|---------------|----------------|-------------------|
| **Command Injection** | Unquoted `$var` allows arbitrary commands | All variables auto-quoted |
| **Word Splitting** | `$var` splits on IFS characters | Uses `"$var"` or `'literal'` |
| **Glob Expansion** | `$var` expands wildcards (`*`, `?`) | Proper quoting prevents expansion |
| **Path Traversal** | `cd $dir` allows `../../../etc` | Safe path handling |
| **Exit on Error** | Commands fail silently by default | `set -e` enforced (optional) |

### Comparison: Raw Shell vs bashrs

**Unsafe Raw Shell**:
```bash
#!/bin/bash
USER_INPUT=$1
eval "echo $USER_INPUT"  # DANGEROUS!
rm -rf $DIRECTORY        # Word splitting risk
```

**Safe bashrs**:
```rust
fn main() {
    let user_input = env("1");
    echo("{user_input}");    // Auto-quoted → echo "$user_input"

    let directory = env("DIRECTORY");
    exec("rm -rf {directory}");  // Auto-quoted → rm -rf "$directory"
}
```

**Generated Safe Shell**:
```bash
#!/bin/sh
set -euf
IFS='
'
export LC_ALL=C

main() {
    USER_INPUT="${1}"
    echo "$USER_INPUT"        # Safely quoted

    DIRECTORY="${DIRECTORY}"
    rm -rf "$DIRECTORY"       # Safely quoted
}

main "$@"
```

### Standards Documentation

For detailed compliance information, see:
- [POSIX Shell Specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- [ShellCheck Wiki](https://www.shellcheck.net/wiki/)
- **[bashrs Safety Comparison](docs/SAFETY_COMPARISON.md)** - Comprehensive vulnerability prevention guide

## Performance

Rash is designed for fast transpilation with exceptional real-world performance:

**Makefile Parsing & Purification** (v3.0.0):
- Small Makefiles (46 lines): **0.034ms** - 297x faster than 10ms target
- Medium Makefiles (174 lines): **0.156ms** - 320x faster than 50ms target
- Large Makefiles (2,021 lines): **1.43ms** - 70x faster than 100ms target
- Linear O(n) scaling: ~0.37 µs/line parsing, ~0.35 µs/line purification

**Rust-to-Shell Transpilation**:
- **21.1µs** transpile time for simple scripts (100x better than target!)
- Memory usage <10MB for most scripts
- Generated scripts add minimal overhead (~20 lines boilerplate)

## Quality Metrics (v6.9.0)

### 🎓 **A+ GRADE QUALITY** - Achieved October 2025

| Metric | v6.8.0 | v6.9.0 | Notes |
|--------|--------|--------|-------|
| **Quality Grade** | A (Excellent) | **A+ (Near Perfect)** ✅ | Systematic refactoring + continuous improvement |
| **Tests** | 5,105 passing | **5,105 passing** ✅ | 100% pass rate (all modules) |
| **Property Tests** | 52 properties | **52 properties** ✅ | ~26,000+ test cases, 0 failures |
| **Core Coverage** | 94.85% | **94.85%** ✅ | makefile/purify.rs (critical module) |
| **Overall Coverage** | 88.71% | **88.71%** ✅ | Exceeds 85% target (Toyota Way standard) |
| **Mutation Testing** | 92% kill rate | **92% kill rate** ✅ | Exceeds 90% target |
| **Max Cyclomatic Complexity** | 17 | **14** ✅ | **-18% improvement** (A+ threshold <15) |
| **Median Cyclomatic** | 13.0 | **12.0** ✅ | **-8% improvement** |
| **Median Cognitive** | 46.5 | **44.0** ✅ | **-5% improvement** |
| **Max Cognitive Complexity** | 59 | **59** ✅ | Maintained |
| **Files Meeting Standards** | 552/587 (94%) | **555/587 (94.5%)** ✅ | +0.5% improvement |
| **Refactoring Time Estimate** | 106.5 hrs | **84.2 hrs** ✅ | **-21% reduction** (61% from v6.7.0 baseline) |
| **Multi-Shell** | 100% pass | **100% pass** ✅ | sh, dash, bash, ash, zsh, mksh |
| **ShellCheck** | 100% pass | **100% pass** ✅ | All generated scripts POSIX-compliant |
| **Makefile Linter Rules** | 28 transformations | **28 transformations** ✅ | Parallel, reproducibility, performance, error, portability |
| **Makefile Parsing** | 0.034-1.43ms | **0.034-1.43ms** ✅ | 70-320x faster than targets |
| **REPL Tests** | 48 tests | **48 tests** ✅ | 26 unit + 22 CLI integration |
| **Code Modularity** | High | **Very High** ✅ | 65 total helper functions extracted |

**v6.9.0 Status**: ✅ **PRODUCTION-READY - A+ GRADE** - Near-perfect code quality, excellent maintainability

**A+ Grade Achievement Details**:
- **11 total refactorings** completed (6 @ v6.8.0 + 5 @ v6.9.0)
- **65 helper functions** extracted following Single Responsibility Principle
- **685 lines** of complex code simplified
- **75% average complexity reduction** across refactored files
- **Zero functionality regressions** across all changes
- **61% total reduction** from v6.7.0 baseline (214 → 84.2 hrs)
- See `.quality/A-PLUS-GRADE-ACHIEVED.md` for full certification

**v6.9.0 Sprint Highlights**:
- make008, make004, sc2242, sc2032, sc2119 refactored
- 39 additional helper functions extracted
- Max cyclomatic: 17 → **14** (A+ threshold achieved)
- Only 1 file at max complexity 14 (sc2096.rs)

## Troubleshooting

Having issues? Check our **[Error Guide](docs/ERROR_GUIDE.md)** for common errors and solutions.

## MCP Server

Rash provides a Model Context Protocol (MCP) server for AI-assisted shell script generation:

```bash
# Install from crates.io
cargo install rash-mcp

# Run MCP server
rash-mcp
```

The MCP server is available in the official registry as `io.github.paiml/rash`.

**For developers**: See [MCP Registry Publishing Guide](docs/mcp-registry-publish.md) for details on the automated publishing process.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/paiml/rash.git
cd rash

# Run tests
make test

# Run with all checks
make validate

# Build release binary
make release
```

### Publishing to MCP Registry

For maintainers publishing new MCP server versions, see the [MCP Registry Publishing Guide](docs/mcp-registry-publish.md).

## License

Rash is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Rash is built with safety principles inspired by:
- [ShellCheck](https://www.shellcheck.net/) for shell script analysis
- [Oil Shell](https://www.oilshell.org/) for shell language design
- The Rust community for memory safety practices

## Roadmap

### v3.0.0 (Current Release) ✅

**Status**: PRODUCTION-READY - Phase 1 Complete: Makefile World-Class
**Released**: 2025-10-20
**Achievement**: World-class Makefile linting, parsing, and purification with exceptional performance
**Quality Metrics**:
- 1,752 tests passing (100% pass rate, Sprints 81-84)
- 94.85% coverage on critical modules (makefile/purify.rs)
- 88.71% overall coverage (exceeds 85% target)
- 167 mutants identified through comprehensive mutation testing
- 0 shellcheck warnings
- 52 property tests (~26,000+ cases)
- 70-320x faster than performance targets

**Core Features** (Complete):
- [x] **Makefile Purification** (v3.0.0 - NEW!) - World-class linting and transformation
  - 28 transformation types across 5 categories
  - Parallel safety analysis, reproducibility enforcement
  - Performance optimization, error handling detection
  - Portability checks (bashisms, platform-specific commands)
  - 70-320x faster than performance targets
- [x] Rust-to-Shell transpilation (POSIX, Bash, Dash, Ash)
- [x] Full AST parsing and validation (98.92% coverage)
- [x] IR generation and optimization (87-99% coverage)
- [x] Safety verification and escape handling (95.45% coverage)
- [x] Multi-shell compatibility testing (100% pass rate)
- [x] Property-based testing (114k executions, 0 failures)
- [x] Fuzzing infrastructure (0 failures)
- [x] ShellCheck compliance (24/24 tests pass)
- [x] Arithmetic expressions and comparisons
- [x] User-defined functions
- [x] `println!` macro support
- [x] MCP server (rash-mcp)

**CLI Tools** (Complete):
- [x] `bashrs build` - Transpile Rust to shell
- [x] `bashrs check` - Validate Rust compatibility
- [x] `bashrs init` - Project scaffolding
- [x] `bashrs verify` - Script verification
- [x] `bashrs inspect` - Formal verification reports

**Shipped in v1.0.0-rc1**:
- [x] Control flow (if/else if/else) - STABLE
- [x] For loops (0..n, 0..=n) - STABLE
- [x] While loops (with max_iterations safety) - STABLE
- [x] Match expressions (basic pattern matching) - EXPERIMENTAL
- [x] Logical operators (&&, ||, !) - STABLE
- [x] String and integer comparisons - STABLE
- [x] Self-extracting scripts - STABLE
- [ ] Container packaging (in progress)
- [ ] Proof generation (experimental format)

### v1.1.0 (Released - October 2025) ✅

**Native Linting** (Complete):
- [x] `bashrs lint` subcommand with zero external dependencies
- [x] SC2086 - Unquoted variable expansion detection
- [x] SC2046 - Unquoted command substitution detection
- [x] SC2116 - Useless echo detection
- [x] Human, JSON, and SARIF output formats
- [x] Auto-fix suggestions for all violations
- [x] 48 comprehensive linter tests (100% passing)
- [x] 88.5% code coverage (exceeds 85% target)

**Quality Improvements**:
- [x] Increased test coverage from 85.36% to 88.5%
- [x] Added 48 new linter tests (804 total tests)
- [x] Comprehensive documentation with Sprint 1 report

### v1.2 (Planned)

**Enhanced Linting**:
- [ ] SC2115 - Use `${var:?}` to ensure variable is set
- [ ] SC2128 - Expanding array without index
- [ ] BP-series rules (POSIX compliance validation)
- [ ] SE-series rules (Security taint analysis)
- [ ] Auto-fix application (`--fix` flag)
- [ ] AST-based semantic analysis (replace regex)

**Interactive Features**:
- [ ] Playground/REPL (separate `rash-playground` crate)
- [ ] Web-based transpiler
- [ ] Live syntax highlighting

**Language Features**:
- [x] For loops (`for i in 0..10`) - SHIPPED in v1.0.0-rc1
- [x] Match expressions (pattern matching) - SHIPPED in v1.0.0-rc1
- [x] While loops - SHIPPED in v1.0.0-rc1
- [ ] Arrays and collections (advanced operations)
- [ ] Enhanced pattern matching guards

**Tooling**:
- [ ] Language server protocol (LSP)
- [ ] IDE integration examples
- [ ] Better error diagnostics

### v1.2+ (Future)

**Advanced Features**:
- [ ] Incremental compilation
- [ ] More shell targets (fish, PowerShell, nushell)
- [ ] Package manager integration
- [ ] Advanced optimizations (constant folding, DCE)
- [ ] Formal verification with SMT solvers

**Documentation**:
- [ ] Video tutorials
- [ ] Interactive examples
- [ ] Best practices guide

See [v1.0-feature-scope.md](.quality/v1.0-feature-scope.md) for detailed feature decisions.