# The Rash Programming Language Book

This is the official book for the Rash programming language - a Rust-to-Shell transpiler following EXTREME TDD principles.

## 📖 Reading the Book

**Online**: https://paiml.github.io/bashrs/ (coming soon)

**Locally**:
```bash
mdbook serve rash-book
# Open http://localhost:3000
```

## 🔨 Building

```bash
# Install mdbook if needed
cargo install mdbook

# Build the book
mdbook build rash-book

# Output: rash-book/book/
```

## 📝 Contributing

Found an edge case? Want to improve documentation?

1. Create test case: `tests/edge-cases/test_XX_your_case.rs`
2. Document in appropriate chapter
3. Run `mdbook build rash-book` to verify
4. Submit PR

## 📊 Current Status

- **Chapters**: 20 planned
- **Completed**: 3 (Ch1, Ch18, Test Status)
- **Edge Cases Documented**: 11
- **Examples Tested**: 5 in Ch1

## 🎯 Philosophy

This book follows **EXTREME TDD**:
- **RED**: Write tests first
- **GREEN**: Make them pass
- **REFACTOR**: Optimize with safety
- **Toyota Way**: Jidoka, Hansei, Kaizen

Every code example in this book:
1. Has a corresponding test
2. Is verified with current Rash version
3. Generates valid, ShellCheck-compliant scripts
4. Is deterministic and idempotent

## 🏗️ Book Structure

```
rash-book/
├── book.toml         # mdBook configuration
├── src/              # Markdown chapters
│   ├── SUMMARY.md    # Table of contents
│   ├── ch*.md        # Chapter files
│   └── appendix-*.md # Appendices
├── theme/            # Custom CSS/JS (future)
└── tests/            # Test cases for examples (future)
```

## 📚 Chapters

### Part I: Core Transpilation (Test-Driven)
- Ch 1: Hello Shell Script ✅
- Ch 2-6: Variables, Functions, Control Flow, Error Handling, Escaping 📋
- Ch 7-10: POSIX, ShellCheck, Determinism, Security 📋

### Part II: Advanced Features
- Ch 11-14: Bootstrap, Config, Verification, Dialects 📋

### Part III: Tool Integration
- Ch 15-17: CI/CD, MCP Server, Testing 📋

### Part IV: Edge Cases and Limitations
- Ch 18: Known Limitations ✅
- Ch 19-20: Best Practices, Roadmap 📋

## 🐛 Edge Cases

We've discovered and documented **11 edge cases** (see Chapter 18):
- 🔴 3 Critical (P0): Empty functions, println!, negative integers
- 🟡 2 High (P1): Comparisons, function nesting
- 🟢 4 Medium (P2): For loops, match, returns, arithmetic
- 🔵 2 Low (P3): Empty main, integer overflow

## 🚀 Next Steps

1. **Fix P0 edge cases** (Sprint 10)
2. **Complete core chapters** (Ch2-6)
3. **Add 100+ tested examples**
4. **Deploy to GitHub Pages**

---

**License**: MIT
**Repository**: https://github.com/paiml/bashrs
**Questions**: File an issue on GitHub
