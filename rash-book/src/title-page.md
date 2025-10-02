# The Rash Programming Language

**Deterministic Rust-to-Shell Transpilation**

---

**Version:** 0.1.0
**Status:** Beta
**Test Coverage:** 85.36% (core modules)
**Test Pass Rate:** 100% (539/539 tests)

---

## What is Rash?

Rash (Rust-to-Shell) is a transpiler that converts a restricted subset of Rust into POSIX-compliant shell scripts. It's designed for creating deterministic, verifiable bootstrap installers with formal correctness guarantees.

## Key Features

- ✅ **POSIX Compliance**: Every generated script passes `shellcheck -s sh`
- ✅ **Determinism**: Same Rust input produces byte-identical shell output
- ✅ **Safety**: Injection-proof quoting and escaping
- ✅ **Verification**: Multiple verification levels (basic, strict, paranoid)
- ✅ **Performance**: <100µs transpilation time for typical scripts
- ✅ **Quality**: 100% test pass rate, 85%+ coverage, <10 complexity

## Philosophy

Rash follows **EXTREME TDD** principles:
- **RED**: Write tests first, documenting expected behavior
- **GREEN**: Implement features to make tests pass
- **REFACTOR**: Optimize while maintaining tests
- **Toyota Way**: Jidoka (build quality in), Hansei (reflection), Kaizen (continuous improvement)

## Status

This book documents working features through test-driven examples. Every code sample in this book:
1. Has a corresponding test case
2. Is verified to work with the current Rash version
3. Generates valid, ShellCheck-compliant shell scripts
4. Is deterministic and idempotent

---

**© 2025 Pragmatic AI Labs**
**License:** MIT
**Repository:** https://github.com/paiml/bashrs
