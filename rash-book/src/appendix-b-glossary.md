# Appendix B: Glossary

This glossary defines key terms used throughout the bashrs book.

---

## A

**Andon Cord**: A Toyota manufacturing principle where any worker can stop the production line when they detect a quality issue. In bashrs development, this means immediately stopping work to fix bugs when discovered during validation.

**ash (Almquist Shell)**: A lightweight POSIX shell used in BusyBox, Alpine Linux, and embedded systems. bashrs generates scripts compatible with ash.

**AST (Abstract Syntax Tree)**: A tree representation of source code structure. bashrs parses bash scripts into an AST for analysis and transformation.

**Auto-fix**: Automatic correction of linting issues. bashrs can automatically fix some shellcheck violations.

---

## B

**Bash (Bourne Again Shell)**: The most common Unix shell, featuring extensions beyond POSIX sh. bashrs generates POSIX-compliant scripts that work in bash but don't require bash-specific features.

**Bashism**: A bash-specific feature that breaks POSIX compatibility (e.g., `[[`, arrays, `$RANDOM`). bashrs avoids all bashisms.

**Best Practices**: Production-proven patterns for writing safe, maintainable shell scripts. See Chapter 19.

**Bootstrap Installer**: A minimal, self-contained script that downloads and installs software. See Chapter 11.

**BusyBox**: A single executable containing minimal implementations of common Unix utilities, including ash shell. Used extensively in containers and embedded systems.

---

## C

**CI/CD (Continuous Integration/Continuous Deployment)**: Automated pipelines for testing and deploying code. bashrs integrates with GitHub Actions, GitLab CI, Jenkins, CircleCI, etc. See Chapter 15.

**Clippy**: Rust's linting tool. bashrs uses clippy with `-D warnings` to enforce code quality.

**Command Substitution**: Capturing command output in a variable using `$(command)` (POSIX) or `` `command` `` (legacy). bashrs uses `$(command)`.

---

## D

**dash (Debian Almquist Shell)**: The default `/bin/sh` on Debian and Ubuntu systems. dash is strictly POSIX compliant and 4x faster than bash. bashrs generates dash-compatible scripts.

**Determinism**: Property where the same inputs always produce the same outputs. bashrs removes non-deterministic elements like `$RANDOM`, timestamps, and process IDs.

**Dockerfile**: Configuration file for building Docker images. bashrs can lint Dockerfiles for security and best practices.

---

## E

**EXTREME TDD (Test-Driven Development)**: bashrs development methodology: TDD + property testing + mutation testing + fuzzing + PMAT + example verification. See CLAUDE.md for details.

**Error Handling**: Managing operation failures explicitly using `Result<T, E>` in Rust, which translates to exit codes and error checking in shell scripts. See Chapter 5.

---

## F

**False Positive**: A linting warning incorrectly flagged on valid code. bashrs actively fixes false positives (e.g., Issue #24).

**Fish Shell**: A modern, user-friendly shell with different syntax from POSIX sh. Future bashrs versions may support Fish → POSIX transpilation.

**Formal Verification**: Mathematical proof that code behaves correctly. bashrs plans to use Coq/Lean for formal verification of core transformations.

**Fuzzing**: Automated testing technique that generates random inputs to find edge cases and crashes. bashrs uses fuzzing to improve robustness.

---

## G

**GitHub Actions**: GitHub's CI/CD platform. bashrs provides GitHub Actions workflows for automated testing. See Chapter 15.

**GitLab CI**: GitLab's CI/CD platform. See Chapter 15 for integration examples.

**Glob Pattern**: Wildcard pattern for matching filenames (e.g., `*.txt`, `**/*.sh`). bashrs properly handles glob patterns to prevent injection.

---

## H

**Heredoc (Here Document)**: Multi-line string literal in shell scripts using `<<EOF` syntax. bashrs uses heredocs for multi-line content.

---

## I

**Idempotent**: Property where running an operation multiple times produces the same result as running it once. bashrs makes operations idempotent by using flags like `-p`, `-f`, `-sf`. See Chapter 9.

**Injection Attack**: Security vulnerability where untrusted input is executed as code. bashrs prevents injection by automatically quoting all variables. See Chapter 6.

---

## J

**Jenkins**: Popular CI/CD server. bashrs provides Jenkinsfile examples for integration. See Chapter 15.

---

## K

**Kaizen**: Japanese philosophy of continuous improvement. bashrs applies kaizen through incremental quality enhancements and EXTREME TDD.

**ksh (Korn Shell)**: Shell common on enterprise Unix systems (AIX, Solaris). bashrs generates ksh-compatible scripts.

---

## L

**Linting**: Static analysis to find bugs and enforce style. bashrs implements shellcheck rules for bash/Makefile linting.

**LSP (Language Server Protocol)**: Standard for IDE integration. bashrs plans to provide an LSP server for real-time linting in editors (v8.0.0).

---

## M

**Makefile**: Build automation file. bashrs can parse and purify Makefiles (v1.4.0).

**mdbook**: Tool for creating online books from Markdown. bashrs uses mdbook for documentation.

**Minimal Validation**: Default bashrs validation level with essential safety checks (8 rules). See Chapter 13.

**Mutation Testing**: Testing technique that modifies code to verify test quality. bashrs requires ≥90% mutation kill rate.

---

## N

**None Validation**: bashrs validation level with no checks (fastest, for prototyping only). See Chapter 13.

---

## O

**Option<T>**: Rust type representing optional values (Some or None). bashrs uses Option for values that may be absent.

---

## P

**Paranoid Validation**: bashrs's strictest validation level with 30+ rules for critical systems. See Chapter 13.

**PMAT (paiml-mcp-agent-toolkit)**: Quality analysis tool used in bashrs development for code complexity, quality scoring, and TDG verification.

**POSIX (Portable Operating System Interface)**: Standard for Unix compatibility. bashrs generates pure POSIX sh that works across all POSIX-compliant shells.

**Pre-commit Hook**: Git hook that runs before commits. bashrs provides pre-commit configuration for local validation.

**Property Testing**: Testing technique using generated inputs to verify properties. bashrs uses proptest for 100+ test cases per feature.

**Purification**: bashrs's process of transforming non-deterministic, non-idempotent bash into safe, POSIX-compliant shell scripts.

---

## Q

**Quality Gates**: Automated checks that must pass before code is committed/deployed. bashrs enforces 9 quality gates in pre-commit hooks.

---

## R

**Result<T, E>**: Rust type for operations that can fail, containing either Ok(T) or Err(E). bashrs uses Result extensively for error handling. See Chapter 5.

**Roadmap**: Long-term development plan. bashrs publishes a transparent roadmap. See Chapter 20.

---

## S

**SC#### (ShellCheck Rule)**: Unique identifier for shellcheck linting rules (e.g., SC2086 = unquoted variable). bashrs implements 20+ shellcheck rules.

**Security Linting**: Static analysis focused on security vulnerabilities (injection, command execution, etc.). bashrs provides 8+ security rules.

**shellcheck**: Popular shell script linter. bashrs integrates shellcheck validation and implements shellcheck-compatible rules.

**Shell Dialect**: Variant of shell with different features (sh, bash, dash, ash, zsh, ksh). bashrs generates scripts compatible with all major dialects. See Chapter 14.

**Shebang**: First line of script specifying interpreter (`#!/bin/sh`). bashrs uses `#!/bin/sh` for maximum portability.

**Strict Mode**: bashrs flag (`--strict`) that treats warnings as errors. Always use in CI/CD.

**Strict Validation**: bashrs validation level recommended for production (18 rules). See Chapter 13.

---

## T

**TDD (Test-Driven Development)**: Development practice of writing tests before implementation (RED → GREEN → REFACTOR). bashrs uses EXTREME TDD.

**Toyota Way**: Production system principles from Toyota emphasizing quality, continuous improvement, and stopping production to fix issues. bashrs applies Toyota Way principles.

**Transpilation**: Converting source code from one language to another (e.g., Rust → shell, bash → POSIX sh).

---

## U

**Undefined Variable**: Variable referenced before assignment. bashrs detects undefined variables with SC2154 rule.

---

## V

**Validation Level**: bashrs setting controlling strictness of checks (None, Minimal, Strict, Paranoid). See Chapter 13.

**Verification**: Process of checking that code meets specifications. bashrs plans formal verification with mathematical proofs.

---

## W

**WASM (WebAssembly)**: Binary format for running code in browsers. bashrs has experimental WASM support for browser-based linting.

---

## Z

**Zero-Warning Policy**: Development practice of treating all warnings as errors. bashrs enforces this with `--strict` flag.

**zsh (Z Shell)**: Feature-rich shell, default on macOS since Catalina. bashrs generates zsh-compatible scripts.

---

## Symbols

**`$@`**: Special shell parameter containing all positional arguments. bashrs uses `"$@"` instead of arrays for POSIX compatibility.

**`$?`**: Exit code of last command (0 = success, 1+ = error). bashrs uses exit codes for error handling.

**`$(command)`**: POSIX command substitution syntax (preferred over backticks).

**`${var}`**: Variable expansion with braces. bashrs always uses braces for clarity: `"${var}"`.

**`#!/bin/sh`**: Shebang for POSIX sh. bashrs uses this for maximum portability.

**`set -e`**: Shell option to exit on error. bashrs uses `set -euo pipefail` in strict mode.

---

## Acronyms

| Acronym | Full Name |
|---------|-----------|
| AFL | American Fuzzy Lop (fuzzing tool) |
| API | Application Programming Interface |
| AST | Abstract Syntax Tree |
| CI/CD | Continuous Integration/Continuous Deployment |
| CLI | Command-Line Interface |
| LSP | Language Server Protocol |
| MCP | Model Context Protocol |
| POSIX | Portable Operating System Interface |
| PR | Pull Request |
| REPL | Read-Eval-Print Loop |
| ROI | Return on Investment |
| SATD | Self-Admitted Technical Debt |
| SC | ShellCheck (rule prefix) |
| TDD | Test-Driven Development |
| TDG | Test-Driven Generation |
| WASM | WebAssembly |
| YAML | YAML Ain't Markup Language |

---

## Common Commands

| Command | Description |
|---------|-------------|
| `bashrs build` | Transpile Rust to shell with validation |
| `bashrs parse` | Parse bash/Makefile to AST |
| `bashrs purify` | Transform bash to safe POSIX sh |
| `bashrs lint` | Lint bash/Makefile with shellcheck rules |
| `bashrs check` | Type-check and validate |
| `bashrs bench` | Benchmark performance and memory |
| `cargo test` | Run Rust test suite |
| `cargo clippy` | Lint Rust code |
| `mdbook test` | Test book examples |
| `shellcheck -s sh` | Validate POSIX compliance |

---

## See Also

- **Chapter 1**: Getting started with bashrs
- **Chapter 13**: Validation levels explained
- **Chapter 19**: Best practices for production scripts
- **Chapter 20**: Roadmap and future features
- **Appendix C**: Shell compatibility matrix
- **Appendix D**: Complete API reference

---

*Glossary last updated: 2025-11-14 for bashrs v6.34.1*
