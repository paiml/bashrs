# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-06-04

### 🔧 Critical Installation and Documentation Fixes

#### Fixed
- **Installation Script**: Fixed broken install.sh in GitHub releases with proper error handling and verification
- **POSIX Compliance**: Fixed generated shell scripts to use POSIX-compatible IFS setting instead of bash-specific `$'\n\t'`
- **Documentation**: Corrected README.md examples to use supported Rust syntax (removed unsupported `#[rash::main]`)
- **Release Workflow**: Updated to generate proper install.sh with dynamic versioning from git tags

#### Added  
- **Robust Installer**: Created install-rash.sh with comprehensive error checking and user guidance
- **Installation Tests**: Added comprehensive Rust test suite for installation workflows
- **Troubleshooting Guide**: Added detailed installation troubleshooting with multiple methods
- **PATH Setup Instructions**: Clear instructions for bash and zsh users

#### Changed
- **README.md**: Complete overhaul with accurate installation instructions and examples
- **Installation Method**: Primary installer now hosted in repository (install-rash.sh) with fallback to GitHub releases
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

[0.2.0]: https://github.com/paiml/rash/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/paiml/rash/releases/tag/v0.1.0