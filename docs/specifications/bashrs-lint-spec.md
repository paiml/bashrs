# bashrs lint - Shell Script Linter Specification

## Overview

**bashrs lint** is a blazing-fast shell script linter that combines the best features of shellcheck, ruff, and deno lint. Written in Rust, it provides comprehensive static analysis for both "raw" (unpurified) and "purified" bash/shell scripts with zero-configuration defaults and powerful auto-fix capabilities.

### Design Principles

1. **Speed First** (inspired by ruff): 10-100x faster than traditional shell linters through Rust implementation and built-in caching
2. **Comprehensive Analysis** (inspired by shellcheck): Deep semantic understanding of shell script behavior across multiple dialects
3. **Zero Configuration** (inspired by deno lint): Sensible defaults with opinionated rules that work out-of-the-box
4. **Auto-Fix Everything** (inspired by ruff + deno): Automatic correction for fixable issues via `--fix` flag
5. **Purification-Aware**: Unique capability to lint both raw and purified scripts with context-specific rules

---

## Key Features

### 1. Blazing Fast Performance (Ruff-inspired)

- **Built-in Rust**: Compiled performance, no interpreter overhead
- **Single-pass analysis**: All rules evaluated in one traversal of AST
- **Built-in caching**: Avoid re-analyzing unchanged files
- **Parallel processing**: Lint multiple files concurrently

**Target Performance**:
- < 10ms for small scripts (< 100 lines)
- < 100ms for medium scripts (< 1000 lines)
- < 1s for large scripts (< 10,000 lines)

### 2. Comprehensive Rule System (ShellCheck-inspired)

**Rule Categories**:
- **Quoting (Q)**: Unquoted variables, glob expansion, word splitting
- **Conditionals (C)**: Test expression issues, operator misuse
- **Commands (CMD)**: Command misuse, argument errors
- **Style (S)**: Best practices, readability improvements
- **Portability (P)**: POSIX compliance, shell dialect compatibility
- **Security (SEC)**: Injection vectors, unsafe patterns
- **Determinism (DET)**: Non-deterministic code patterns (bashrs-specific)
- **Idempotency (IDEM)**: Non-idempotent operations (bashrs-specific)

**800+ Built-in Rules** covering:
- Syntax errors and beginner mistakes
- Semantic issues causing unexpected behavior
- Robustness concerns (error handling, edge cases)
- Performance anti-patterns
- Security vulnerabilities

### 3. Zero-Configuration Defaults (Deno-inspired)

**Out-of-the-box behavior**:
- Lints all `.sh`, `.bash` files in current directory
- Enables "recommended" rule set by default
- Detects shell dialect automatically (shebang inspection)
- Reports errors in human-readable format with colors

**No configuration files required** - just run `bashrs lint` and get results.

### 4. Auto-Fix Capabilities (Ruff + Deno-inspired)

**Automatic fixes** for 70%+ of detected issues:

```bash
# Preview fixes without applying
bashrs lint --fix --dry-run script.sh

# Apply fixes automatically
bashrs lint --fix script.sh

# Fix all scripts in directory
bashrs lint --fix .
```

**Fixable issues include**:
- Adding missing quotes to variables
- Converting non-POSIX syntax to POSIX equivalents
- Replacing deprecated commands with modern alternatives
- Adding idempotent flags (-p, -f, etc.)
- Wrapping $(wildcard) with $(sort) for determinism

### 5. Purification-Aware Linting (bashrs-specific)

**Dual Mode Operation**:

1. **Raw Mode** (default): Lint unpurified scripts
   - Detect all traditional shellcheck issues
   - Flag non-deterministic patterns ($RANDOM, timestamps)
   - Flag non-idempotent operations
   - Suggest purification opportunities

2. **Purified Mode** (`--purified`): Lint purified scripts
   - Verify purification completeness
   - Ensure no non-deterministic code remains
   - Validate idempotent operations
   - Check POSIX compliance

**Example**:
```bash
# Lint raw bash script
bashrs lint deploy.sh
# → Warns about $RANDOM, timestamps, non-idempotent mkdir

# Lint purified output
bashrs lint --purified deploy-purified.sh
# → Verifies determinism, idempotency, POSIX compliance
```

---

## Architecture

### Leveraging Existing bashrs Infrastructure

**bashrs lint** reuses existing bashrs components:

1. **Parser** (`rash/src/make_parser/parser.rs`):
   - Already parses shell scripts to AST
   - Supports both bash and POSIX sh
   - Handles Makefile shell recipes

2. **AST** (`rash/src/make_parser/ast.rs`):
   - Structured representation of shell code
   - Enables semantic analysis
   - Supports visitor pattern for rule evaluation

3. **Semantic Analysis** (`rash/src/make_parser/semantic.rs`):
   - `detect_wildcard()`: Find non-deterministic glob patterns
   - `detect_shell_date()`: Find timestamp usage
   - `detect_random()`: Find $RANDOM usage
   - Already implements core purification checks

4. **Purification Logic**:
   - Transformation rules for deterministic conversion
   - Idempotent operation detection
   - POSIX compliance verification

### New Components

**lint/mod.rs** - Main linter module
- Rule registry and execution engine
- AST visitor for rule evaluation
- Fix application logic

**lint/rules/** - Rule implementations
- `quoting.rs`: Q001-Q099 (quoting issues)
- `conditionals.rs`: C001-C099 (conditional logic)
- `commands.rs`: CMD001-CMD099 (command usage)
- `style.rs`: S001-S099 (style improvements)
- `portability.rs`: P001-P099 (POSIX compliance)
- `security.rs`: SEC001-SEC099 (security vulnerabilities)
- `determinism.rs`: DET001-DET099 (non-deterministic patterns)
- `idempotency.rs`: IDEM001-IDEM099 (non-idempotent operations)

**lint/config.rs** - Configuration system
- Load `.bashrs.toml` or `bashrs.toml`
- Hierarchical configuration (monorepo-friendly)
- Rule selection (tags, include, exclude)

**lint/fix.rs** - Auto-fix engine
- Safe fix application
- Dry-run mode
- Fix conflict resolution

---

## Rule Examples

### Quoting Rules (Q)

**Q001: Unquoted Variable Expansion**
```bash
# ❌ BAD
rm -rf $BUILD_DIR/*.o

# ✅ GOOD (auto-fixable)
rm -rf "${BUILD_DIR}"/*.o
```

**Q002: Unquoted Command Substitution**
```bash
# ❌ BAD
FILES=$(ls *.txt)

# ✅ GOOD (auto-fixable)
FILES="$(ls *.txt)"
```

### Determinism Rules (DET) - bashrs-specific

**DET001: $RANDOM Usage**
```bash
# ❌ BAD
SESSION_ID=$RANDOM

# ✅ GOOD (auto-fixable with version argument)
SESSION_ID="session-${VERSION}"
```

**DET002: Timestamp Usage**
```bash
# ❌ BAD
RELEASE="release-$(date +%s)"

# ✅ GOOD (auto-fixable)
RELEASE="release-${VERSION}"
```

**DET003: Unordered Wildcard**
```bash
# ❌ BAD
FILES=$(wildcard *.c)

# ✅ GOOD (auto-fixable)
FILES=$(sort $(wildcard *.c))
```

### Idempotency Rules (IDEM) - bashrs-specific

**IDEM001: Non-idempotent mkdir**
```bash
# ❌ BAD
mkdir /app/releases

# ✅ GOOD (auto-fixable)
mkdir -p /app/releases
```

**IDEM002: Non-idempotent rm**
```bash
# ❌ BAD
rm /app/current

# ✅ GOOD (auto-fixable)
rm -f /app/current
```

**IDEM003: Non-idempotent ln**
```bash
# ❌ BAD
ln -s /app/releases/v1.0 /app/current

# ✅ GOOD (auto-fixable)
rm -f /app/current && ln -s /app/releases/v1.0 /app/current
```

### Security Rules (SEC)

**SEC001: Command Injection Risk**
```bash
# ❌ BAD
eval "rm -rf $USER_INPUT"

# ✅ GOOD (not auto-fixable, requires manual review)
# Use array and proper quoting instead of eval
```

**SEC002: Unquoted Variable in Command**
```bash
# ❌ BAD
curl $URL

# ✅ GOOD (auto-fixable)
curl "${URL}"
```

### Portability Rules (P)

**P001: Bash-specific Array Syntax**
```bash
# ❌ BAD
arr=(foo bar baz)

# ✅ GOOD (auto-fixable for simple cases)
set -- foo bar baz
```

**P002: Non-POSIX [[ ]] Syntax**
```bash
# ❌ BAD
if [[ -f "$file" ]]; then

# ✅ GOOD (auto-fixable)
if [ -f "$file" ]; then
```

---

## Configuration

### Configuration File Format

**bashrs.toml** (or `.bashrs.toml`):

```toml
[lint]
# Files to include (glob patterns)
include = ["src/**/*.sh", "scripts/*.bash"]

# Files to exclude (glob patterns)
exclude = ["tests/fixtures/**", "vendor/**"]

# Rule configuration
[lint.rules]
# Enable rule sets by tag
tags = ["recommended", "security", "determinism"]

# Include specific rules
include = ["DET001", "DET002", "IDEM001"]

# Exclude specific rules
exclude = ["S042"]  # Allow TODO comments

# Shell dialect (auto-detected if not specified)
shell = "sh"  # Options: sh, bash, dash, ash

# Purified mode (default: false)
purified = false

# Auto-fix mode (default: false)
fix = false

# Output format
format = "pretty"  # Options: pretty, json, github, checkstyle
```

### Hierarchical Configuration (Monorepo Support)

**Project structure**:
```
monorepo/
├── bashrs.toml          # Root config
├── service-a/
│   ├── bashrs.toml      # Override for service-a
│   └── scripts/
│       └── deploy.sh
└── service-b/
    └── scripts/
        └── build.sh
```

Configuration cascades from root → child directories.

### Zero Configuration Defaults

If no configuration file exists, bashrs lint uses these defaults:

```toml
[lint]
include = ["**/*.sh", "**/*.bash"]
exclude = ["node_modules/**", "vendor/**", ".git/**"]

[lint.rules]
tags = ["recommended"]
shell = "auto"  # Detect from shebang
purified = false
fix = false
format = "pretty"
```

---

## CLI Interface

### Basic Usage

```bash
# Lint current directory (zero-config)
bashrs lint

# Lint specific file
bashrs lint script.sh

# Lint multiple files
bashrs lint deploy.sh build.sh test.sh

# Lint directory recursively
bashrs lint scripts/

# Lint with auto-fix
bashrs lint --fix script.sh

# Lint purified scripts
bashrs lint --purified purified-output.sh

# Preview fixes without applying
bashrs lint --fix --dry-run script.sh
```

### Advanced Options

```bash
# Specify shell dialect
bashrs lint --shell sh script.sh

# Enable/disable rule tags
bashrs lint --tag recommended --tag security script.sh

# Include specific rules
bashrs lint --rule DET001 --rule IDEM001 script.sh

# Exclude specific rules
bashrs lint --no-rule S042 script.sh

# Output formats
bashrs lint --format json script.sh
bashrs lint --format github script.sh  # GitHub Actions annotations
bashrs lint --format checkstyle script.sh

# Show all available rules
bashrs lint --list-rules

# Explain a specific rule
bashrs lint --explain DET001
```

### CI/CD Integration

**GitHub Actions**:
```yaml
- name: Lint shell scripts
  run: bashrs lint --format github
```

**GitLab CI**:
```yaml
lint:
  script:
    - bashrs lint --format checkstyle > report.xml
  artifacts:
    reports:
      junit: report.xml
```

**Pre-commit Hook**:
```bash
# .git/hooks/pre-commit
bashrs lint --fix $(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(sh|bash)$')
```

---

## Output Formats

### Pretty (Default)

```
deploy.sh
  DET001 [Error] Non-deterministic $RANDOM usage
    5 │ SESSION_ID=$RANDOM
      │            ^^^^^^^
      │ Replace with deterministic identifier
      │
      │ Fix: SESSION_ID="session-${VERSION}"

  IDEM001 [Warning] Non-idempotent mkdir
    8 │ mkdir /app/releases
      │ ^^^^^
      │ Add -p flag for idempotent operation
      │
      │ Fix: mkdir -p /app/releases

Found 2 errors, 0 warnings in 1 file (10ms)
Run with --fix to apply automatic fixes
```

### JSON

```json
{
  "files": [
    {
      "path": "deploy.sh",
      "diagnostics": [
        {
          "rule": "DET001",
          "severity": "error",
          "message": "Non-deterministic $RANDOM usage",
          "line": 5,
          "column": 12,
          "fix": {
            "applicable": true,
            "suggestion": "SESSION_ID=\"session-${VERSION}\""
          }
        }
      ]
    }
  ],
  "summary": {
    "errors": 2,
    "warnings": 0,
    "files": 1,
    "duration_ms": 10
  }
}
```

### GitHub Actions

```
::error file=deploy.sh,line=5,col=12::DET001: Non-deterministic $RANDOM usage
::warning file=deploy.sh,line=8,col=1::IDEM001: Non-idempotent mkdir
```

---

## Rule Reference

### Rule Severity Levels

- **Error**: Code will cause incorrect behavior or security issues
- **Warning**: Code may cause issues in certain conditions
- **Info**: Code style improvements and best practices

### Rule Tags

- `recommended`: Essential rules enabled by default (150+ rules)
- `security`: Security vulnerability detection (50+ rules)
- `determinism`: Non-deterministic pattern detection (20+ rules)
- `idempotency`: Non-idempotent operation detection (15+ rules)
- `portability`: POSIX compliance checks (100+ rules)
- `style`: Code style and readability (200+ rules)
- `performance`: Performance anti-patterns (30+ rules)
- `all`: All available rules (800+ rules)

### Core Rule Categories

**Quoting (Q001-Q099)**: 25 rules
- Unquoted variables, command substitutions, globs
- Word splitting issues
- Tilde expansion problems

**Conditionals (C001-C099)**: 30 rules
- Test expression issues
- Operator misuse
- Constant conditions

**Commands (CMD001-CMD099)**: 50 rules
- Command argument errors
- Deprecated command usage
- Command misapplication

**Style (S001-S099)**: 100 rules
- Readability improvements
- Best practices
- Naming conventions

**Portability (P001-P099)**: 80 rules
- POSIX compliance
- Shell dialect compatibility
- Platform-specific issues

**Security (SEC001-SEC099)**: 45 rules
- Injection vulnerabilities
- Unsafe command usage
- Privilege escalation risks

**Determinism (DET001-DET099)**: 20 rules (bashrs-specific)
- $RANDOM usage
- Timestamp generation
- Unordered wildcards
- Process IDs ($$)
- Hostname dependencies

**Idempotency (IDEM001-IDEM099)**: 15 rules (bashrs-specific)
- Non-idempotent mkdir/rm/ln
- Append vs overwrite operations
- State dependencies

---

## Performance Benchmarks

### Target Performance (Ruff-inspired)

**Single File**:
- Small (< 100 lines): < 10ms
- Medium (< 1000 lines): < 100ms
- Large (< 10,000 lines): < 1s

**Directory Linting**:
- 100 files: < 1s
- 1,000 files: < 10s
- 10,000 files: < 100s

### Optimization Techniques

1. **Built-in Caching**:
   - Cache file hashes to skip unchanged files
   - Cache AST for files that haven't changed
   - Persistent cache across runs

2. **Parallel Processing**:
   - Lint multiple files concurrently
   - Utilize all CPU cores
   - Work-stealing task scheduler

3. **Single-Pass Analysis**:
   - All rules evaluated in one AST traversal
   - No separate passes for different rule categories
   - Minimal memory allocations

4. **Incremental Linting**:
   - Only lint changed files in git repos
   - `bashrs lint --changed` for CI optimization

---

## Implementation Roadmap

### Phase 1: Core Infrastructure (4-6 weeks)

**Week 1-2: Basic Linting Framework**
- [ ] Create `rash/src/lint/mod.rs` module structure
- [ ] Implement rule registry system
- [ ] Create AST visitor for rule evaluation
- [ ] Add CLI interface: `bashrs lint <file>`
- [ ] Implement basic output formatting (pretty)

**Week 3-4: Configuration System**
- [ ] Parse `bashrs.toml` configuration files
- [ ] Implement hierarchical configuration
- [ ] Add rule filtering (tags, include, exclude)
- [ ] Zero-config defaults

**Week 5-6: Auto-Fix Engine**
- [ ] Design fix application system
- [ ] Implement safe fix application
- [ ] Add `--fix` and `--dry-run` flags
- [ ] Handle fix conflicts

### Phase 2: Rule Implementation (8-10 weeks)

**Priority 1: Determinism + Idempotency Rules (bashrs-specific)**
- [ ] DET001-DET020: All determinism rules
- [ ] IDEM001-IDEM015: All idempotency rules
- [ ] Integration with existing semantic analysis

**Priority 2: Security Rules**
- [ ] SEC001-SEC045: Security vulnerability detection
- [ ] Command injection patterns
- [ ] Unsafe variable expansion

**Priority 3: Portability Rules**
- [ ] P001-P080: POSIX compliance checks
- [ ] Shell dialect compatibility
- [ ] Platform-specific issues

**Priority 4: Core ShellCheck-equivalent Rules**
- [ ] Q001-Q025: Quoting issues
- [ ] C001-C030: Conditional logic
- [ ] CMD001-CMD050: Command usage

**Priority 5: Style Rules**
- [ ] S001-S100: Code style and best practices

### Phase 3: Advanced Features (4-6 weeks)

**Week 1-2: Output Formats**
- [ ] JSON output format
- [ ] GitHub Actions format
- [ ] CheckStyle XML format
- [ ] Editor integration (LSP support)

**Week 3-4: Performance Optimization**
- [ ] Built-in caching system
- [ ] Parallel file processing
- [ ] Incremental linting (`--changed`)
- [ ] Benchmarking suite

**Week 5-6: Plugin System**
- [ ] Plugin API for custom rules
- [ ] Plugin loading mechanism
- [ ] Example plugin template

### Phase 4: Integration & Polish (2-4 weeks)

**Week 1-2: Integration**
- [ ] Pre-commit hook template
- [ ] CI/CD examples (GitHub Actions, GitLab CI)
- [ ] Editor plugins (VSCode, Vim, Emacs)

**Week 3-4: Documentation**
- [ ] Rule reference documentation
- [ ] Configuration guide
- [ ] Migration guide from shellcheck
- [ ] Tutorial and examples

---

## Testing Strategy (EXTREME TDD)

### Test Coverage Requirements

- **Unit Tests**: >85% coverage on all lint modules
- **Integration Tests**: End-to-end linting workflows
- **Property Tests**: Generative testing for rule correctness
- **Mutation Tests**: ≥90% kill rate on lint code

### Test Structure

```rust
// rash/src/lint/rules/determinism.rs

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;

    // RED: Write failing test first
    #[test]
    fn test_DET001_random_usage_detected() {
        let script = "SESSION_ID=$RANDOM";
        let violations = lint_script(script);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "DET001");
        assert_eq!(violations[0].severity, Severity::Error);
    }

    // GREEN: Implement rule to make test pass

    // REFACTOR: Clean up implementation

    // PROPERTY: Add generative tests
    #[test]
    fn test_DET001_property_random_always_detected() {
        use proptest::prelude::*;

        proptest!(|(var_name in "[A-Z_]+")| {
            let script = format!("{}=$RANDOM", var_name);
            let violations = lint_script(&script);
            prop_assert_eq!(violations.len(), 1);
        });
    }

    // CLI Integration Test
    #[test]
    fn test_DET001_cli_integration() {
        use std::fs;

        let temp = "/tmp/test_random.sh";
        fs::write(temp, "SESSION_ID=$RANDOM").unwrap();

        Command::cargo_bin("bashrs")
            .unwrap()
            .arg("lint")
            .arg(temp)
            .assert()
            .failure()
            .stdout(predicate::str::contains("DET001"));

        fs::remove_file(temp).unwrap();
    }
}
```

### Quality Gates

Before marking any rule as "completed":

- [ ] ✅ RED test written and failing
- [ ] ✅ GREEN implementation passes test
- [ ] ✅ REFACTOR code cleaned up (complexity <10)
- [ ] ✅ Property tests passing (100+ cases)
- [ ] ✅ Mutation score ≥90%
- [ ] ✅ Auto-fix implemented (if applicable)
- [ ] ✅ Documentation added to rule reference
- [ ] ✅ CLI integration test passing

---

## Success Metrics

### Phase 1 Success (Core Infrastructure)
- [ ] Can lint basic shell scripts
- [ ] Configuration system working
- [ ] Auto-fix applies correctly
- [ ] < 100ms for small files

### Phase 2 Success (Rules)
- [ ] 100+ rules implemented
- [ ] bashrs-specific rules complete (DET + IDEM)
- [ ] Security rules complete
- [ ] Parity with shellcheck core rules

### Phase 3 Success (Advanced Features)
- [ ] All output formats working
- [ ] < 10s for 1,000 files
- [ ] Plugin system functional
- [ ] Editor integration available

### Phase 4 Success (Production Ready)
- [ ] Documentation complete
- [ ] CI/CD integration examples
- [ ] 85%+ test coverage
- [ ] ≥90% mutation score
- [ ] Zero regressions

---

## Competitive Advantages

### vs ShellCheck

✅ **Faster**: 10-100x faster through Rust implementation
✅ **Auto-fix**: Automatic correction of 70%+ issues
✅ **Purification-aware**: Unique support for purified scripts
✅ **Zero-config**: Works out-of-the-box
✅ **Determinism checks**: bashrs-specific rules

### vs Traditional Linters

✅ **Shell-specific**: Deep understanding of shell semantics
✅ **Idempotency**: Unique idempotent operation verification
✅ **POSIX focus**: Strong portability guarantees
✅ **Security**: Injection vulnerability detection

### Unique bashrs Features

✅ **Purified mode**: Lint purified output for correctness
✅ **Determinism rules**: Detect $RANDOM, timestamps, wildcards
✅ **Idempotency rules**: Detect non-idempotent operations
✅ **Integration**: Seamless with bashrs purify workflow

---

## Example Workflows

### Workflow 1: Lint Raw Script

```bash
# 1. Write messy bash script
cat > deploy.sh << 'EOF'
#!/bin/bash
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
mkdir /app/releases/$RELEASE
rm /app/current
ln -s /app/releases/$RELEASE /app/current
EOF

# 2. Lint with bashrs
bashrs lint deploy.sh

# Output:
# deploy.sh
#   DET001 [Error] Non-deterministic $RANDOM usage
#   DET002 [Error] Non-deterministic timestamp
#   IDEM001 [Warning] Non-idempotent mkdir
#   IDEM002 [Warning] Non-idempotent rm
#   IDEM003 [Warning] Non-idempotent ln
#
# Found 2 errors, 3 warnings

# 3. Auto-fix issues
bashrs lint --fix deploy.sh

# 4. Verify fixes
cat deploy.sh
```

### Workflow 2: Lint Purified Script

```bash
# 1. Purify script
bashrs purify deploy.sh --output deploy-purified.sh

# 2. Lint purified output
bashrs lint --purified deploy-purified.sh

# Output:
# deploy-purified.sh
#   ✓ No determinism issues
#   ✓ All operations idempotent
#   ✓ POSIX compliant
#
# All checks passed! ✅
```

### Workflow 3: CI/CD Integration

```bash
# GitHub Actions
name: Lint Shell Scripts

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install bashrs
        run: cargo install bashrs

      - name: Lint shell scripts
        run: bashrs lint --format github

      - name: Auto-fix and commit
        if: github.event_name == 'push'
        run: |
          bashrs lint --fix .
          git config user.name "bashrs-bot"
          git commit -am "fix: Auto-fix shell script issues" || true
          git push || true
```

---

## Open Questions

1. **Rule Naming Convention**: Use shellcheck's SC#### format or create bashrs-specific format (DET###, IDEM###)?
   - **Recommendation**: bashrs format for clarity and extensibility

2. **Shell Dialect Detection**: Auto-detect from shebang vs require explicit configuration?
   - **Recommendation**: Auto-detect with override option

3. **Fix Safety**: Apply all fixes at once or require user approval for certain categories?
   - **Recommendation**: Auto-fix safe changes, warn for manual review on risky fixes

4. **Plugin System Priority**: Implement in Phase 3 or defer to Phase 5?
   - **Recommendation**: Phase 3 for extensibility

5. **LSP Integration**: Build into bashrs or separate project?
   - **Recommendation**: Separate but coordinated (bashrs-lsp)

---

## Conclusion

**bashrs lint** combines the speed of ruff, the comprehensiveness of shellcheck, and the developer experience of deno lint, with unique purification-aware capabilities that make it the definitive shell script linter.

### Key Differentiators

1. ✅ **10-100x faster** than existing shell linters
2. ✅ **800+ rules** covering all shell script issues
3. ✅ **70%+ auto-fix** rate for detected issues
4. ✅ **Zero configuration** required to get started
5. ✅ **Purification-aware** - unique to bashrs
6. ✅ **Determinism + Idempotency** - unique rule categories
7. ✅ **Production-ready** performance and quality

### Timeline

- **Phase 1**: 4-6 weeks (Core infrastructure)
- **Phase 2**: 8-10 weeks (Rule implementation)
- **Phase 3**: 4-6 weeks (Advanced features)
- **Phase 4**: 2-4 weeks (Integration & polish)

**Total**: 18-26 weeks to production-ready v1.0

---

**Status**: 📋 SPECIFICATION COMPLETE
**Next Step**: Phase 1 implementation (Core infrastructure)
**Ready for**: EXTREME TDD development
**Version**: 1.0.0-spec
**Date**: October 18, 2025
