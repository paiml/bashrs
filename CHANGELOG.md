# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-10-02

### 🎉 Major Release - Production Ready (Sprints 1-11)

This release represents **11 sprints of EXTREME TDD development** using Toyota Way principles. The transpiler is now production-ready with **7/11 edge cases fixed** and exceptional quality metrics.

### Added

#### Core Language Features
- **Arithmetic expressions** with POSIX `$((expr))` syntax (Sprint 11, #TICKET-5006)
  - Support for `+`, `-`, `*`, `/` operators
  - Nested arithmetic: `$((a + b) * c)`

- **Function return values** via echo and command substitution (Sprint 11, #TICKET-5007)
  - Functions with return types emit `echo`
  - Call sites capture with `$(command)`

- **`println!` macro support** (Sprint 10, #TICKET-5002)

- **Negative integer literals** (Sprint 10, #TICKET-5003)

- **Integer comparison operators** (Sprint 10, #TICKET-5004)
  - POSIX test syntax: `[ "$x" -gt 0 ]`

- **Global function scope** (Sprint 10, #TICKET-5005)

#### MCP Server
- **rash-mcp** - Model Context Protocol server
  - TranspileHandler with type-safe JSON Schema I/O
  - 3/3 handler tests passing

#### Testing & Quality
- **520 total tests** (100% pass rate)
- **23 property tests** (~13,300 cases)
- **24 ShellCheck tests** (100% pass)
- **Coverage: 85.36%** (target achieved!)

### Fixed

- Empty function bodies (#TICKET-5001)
- println! parsing (#TICKET-5002)
- Negative integers → "unknown" (#TICKET-5003)
- Comparison operators wrong syntax (#TICKET-5004)
- Functions nested in main() (#TICKET-5005)
- Arithmetic → no-ops (#TICKET-5006)
- Return values → "unknown" (#TICKET-5007)

### Changed

- **96% complexity reduction** (Sprint 7)
- **86% parse complexity reduction** (Sprint 8)
- All core functions <10 cognitive complexity

### Performance

- **21.1µs** transpile time (100x target!)
- **3.7MB** binary size
- Zero runtime dependencies

### Known Limitations

Not yet supported:
- For loops (P2 - deferred)
- Match expressions (P2 - deferred)

### Quality Metrics (v0.4.0)

| Metric | Status |
|--------|--------|
| Tests | 520/520 ✅ |
| Coverage | 85.36% ✅ |
| Complexity | <10 ✅ |
| Performance | 21µs ✅ |
| Edge Cases | 7/11 (64%) 🟡 |

---

## [0.3.1] - 2025-06-05

### 🔧 Default Features Update

#### Changed
- **Default Features**: Now includes compile mode and playground by default
  - Binary compilation and self-extracting scripts work out-of-the-box
  - Interactive playground available without additional features
  - No need for `--all-features` flag anymore

#### Added
- Minimal build option for smaller binaries (`--no-default-features --features minimal`)
- Updated documentation to reflect new defaults

#### Binary Sizes
- Default build: ~4.6MB (includes all core features)
- Minimal build: ~3.2MB (transpilation only)

## [0.3.0] - 2025-06-05

### 🚀 Major Features

#### Added
- **Binary Compilation Mode**: Create standalone executables and self-extracting scripts
  - Self-extracting shell scripts with Zstandard compression
  - Static binary generation with embedded dash runtime
  - Minimal container generation (Docker/OCI formats)
  
- **Interactive Playground** (Experimental): TypeScript-style REPL for development
  - Live transpilation with incremental parsing
  - Session management with save/load/share capabilities
  - VI/Emacs keybindings and multiple layout modes
  - SIMD-accelerated syntax highlighting
  
- **Formal Verification Engine**: Mathematical proofs of correctness
  - Property-based verification for injection safety
  - Machine-readable proof generation
  - AST inspection with complexity analysis
  
- **Kaizen Mode**: Continuous improvement tooling
  - Quality metrics dashboard
  - Performance regression detection
  - Demo mode for showcasing capabilities

#### Technical Improvements
- **Architecture**: Modular design with clean separation of concerns
- **Testing**: 88.70% coverage with 400+ unit tests and 1000+ property tests
- **Performance**: <25ms transpilation, <10MB memory usage
- **Quality**: Average complexity 4.2 per function (threshold: 10)

#### Dependencies
- Added tree-sitter, ropey, dashmap, petgraph for playground features
- Added zstd, tar, brotli for compression and packaging
- Updated all dependencies to latest versions

### Fixed
- CI/CD streamlined to focus on Linux platform
- Clippy warnings and code quality issues resolved
- Improved error messages and handling

## [0.2.1] - 2025-06-04

### 🔧 Critical Installation and Documentation Fixes

#### Fixed
- **Installation Script**: Fixed broken install.sh in GitHub releases with proper error handling and verification
- **POSIX Compliance**: Fixed generated shell scripts to use POSIX-compatible IFS setting instead of bash-specific `$'\n\t'`
- **Documentation**: Corrected README.md examples to use supported Rust syntax (removed unsupported `#[bashrs::main]`)
- **Release Workflow**: Updated to generate proper install.sh with dynamic versioning from git tags

#### Added  
- **Robust Installer**: Created install-bashrs.sh with comprehensive error checking and user guidance
- **Installation Tests**: Added comprehensive Rust test suite for installation workflows
- **Troubleshooting Guide**: Added detailed installation troubleshooting with multiple methods
- **PATH Setup Instructions**: Clear instructions for bash and zsh users

#### Changed
- **README.md**: Complete overhaul with accurate installation instructions and examples
- **Installation Method**: Primary installer now hosted in repository (install-bashrs.sh) with fallback to GitHub releases
- **Error Messages**: Improved error messages in generated shell scripts

## [0.2.0] - 2025-06-04

### 🚀 Major Technical Debt Reduction & Code Quality Improvements

#### Added
- Comprehensive enterprise test suite for major tech companies (Amazon, Google, Microsoft, Meta, Netflix, Uber)
- Open source project bootstrap examples (Kubernetes, Node.js, Python)
- Enhanced security documentation and implementation clarity
- Technical debt analysis using paiml-mcp-agent-toolkit

#### Changed
- **Reduced technical debt by 42%**: From 133.5 to 77.5 hours of estimated debt
- **Reduced maximum cyclomatic complexity by 18.75%**: From 32 to 26
- **Reduced compilation errors by 58%**: From 12 to 5 critical issues
- Refactored high-complexity functions across multiple modules:
  - `Stmt::validate` function in `ast/restricted.rs` (complexity 32 → 18)
  - `PosixEmitter::write_runtime` and `emit_ir` functions in `emitter/posix.rs`
  - `eval_command` function in `formal/semantics.rs`

#### Fixed
- All clippy warnings resolved (borrowed_box, ptr_arg issues)
- Comprehensive code formatting applied across codebase
- Enhanced SATD (Self-Admitted Technical Debt) documentation
- Improved method extraction and single responsibility principle adherence

#### Performance
- All 324 tests passing after extensive refactoring
- No functional regressions introduced
- Verified semantic preservation of refactored code
- Improved maintainability while preserving functionality

## [0.1.0] - 2025-06-04

### Initial Release
- Rust-to-Shell transpiler core functionality
- POSIX compliance and ShellCheck validation
- Basic CLI interface and project initialization
- Formal verification framework
- Comprehensive test suite

[0.2.0]: https://github.com/paiml/bashrs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/paiml/bashrs/releases/tag/v0.1.0