# ShellCheck-Native Subcommand Specification

**Status**: Draft
**Version**: 0.1.0
**Author**: bashrs Team
**Date**: 2025-10-10

## Abstract

This specification defines a native `bashrs lint` subcommand that provides **ShellCheck-level validation and MORE** for both:
1. **Ingested shell scripts** (Bash → Rust conversion validation)
2. **Generated shell scripts** (Rust → Shell transpilation validation)

Unlike external ShellCheck integration, this native implementation leverages bashrs's internal AST, IR, and formal verification infrastructure to provide **deeper semantic analysis** with context-aware recommendations.

---

## Table of Contents

1. [Motivation](#motivation)
2. [Goals](#goals)
3. [Architecture](#architecture)
4. [Rule Categories](#rule-categories)
5. [Command Interface](#command-interface)
6. [Implementation Strategy](#implementation-strategy)
7. [Rule Specifications](#rule-specifications)
8. [Output Formats](#output-formats)
9. [Integration with bashrs Workflow](#integration-with-bashrs-workflow)
10. [Comparison: ShellCheck vs bashrs lint](#comparison-shellcheck-vs-bashrs-lint)

---

## Motivation

### Why Native Validation?

**Problem 1: External Tool Dependency**
- Current: bashrs relies on external `shellcheck` binary for validation
- Issue: Extra installation step, version compatibility, inconsistent results

**Problem 2: Limited Context for Ingested Shell**
- Current: ShellCheck validates raw shell syntax only
- Issue: Can't provide Rust-specific recommendations during Bash → Rust conversion

**Problem 3: Missed Opportunities for Generated Shell**
- Current: Generate shell, then validate with external tool
- Issue: Can't prevent issues **before** emission; post-hoc validation only

**Solution: Native `bashrs lint` Subcommand**
- ✅ Zero external dependencies (pure Rust implementation)
- ✅ AST-level analysis (deeper than ShellCheck's token-based approach)
- ✅ Context-aware recommendations (knows Rust idioms + shell safety)
- ✅ Bidirectional validation (ingested AND generated shell)
- ✅ Formal verification integration (uses bashrs's proof engine)

---

## Goals

### Primary Goals

1. **Parity with ShellCheck**: Implement all critical ShellCheck rules (SC1000-SC3000 series)
2. **Enhanced Validation**: Add bashrs-specific rules beyond ShellCheck's scope
3. **Bidirectional Analysis**: Validate both Bash → Rust and Rust → Shell workflows
4. **Zero Dependencies**: No external tools required (pure Rust implementation)
5. **Integration**: Seamless integration with `bashrs build`, `bashrs check`, `bashrs verify`

### Secondary Goals

6. **Performance**: Faster than external ShellCheck (leverages existing AST parsing)
7. **Machine-Readable Output**: JSON, SARIF, and human-readable formats
8. **Auto-Fix**: Suggest (and optionally apply) automatic fixes
9. **Custom Rules**: Allow users to define project-specific linting rules
10. **IDE Integration**: LSP-compatible diagnostics for editor support

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    bashrs lint Subcommand                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
        ┌─────────────────────────────────────────┐
        │        Input Dispatcher                  │
        │  - Detect file type (.sh vs .rs)        │
        │  - Route to appropriate analyzer         │
        └─────────────────────────────────────────┘
                 │                        │
                 ▼                        ▼
    ┌────────────────────┐    ┌────────────────────┐
    │   Shell Analyzer   │    │   Rust Analyzer    │
    │  (Ingested Bash)   │    │  (Rash Source)     │
    └────────────────────┘    └────────────────────┘
                 │                        │
                 ▼                        ▼
    ┌────────────────────┐    ┌────────────────────┐
    │   bash_parser      │    │   syn Parser       │
    │   → BashAst        │    │   → syn::File      │
    └────────────────────┘    └────────────────────┘
                 │                        │
                 ▼                        ▼
    ┌────────────────────┐    ┌────────────────────┐
    │  ShellLintEngine   │    │  RashLintEngine    │
    │  - SC* rules       │    │  - Rust idioms     │
    │  - Safety checks   │    │  - Shell safety    │
    │  - POSIX compat    │    │  - Transpile hints │
    └────────────────────┘    └────────────────────┘
                 │                        │
                 └────────────┬───────────┘
                              ▼
                    ┌──────────────────┐
                    │  Diagnostic      │
                    │  Aggregator      │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Output Formatter│
                    │  - Human-readable│
                    │  - JSON          │
                    │  - SARIF         │
                    │  - LSP           │
                    └──────────────────┘
```

### Core Components

1. **Input Dispatcher**: Detects file type and routes to appropriate analyzer
2. **Shell Analyzer**: Parses ingested bash using `bash_parser` module
3. **Rust Analyzer**: Parses Rash source using `syn` crate
4. **ShellLintEngine**: Implements ShellCheck-equivalent rules for shell scripts
5. **RashLintEngine**: Implements Rust-specific and transpilation-aware rules
6. **Diagnostic Aggregator**: Collects, deduplicates, and prioritizes issues
7. **Output Formatter**: Formats diagnostics in various output formats

---

## Rule Categories

### Category 1: ShellCheck Parity Rules (SC-series)

Implement critical ShellCheck rules with bashrs-native analysis:

| SC Code | Rule | bashrs Implementation |
|---------|------|----------------------|
| **SC2086** | Unquoted variable expansion | AST-level detection of unquoted `BashExpr::Variable` |
| **SC2046** | Unquoted command substitution | AST-level detection of unquoted `BashExpr::CommandSubst` |
| **SC2116** | Useless echo wrapping | IR-level detection of redundant echo chains |
| **SC2115** | Use `${var:?}` to ensure set | Effect analysis for undefined variable usage |
| **SC2154** | Variable referenced but not assigned | Data-flow analysis across AST |
| **SC2155** | Declare and assign separately | AST pattern matching for local declarations |
| **SC2164** | Use `cd ... || exit` | Control-flow analysis for `cd` commands |
| **SC2181** | Check exit code directly | AST pattern for `$?` usage |

### Category 2: POSIX Compliance Rules (BP-series - "Bash Portability")

bashrs-specific rules for POSIX compliance:

| Code | Rule | Detection Method |
|------|------|-----------------|
| **BP1001** | Bash-specific syntax in POSIX mode | AST detection of `[[`, `((`, etc. |
| **BP1002** | Local keyword not POSIX | Keyword usage analysis |
| **BP1003** | Process substitution not POSIX | AST detection of `<(...)` |
| **BP1004** | Arrays not in POSIX | AST detection of array syntax |
| **BP1005** | String indexing not POSIX | Parameter expansion analysis |

### Category 3: Safety Enhancement Rules (SE-series - "Security Enhancement")

bashrs-specific security analysis beyond ShellCheck:

| Code | Rule | Analysis Type |
|------|------|---------------|
| **SE2001** | Potential command injection | Taint analysis on variable flow |
| **SE2002** | Unsafe eval usage | AST detection + data-flow analysis |
| **SE2003** | Unvalidated path traversal | Path manipulation analysis |
| **SE2004** | Missing input sanitization | Effect analysis for external inputs |
| **SE2005** | Dangerous file operations | AST detection of `rm -rf`, etc. |
| **SE2006** | Unquoted glob patterns | AST + semantic analysis |
| **SE2007** | Race condition (TOCTOU) | Temporal analysis of file operations |

### Category 4: Idempotency Rules (ID-series)

bashrs-specific idempotency validation:

| Code | Rule | Detection |
|------|------|-----------|
| **ID3001** | Non-idempotent mkdir | Detect `mkdir` without `-p` |
| **ID3002** | Non-idempotent rm | Detect `rm` without `-f` |
| **ID3003** | Timestamp dependency | Detect `date`, `$RANDOM` usage |
| **ID3004** | Process ID dependency | Detect `$$`, `$PPID` usage |
| **ID3005** | Non-deterministic commands | Detect shuffle, random, etc. |

### Category 5: Rust Transpilation Rules (RT-series - "Rash Transpilation")

Rules specific to Rash → Shell transpilation:

| Code | Rule | Context |
|------|------|---------|
| **RT4001** | Unsupported Rust feature | Parser-level detection |
| **RT4002** | Inefficient shell generation | IR optimization hints |
| **RT4003** | Missing error handling | Effect analysis |
| **RT4004** | Overly complex function | Cyclomatic complexity analysis |
| **RT4005** | Performance warning | Generated shell overhead |

---

## Command Interface

### Basic Usage

```bash
# Lint a shell script (ingested)
bashrs lint script.sh

# Lint a Rash source file
bashrs lint install.rs

# Lint with specific severity
bashrs lint --severity error script.sh

# Lint with auto-fix suggestions
bashrs lint --fix script.sh

# Lint and apply auto-fixes
bashrs lint --fix --apply script.sh

# Output in JSON format
bashrs lint --format json script.sh
```

### Command Signature

```
bashrs lint [OPTIONS] <FILE>

OPTIONS:
    -s, --severity <LEVEL>     Minimum severity to report [default: warning]
                               Values: error, warning, info, style

    -f, --format <FORMAT>      Output format [default: human]
                               Values: human, json, sarif, checkstyle

    -c, --category <CAT>       Filter by rule category
                               Values: shellcheck, posix, security, idempotency, transpile

    --fix                      Show auto-fix suggestions

    --apply                    Apply auto-fixes (requires --fix)

    --rule <CODE>              Enable/disable specific rule
                               Example: --rule SC2086=off

    --config <FILE>            Use custom lint configuration
                               Default: .bashrslint.toml

    --exclude <PATTERN>        Exclude files matching pattern

    -q, --quiet                Only show errors (no warnings)

    -v, --verbose              Show detailed explanations

    --explain <CODE>           Explain a specific rule code
                               Example: --explain SC2086
```

### Integration Commands

```bash
# Lint during build (pre-transpilation)
bashrs build install.rs --lint

# Lint after build (post-transpilation)
bashrs build install.rs --lint-output

# Lint both source and output
bashrs build install.rs --lint-all

# Verify existing shell script matches source
bashrs verify install.rs install.sh --lint
```

---

## Implementation Strategy

### Phase 1: Foundation (Sprint 1-2)

**Goal**: Basic linting infrastructure

1. **Create `lint` module** (`rash/src/lint/`)
   - `mod.rs` - Public API
   - `engine.rs` - Linting engine
   - `rules/` - Rule implementations
   - `diagnostic.rs` - Diagnostic types
   - `formatter.rs` - Output formatters

2. **Implement 10 Core ShellCheck Rules**
   - SC2086 (unquoted variables)
   - SC2046 (unquoted command substitution)
   - SC2116 (useless echo)
   - SC2154 (variable not assigned)
   - SC2164 (cd without error check)
   - SC2181 (check exit code directly)
   - SC2115 (use ${var:?})
   - SC2155 (declare/assign separately)
   - SC1091 (sourced file not found)
   - SC2001 (use parameter expansion)

3. **CLI Integration**
   - Add `lint` subcommand to `rash/src/cli/mod.rs`
   - Argument parsing with `clap`
   - File type detection

4. **Basic Output Formatters**
   - Human-readable (colored terminal output)
   - JSON (machine-readable)

**Deliverable**: `bashrs lint script.sh` works with 10 rules

---

### Phase 2: Shell Analysis (Sprint 3-4)

**Goal**: Complete shell script validation

5. **Implement Remaining ShellCheck Rules**
   - Add 20+ additional SC-series rules
   - Focus on high-severity issues

6. **POSIX Compliance Rules (BP-series)**
   - Detect bash-isms in POSIX mode
   - Suggest portable alternatives

7. **Shell Analyzer Enhancement**
   - Integrate with `bash_parser` module
   - AST traversal for rule checking
   - Data-flow analysis for variables

8. **Configuration System**
   - `.bashrslint.toml` configuration file
   - Rule enable/disable
   - Severity overrides
   - Custom ignore patterns

**Deliverable**: Comprehensive shell script validation

---

### Phase 3: Rust/Rash Analysis (Sprint 5-6)

**Goal**: Validate Rash source files

9. **Rash Linter Implementation**
   - Parse Rash source with `syn`
   - Implement RT-series rules
   - Detect unsupported Rust features
   - Suggest idiomatic Rash patterns

10. **Transpilation-Aware Analysis**
    - Pre-transpilation validation
    - Generated shell prediction
    - Performance warnings
    - Optimization hints

11. **Effect System Integration**
    - Leverage existing effect analysis
    - Detect missing error handling
    - Validate side-effect declarations

**Deliverable**: `bashrs lint install.rs` validates Rash source

---

### Phase 4: Advanced Features (Sprint 7-8)

**Goal**: Auto-fix, LSP, and advanced analysis

12. **Auto-Fix System**
    - Suggest code fixes for common issues
    - Apply fixes automatically (with `--apply`)
    - Multi-fix orchestration

13. **Security Analysis (SE-series)**
    - Taint analysis for command injection
    - Path traversal detection
    - TOCTOU race condition detection

14. **Idempotency Validation (ID-series)**
    - Detect non-idempotent operations
    - Suggest idempotent alternatives

15. **LSP Integration**
    - Diagnostics in LSP format
    - Real-time linting in editors
    - Fix suggestions via code actions

**Deliverable**: Production-ready linting with auto-fix

---

### Phase 5: Output & Integration (Sprint 9-10)

**Goal**: Complete output formats and workflow integration

16. **Additional Output Formats**
    - SARIF (Static Analysis Results Interchange Format)
    - Checkstyle XML
    - GitHub Actions annotations
    - JUnit XML (for CI integration)

17. **Workflow Integration**
    - `--lint` flag for `build`, `check`, `verify`
    - Pre-commit hook support
    - CI/CD pipeline integration examples

18. **Custom Rules API**
    - Plugin system for user-defined rules
    - Rule authoring documentation
    - Example custom rules

**Deliverable**: Complete linting system with ecosystem integration

---

## Rule Specifications

### Example Rule: SC2086 - Unquoted Variable Expansion

**Detection**:
```rust
// In rash/src/lint/rules/sc2086.rs
pub fn check_sc2086(ast: &BashAst) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for stmt in &ast.statements {
        match stmt {
            BashStmt::Command { args, span, .. } => {
                for arg in args {
                    if let BashExpr::Variable(name) = arg {
                        // Unquoted variable in command argument!
                        diagnostics.push(Diagnostic {
                            code: "SC2086".to_string(),
                            severity: Severity::Warning,
                            message: format!(
                                "Double quote to prevent globbing and word splitting: \"${}\"",
                                name
                            ),
                            span: *span,
                            fix: Some(Fix {
                                description: format!("Quote ${}", name),
                                replacement: format!("\"${}\"", name),
                            }),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}
```

**Output**:
```
script.sh:5:8: warning[SC2086]: Double quote to prevent globbing and word splitting
  |
5 | rm -rf $DIR
  |        ^^^^ unquoted variable
  |
  = help: Use "rm -rf \"$DIR\"" instead
  = note: For more information, run: bashrs lint --explain SC2086
```

---

### Example Rule: RT4001 - Unsupported Rust Feature

**Detection**:
```rust
// In rash/src/lint/rules/rt4001.rs
pub fn check_rt4001(file: &syn::File) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for item in &file.items {
        match item {
            syn::Item::Struct(_) => {
                diagnostics.push(Diagnostic {
                    code: "RT4001".to_string(),
                    severity: Severity::Error,
                    message: "Structs are not supported in Rash v1.0".to_string(),
                    span: item.span(),
                    fix: None, // No auto-fix for unsupported features
                    note: Some(
                        "Rash currently supports functions and basic types only. \
                         Consider refactoring to use function parameters.".to_string()
                    ),
                });
            }
            syn::Item::Trait(_) => {
                diagnostics.push(Diagnostic {
                    code: "RT4001".to_string(),
                    severity: Severity::Error,
                    message: "Traits are not supported in Rash".to_string(),
                    span: item.span(),
                    fix: None,
                });
            }
            _ => {}
        }
    }

    diagnostics
}
```

**Output**:
```
install.rs:10:1: error[RT4001]: Structs are not supported in Rash v1.0
   |
10 | struct Config {
   | ^^^^^^ unsupported Rust feature
   |
   = note: Rash currently supports functions and basic types only.
           Consider refactoring to use function parameters.
```

---

### Example Rule: SE2001 - Potential Command Injection

**Detection**:
```rust
// In rash/src/lint/rules/se2001.rs
pub fn check_se2001(ast: &BashAst) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut tainted_vars = HashSet::new();

    // Phase 1: Identify tainted sources (external input)
    for stmt in &ast.statements {
        if let BashStmt::Assignment { name, value, .. } = stmt {
            if is_external_input(value) {
                tainted_vars.insert(name.clone());
            }
        }
    }

    // Phase 2: Check for tainted variable usage in dangerous contexts
    for stmt in &ast.statements {
        match stmt {
            BashStmt::Command { name, args, span } if name == "eval" => {
                for arg in args {
                    if contains_tainted_var(arg, &tainted_vars) {
                        diagnostics.push(Diagnostic {
                            code: "SE2001".to_string(),
                            severity: Severity::Error,
                            message: "Potential command injection via eval".to_string(),
                            span: *span,
                            fix: None,
                            note: Some(
                                "User input is passed to eval, allowing arbitrary code execution. \
                                 Avoid eval or sanitize input rigorously.".to_string()
                            ),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}

fn is_external_input(expr: &BashExpr) -> bool {
    matches!(expr,
        BashExpr::Variable(name) if name.starts_with('$') ||
        BashExpr::CommandSubst(_)
    )
}

fn contains_tainted_var(expr: &BashExpr, tainted: &HashSet<String>) -> bool {
    match expr {
        BashExpr::Variable(name) => tainted.contains(name),
        BashExpr::Concat(parts) => parts.iter().any(|p| contains_tainted_var(p, tainted)),
        _ => false,
    }
}
```

**Output**:
```
script.sh:12:1: error[SE2001]: Potential command injection via eval
   |
12 | eval "$USER_INPUT"
   | ^^^^^^^^^^^^^^^^^^ dangerous eval with user input
   |
   = note: User input is passed to eval, allowing arbitrary code execution.
           Avoid eval or sanitize input rigorously.
```

---

## Output Formats

### Human-Readable Format (Default)

```
script.sh:5:8: warning[SC2086]: Double quote to prevent globbing and word splitting
  |
5 | rm -rf $DIR
  |        ^^^^ unquoted variable
  |
  = help: Use "rm -rf \"$DIR\"" instead
  = note: For more information, run: bashrs lint --explain SC2086

script.sh:12:1: error[SE2001]: Potential command injection via eval
   |
12 | eval "$USER_INPUT"
   | ^^^^^^^^^^^^^^^^^^ dangerous eval with user input
   |
   = note: User input is passed to eval, allowing arbitrary code execution.
           Avoid eval or sanitize input rigorously.

Summary: 1 error, 1 warning, 0 info (2 issues total)
```

### JSON Format

```json
{
  "version": "1.0.0",
  "file": "script.sh",
  "diagnostics": [
    {
      "code": "SC2086",
      "severity": "warning",
      "message": "Double quote to prevent globbing and word splitting",
      "location": {
        "file": "script.sh",
        "line": 5,
        "column": 8,
        "length": 4
      },
      "fix": {
        "description": "Quote $DIR",
        "replacement": "\"$DIR\""
      },
      "help": "Use \"rm -rf \\\"$DIR\\\"\" instead",
      "url": "https://www.shellcheck.net/wiki/SC2086"
    },
    {
      "code": "SE2001",
      "severity": "error",
      "message": "Potential command injection via eval",
      "location": {
        "file": "script.sh",
        "line": 12,
        "column": 1,
        "length": 18
      },
      "note": "User input is passed to eval, allowing arbitrary code execution."
    }
  ],
  "summary": {
    "errors": 1,
    "warnings": 1,
    "info": 0,
    "total": 2
  }
}
```

### SARIF Format (Static Analysis Results Interchange Format)

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "bashrs lint",
          "version": "1.0.0",
          "informationUri": "https://github.com/paiml/bashrs",
          "rules": [
            {
              "id": "SC2086",
              "shortDescription": {
                "text": "Double quote to prevent globbing and word splitting"
              },
              "helpUri": "https://www.shellcheck.net/wiki/SC2086"
            }
          ]
        }
      },
      "results": [
        {
          "ruleId": "SC2086",
          "level": "warning",
          "message": {
            "text": "Double quote to prevent globbing and word splitting: \"$DIR\""
          },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "script.sh"
                },
                "region": {
                  "startLine": 5,
                  "startColumn": 8,
                  "endColumn": 12
                }
              }
            }
          ],
          "fixes": [
            {
              "description": {
                "text": "Quote $DIR"
              },
              "artifactChanges": [
                {
                  "artifactLocation": {
                    "uri": "script.sh"
                  },
                  "replacements": [
                    {
                      "deletedRegion": {
                        "startLine": 5,
                        "startColumn": 8,
                        "endColumn": 12
                      },
                      "insertedContent": {
                        "text": "\"$DIR\""
                      }
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

---

## Integration with bashrs Workflow

### Pre-Transpilation Linting

```bash
# Lint Rash source before transpilation
bashrs build install.rs --lint

# Workflow:
# 1. Parse install.rs with syn
# 2. Run RashLintEngine (RT-series rules)
# 3. If errors: STOP, report issues
# 4. If warnings: Continue with warnings displayed
# 5. Transpile to shell
```

### Post-Transpilation Linting

```bash
# Lint generated shell after transpilation
bashrs build install.rs --lint-output -o install.sh

# Workflow:
# 1. Transpile install.rs → install.sh
# 2. Parse install.sh with bash_parser
# 3. Run ShellLintEngine (SC-series + BP-series rules)
# 4. Report any issues in generated code
```

### Bidirectional Linting

```bash
# Lint both source and output
bashrs build install.rs --lint-all -o install.sh

# Workflow:
# 1. Lint install.rs (pre-transpilation)
# 2. Transpile to install.sh
# 3. Lint install.sh (post-transpilation)
# 4. Compare issues, report comprehensive diagnostics
```

### Verification with Linting

```bash
# Verify existing shell matches source, with linting
bashrs verify install.rs install.sh --lint

# Workflow:
# 1. Transpile install.rs → expected.sh (in memory)
# 2. Lint expected.sh
# 3. Lint install.sh (actual)
# 4. Compare expected vs actual
# 5. Report differences + lint issues
```

---

## Comparison: ShellCheck vs bashrs lint

| Feature | ShellCheck | bashrs lint | Advantage |
|---------|-----------|-------------|-----------|
| **Installation** | External binary | Built-in | bashrs ✅ |
| **Shell Script Validation** | ✅ Excellent | ✅ Parity + More | bashrs ✅ |
| **Rash Source Validation** | ❌ No | ✅ Yes | bashrs ✅ |
| **AST-Level Analysis** | Token-based | Full AST | bashrs ✅ |
| **Taint Analysis** | ❌ No | ✅ Yes (SE-series) | bashrs ✅ |
| **Idempotency Validation** | ❌ No | ✅ Yes (ID-series) | bashrs ✅ |
| **POSIX Compliance** | ✅ Yes | ✅ Yes (BP-series) | Tie |
| **Auto-Fix** | ❌ No | ✅ Yes | bashrs ✅ |
| **JSON Output** | ✅ Yes | ✅ Yes | Tie |
| **SARIF Output** | ❌ No | ✅ Yes | bashrs ✅ |
| **LSP Integration** | Via wrapper | Native | bashrs ✅ |
| **Custom Rules** | Via config | Plugin API | bashrs ✅ |
| **Effect Analysis** | ❌ No | ✅ Yes | bashrs ✅ |
| **Formal Verification** | ❌ No | ✅ Integrates | bashrs ✅ |
| **Pre-Transpilation Lint** | ❌ N/A | ✅ Yes | bashrs ✅ |
| **Bidirectional Validation** | ❌ No | ✅ Yes | bashrs ✅ |

**Summary**: bashrs lint provides **ShellCheck parity + 10 unique advantages**

---

## Configuration File Format

### `.bashrslint.toml`

```toml
# bashrs Lint Configuration
# Version: 1.0.0

[general]
# Minimum severity to report (error, warning, info, style)
min_severity = "warning"

# Output format (human, json, sarif, checkstyle)
output_format = "human"

# Enable colored output (true/false)
color = true

[rules]
# Enable/disable specific rules
# Format: "RULE_CODE" = "on" | "off" | "error" | "warning" | "info"

# ShellCheck rules (SC-series)
SC2086 = "warning"  # Unquoted variables
SC2046 = "warning"  # Unquoted command substitution
SC2116 = "info"     # Useless echo
SC2154 = "error"    # Variable not assigned

# POSIX compliance rules (BP-series)
BP1001 = "warning"  # Bash-specific syntax
BP1002 = "warning"  # Local keyword not POSIX
BP1003 = "warning"  # Process substitution

# Security rules (SE-series)
SE2001 = "error"    # Command injection
SE2002 = "error"    # Unsafe eval
SE2003 = "warning"  # Path traversal
SE2004 = "warning"  # Missing input sanitization

# Idempotency rules (ID-series)
ID3001 = "warning"  # Non-idempotent mkdir
ID3002 = "warning"  # Non-idempotent rm
ID3003 = "error"    # Timestamp dependency

# Transpilation rules (RT-series)
RT4001 = "error"    # Unsupported Rust feature
RT4002 = "info"     # Inefficient generation
RT4003 = "warning"  # Missing error handling

[exclusions]
# Patterns to exclude from linting
files = [
    "tests/**/*.sh",
    "examples/legacy/*.sh",
]

# Directories to ignore
directories = [
    "node_modules",
    "target",
    ".git",
]

[auto_fix]
# Enable auto-fix suggestions
suggest_fixes = true

# Automatically apply safe fixes (use with caution!)
auto_apply = false

# Maximum number of fixes to apply in one pass
max_fixes_per_pass = 50

[output]
# Include explanations for each diagnostic
include_explanations = true

# Show context lines around issues
context_lines = 2

# Include fix suggestions in output
show_fixes = true

# URL template for rule documentation
rule_url_template = "https://github.com/paiml/bashrs/wiki/{code}"
```

---

## Benefits Over External ShellCheck

### 1. **Zero Installation Friction**
- **ShellCheck**: Requires separate installation (`apt install shellcheck`, `brew install shellcheck`)
- **bashrs lint**: Built-in, works immediately after `cargo install bashrs`

### 2. **Deeper Semantic Analysis**
- **ShellCheck**: Token-based parsing (limited semantic understanding)
- **bashrs lint**: Full AST with data-flow analysis, effect system integration

### 3. **Bidirectional Validation**
- **ShellCheck**: Validates shell scripts only
- **bashrs lint**: Validates both Rash source (Rust) AND generated shell

### 4. **Context-Aware Recommendations**
- **ShellCheck**: Generic shell script advice
- **bashrs lint**: Knows Rash idioms, suggests Rust-specific fixes

### 5. **Formal Verification Integration**
- **ShellCheck**: Static analysis only
- **bashrs lint**: Integrates with bashrs formal verification engine (proofs, SMT)

### 6. **Auto-Fix Capability**
- **ShellCheck**: Identifies issues only
- **bashrs lint**: Suggests AND applies fixes automatically

### 7. **Security-Focused Rules**
- **ShellCheck**: General safety (quoting, etc.)
- **bashrs lint**: Taint analysis, command injection detection, TOCTOU validation

### 8. **Idempotency Validation**
- **ShellCheck**: No idempotency checks
- **bashrs lint**: Dedicated ID-series rules for deployment scripts

### 9. **Performance**
- **ShellCheck**: External process invocation overhead
- **bashrs lint**: Reuses existing AST parsing (faster)

### 10. **Extensibility**
- **ShellCheck**: Limited configuration
- **bashrs lint**: Plugin API for custom project-specific rules

---

## Success Criteria

### Minimum Viable Product (MVP)

**Phase 1 Complete When:**
- ✅ `bashrs lint script.sh` works for shell scripts
- ✅ Implements 10 core ShellCheck rules (SC2086, SC2046, etc.)
- ✅ Human-readable and JSON output formats
- ✅ Zero external dependencies
- ✅ 100% test coverage for implemented rules

### Feature Complete

**Phase 5 Complete When:**
- ✅ Parity with 50+ ShellCheck rules
- ✅ `bashrs lint install.rs` works for Rash source
- ✅ All rule categories implemented (SC, BP, SE, ID, RT)
- ✅ Auto-fix system operational
- ✅ SARIF, Checkstyle, LSP output formats
- ✅ Workflow integration (`--lint`, `--lint-all` flags)
- ✅ Plugin API for custom rules
- ✅ Comprehensive documentation
- ✅ 95%+ test coverage

---

## Future Enhancements (Post-v1.0)

### LSP Server Integration

Provide real-time linting in editors:
```rust
// rash-lsp crate
pub struct RashLanguageServer {
    linter: LintEngine,
}

impl LanguageServer for RashLanguageServer {
    fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let diagnostics = self.linter.lint(&params.text_document.text);
        self.publish_diagnostics(diagnostics);
    }

    fn code_action(&mut self, params: CodeActionParams) -> Vec<CodeAction> {
        self.linter.get_fixes(&params.range)
    }
}
```

### Machine Learning-Assisted Rules

Use ML to detect anti-patterns:
```rust
// ML model trained on high-quality bashrs examples
pub struct MLLinter {
    model: TensorFlowModel,
}

impl MLLinter {
    pub fn suggest_improvements(&self, ast: &BashAst) -> Vec<Suggestion> {
        let features = self.extract_features(ast);
        self.model.predict(features)
    }
}
```

### Web-Based Linter

Online validation tool:
```
https://bashrs.dev/lint
- Paste shell script
- Get instant feedback
- Download auto-fixed version
```

---

## Conclusion

The **`bashrs lint` subcommand** provides:

1. ✅ **ShellCheck parity** (all critical SC-series rules)
2. ✅ **Enhanced validation** (SE, ID, RT, BP rule series)
3. ✅ **Bidirectional analysis** (Bash → Rust AND Rust → Shell)
4. ✅ **Zero dependencies** (pure Rust implementation)
5. ✅ **Auto-fix capability** (suggest and apply fixes)
6. ✅ **Modern output formats** (JSON, SARIF, LSP)
7. ✅ **Workflow integration** (seamless with `build`, `verify`, `check`)

**Key Innovation**: bashrs lint **validates shell scripts AND validates Rust code that generates shell scripts**, providing **end-to-end safety guarantees** that external tools cannot match.

---

## References

- [ShellCheck Wiki](https://www.shellcheck.net/wiki/)
- [POSIX Shell Specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [SARIF Specification](https://github.com/oasis-tcs/sarif-spec)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [bashrs Formal Verification](../FORMAL_VERIFICATION.md)
- [bashrs Effect System](../EFFECT_SYSTEM.md)

---

**Status**: Draft Specification
**Next Steps**: Review, refine, and begin Phase 1 implementation
**Estimated Timeline**: 10 sprints (~20-30 hours)
