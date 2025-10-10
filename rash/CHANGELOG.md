# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
