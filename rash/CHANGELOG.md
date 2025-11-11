# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Issue #4 RESOLVED - Bash Parser Completeness (9 Phases)
All bash parser gaps identified in benchmarks have been resolved through EXTREME TDD:

**Phase 1: $RANDOM and $$ Variables**
- Added special variable parsing for non-deterministic constructs
- Lexer support for $RANDOM, $$, $@, process ID variables

**Phase 2-7: Redirection Operators (Complete Set)**
- Output redirection: `>` (Phase 2)
- Append redirection: `>>` (Phase 3)
- Input redirection: `<` (Phase 4)
- Error redirection: `2>` (Phase 5)
- Append error redirection: `2>>` (Phase 6)
- Combined redirection: `&>` (Phase 7)
- File descriptor duplication: `2>&1` (Phase 8)

**Phase 8: Heredoc Support**
- Complete heredoc parsing (<<EOF ... EOF)
- Unblocks benchmark validation for real-world scripts

**Phase 9: Pipeline Support**
- Basic pipeline operator (`|`) fully implemented
- AST representation for multi-command pipelines
- REPL display support for pipeline commands

**Additional Parser Enhancements**:
- Escape sequence support in barewords (`\;` for find -exec)
- Arithmetic expansion: `$((expr))`
- Command substitution: `$(command)`
- Date format syntax: `date +%Y-%m-%d`
- Multi-item for loop support: `for i in 1 2 3`

#### Issue #2 RESOLVED - Makefile Multi-line Format Preservation
- Parser-level line break tracking with metadata
- Generator reconstruction of backslash continuations
- `--preserve-formatting` and `--skip-consolidation` flags working correctly
- Original indentation preserved in continued lines

**Example**:
```makefile
# Input and output both preserve backslashes:
build:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo build --release; \
    fi
```

#### Dockerfile Purification (6 Transformations)
Complete Dockerfile purification through EXTREME TDD:

**DOCKER001: FROM debian:latest → FROM debian:stable-slim**
- Pins unpinned base images to specific stable versions
- Converts `latest` tags to explicit version tags
- Adds `-slim` suffix for minimal images

**DOCKER002: Pin unpinned base images**
- Ensures all FROM directives use explicit version tags
- Prevents non-deterministic builds from tag updates

**DOCKER003: Package manager cleanup**
- apt/apt-get: Adds `&& rm -rf /var/lib/apt/lists/*`
- apk: Adds `&& rm -rf /var/cache/apk/*`
- Reduces image size, improves layer caching

**DOCKER005: Add --no-install-recommends**
- Inserts `--no-install-recommends` flag after `apt-get install -y`
- Minimizes installed packages, reduces image size

**DOCKER006: Convert ADD to COPY for local files**
- Replaces `ADD` with `COPY` for local file operations
- Preserves `ADD` for URLs and tarballs (correct behavior)
- Follows Docker best practices

#### REPL Enhancements
- AST display mode (`format_ast()`, `format_statement()`)
- Complete coverage of all 11 BashStmt variants
- Property-based tests (6 properties, 100+ cases each)

### Fixed

#### Test Suite Stability
- Fixed flaky tests under high parallel execution (7000+ tests)
- Five Whys root cause analysis (system load sensitivity)
- Retry logic for transient failures (3 attempts, 100ms pause)
- Zero tolerance: All 6484 tests now pass reliably

#### Parser Fixes
- Multi-item for loop parsing (was single-item only)
- Bareword escape sequences (`\;` in find -exec commands)
- Optional semicolon handling (`;` before `then`/`do`)

### Quality Metrics

**Test Suite**:
- ✅ **6,484 tests passing** (was 712 in RC2, +5,772 tests, +811% growth)
- ✅ **100% pass rate** (zero failures, zero ignored)
- ✅ **Zero regressions** maintained across all changes

**Code Quality**:
- ✅ **TDG Score: 94.6/100** (Grade A) - Excellent technical debt management
- ✅ **Clippy clean** (zero warnings in library code)
- ✅ **Complexity target met** (median cyclomatic: 9.0, target: <10)
- ✅ **Zero defects** policy maintained

**Test Coverage**:
- ✅ Property-based testing (100+ cases per feature)
- ✅ Mutation testing configured (90% kill rate target)
- ✅ Integration tests (CLI, end-to-end workflows)
- ✅ Differential testing (bash vs transpiled execution)

**Known Technical Debt**:
- Issue #3: Mutation coverage for generators.rs (P2, explicitly deferred)
- 3 functions exceed complexity target (11-12 vs <10): quality-gate.rs, sc2154.rs, docker004.rs

### Documentation

**Issue Documentation**:
- Issue #2: Multi-line preservation (RESOLVED, 103 lines)
- Issue #3: Mutation coverage gap (OPEN, P2, 89 lines)
- Issue #4: Benchmark parser gaps (RESOLVED, 236 lines)

**Book Updates**:
- Comprehensive Makefile testing chapter
- Makefile purification examples with test generation
- EXTREME TDD session summaries (Toyota Way)
- Dogfooding reports for Makefile purification

### Changed

#### EXTREME TDD Methodology
All features developed using RED → GREEN → REFACTOR → VERIFY workflow:
- RED: Write failing test, verify it fails
- GREEN: Minimal implementation, test passes
- REFACTOR: Clean code, complexity <10
- VERIFY: All tests pass, zero regressions, clippy clean

#### Performance
- Benchmark fixtures updated (minimal → small → medium → large)
- Real-world script validation (500-5700 line bash scripts)

### Removed
- 199 cruft files from repository root (cleanup)

---

## [1.0.0-rc2] - 2025-10-10

### Added

#### Bash Parser Module
- Comprehensive lexer with bash-specific token handling (454 lines)
  - Variables, strings (single/double quotes), operators, keywords
  - Path and glob support (`/`, `.`, `-`, `*`, `?`, `~`, `:`)
  - Bare word parsing for bash-specific syntax
- Recursive descent parser for bash syntax (519 lines)
  - Assignment statements with export support
  - Function definitions and calls
  - Control flow: if/elif/else, for loops, while loops, case statements
  - Test expressions (`[...]` and `[[...]]`)
  - Arithmetic expressions (`$((...))`)
  - Optional semicolon handling before `then` and `do`
  - Comprehensive error reporting with source spans
- Type-safe Abstract Syntax Tree (AST) (267 lines)
  - Complete bash construct representation
  - Metadata tracking (source file, line count, parse time)
  - Span information for error reporting
- Semantic analysis (417 lines)
  - Variable scope tracking across function boundaries
  - Effect tracking (file operations, network, environment modifications)
  - Type inference for variables
  - Read/write analysis

#### Bash-to-Rash Transpiler
- Code generator (457 lines)
  - Converts bash AST to Rust-like syntax (Rash)
  - Pattern-based translation with semantic preservation
  - Proper indentation and formatting
  - Variable assignments → `let` bindings
  - Exported variables → `std::env::set_var()`
  - Functions → `fn` declarations
  - Control flow → Rust equivalents (if/else, while, for, match)
  - Test expressions → comparison operators
  - Comment preservation
- Translation patterns (147 lines)
  - Variable patterns (local/exported)
  - Command patterns (simple/piped/redirected)
  - Control flow patterns
  - Function patterns
- Purification module (470 lines)
  - Idempotency transforms (mkdir -p, rm -f, etc.)
  - Determinism fixes (replaces $RANDOM, $SECONDS, $BASHPID, $PPID)
  - Non-deterministic variable tracking
  - Side-effect isolation and reporting

#### Testing Infrastructure
- **Property-based testing** with proptest
  - 10 property tests with 1000 test cases per property
  - Properties verified:
    - Transpilation determinism (same input → same output)
    - Variable preservation (names/values maintained)
    - Function name preservation
    - Control flow correctness
    - Variable scoping accuracy
    - Exported variables tracked as effects
    - Arithmetic preservation
    - Code structure validity
  - Custom generators for bash constructs (229 lines)
- **Differential testing** framework (307 lines)
  - 8 differential tests comparing bash vs transpiled execution
  - Tests coverage:
    - Simple echo commands
    - Variable assignment and usage
    - Command sequences
    - Function definitions and calls
    - Conditional statements (if/else)
    - Basic installer patterns
    - Determinism verification
  - Coverage metrics: 100% of basic constructs
- **Mutation testing** configuration
  - Configured with cargo-mutants
  - 1909 mutants identified across codebase
  - 431 mutants in bash_parser/bash_transpiler modules
  - Parallel execution with 4 jobs
  - 60-second timeout per mutant
  - Target: ≥85% mutation kill rate
- **Integration tests**
  - 17 parser integration tests
  - 11 transpiler integration tests
  - 3 purification tests

### Quality Metrics
- ✅ **712 tests passing** (0 failures)
- ✅ **0 security vulnerabilities**
- ✅ **0 code duplication**
- ✅ Test coverage adequate for RC2
- ⚠️  125 complexity warnings (refactoring targets for v1.0.0)
- ⚠️  13 TODO items to address
- ⚠️  44.5% dead code (includes unused formal verification infrastructure)

### Documentation
- Updated bash-to-rush-spec.md to use `make coverage` instead of cargo-tarpaulin
- Created comprehensive test generation specification (600+ lines)
- Added PMAT quality verification documentation

### Changed
- Improved error handling with source location tracking
- Enhanced semantic analysis for better scope tracking
- Optimized parser for better performance

### Fixed
- Lexer handling of path characters (`/`, `*`) in bare words
- Parser handling of optional semicolons before `then` and `do`
- Borrow checker issues in semantic analyzer using `std::mem::replace`
- PropTest type inference errors with boxed strategies
- Module visibility for test generators
- Dead code warnings in test infrastructure files

## [1.0.0-rc1] - 2025-09-15

### Added
- Initial Rust-to-Shell transpiler implementation
- Basic CLI with build and check subcommands
- Restricted Rust AST support
- Shell script emission
- Integration with bashrs_runtime

### Known Limitations (RC2)
The following features are planned for v1.0.0:
- Rash→Bash back-transpilation (Phase 4)
- Full mutation testing run (target ≥85% kill rate)
- Complexity refactoring (67 → <15 for complex functions)
- Resolution of 13 TODO items
- Dead code elimination (44.5% → <15%)

### Migration Guide
No breaking changes from RC1 to RC2. All RC1 code continues to work.

---

**Contributors**: Pragmatic AI Labs Team
**License**: MIT
**Repository**: https://github.com/paiml/bashrs
