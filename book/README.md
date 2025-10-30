# The Rash Book

This directory contains **The Rash Book**, the comprehensive guide to using Rash for shell safety and purification.

## Structure

```
book/
├── book.toml           # mdBook configuration
├── src/                # Book source (Markdown)
│   ├── SUMMARY.md     # Table of contents
│   ├── introduction.md
│   ├── getting-started/
│   ├── concepts/
│   ├── linting/
│   ├── config/
│   ├── makefile/
│   ├── examples/
│   ├── advanced/
│   ├── reference/
│   └── contributing/
└── book/              # Generated output (HTML)
```

## Development

### Building the Book

```bash
# Build HTML output
mdbook build

# Serve locally with live reload
mdbook serve --open
```

The book will be available at http://localhost:3000

### Testing Examples

**CRITICAL**: All code examples in the book are automatically tested!

```bash
# Test all code examples
mdbook test

# This ensures:
# - Rust examples compile
# - Tests pass
# - Examples stay up-to-date
```

### Adding New Content

1. **Create a new chapter**:
   ```bash
   touch src/new-chapter.md
   ```

2. **Add to SUMMARY.md** with markdown link format

3. **Write content with tested examples**:
   ```markdown
   # New Chapter

   Here's a Rust example:

   ```rust
   #[test]
   fn test_example() {
       assert_eq!(2 + 2, 4);
   }
   ```
   ```

4. **Test your examples**:
   ```bash
   mdbook test
   ```

## Code Examples

### Marking Code Blocks

mdBook supports several code block annotations:

```markdown
\```rust
// This will be compiled and tested
#[test]
fn test_something() {
    assert!(true);
}
\```

\```rust,no_run
// This will be compiled but not run
fn main() {
    // ...
}
\```

\```rust,ignore
// This will not be compiled or tested
// Use for pseudo-code or incomplete examples
\```

\```bash
# Shell examples (not tested by mdbook)
bashrs --version
\```

\```bash,no_run
# Shell examples marked as no_run
cat > file.sh << 'EOF'
#!/bin/bash
echo "Hello"
EOF
\```
```

### Best Practices for Examples

1. **Make examples testable**: Wrap in tests when possible
2. **Keep examples focused**: One concept per example
3. **Add comments**: Explain what's happening
4. **Test after writing**: Run `mdbook test` immediately

## Release Process

### Pre-Release Checklist

Before every release, the book **MUST** be updated with new features:

```bash
# Run pre-release check
./scripts/check-book-updated.sh

# This checks:
# ✅ Book builds successfully
# ✅ All examples pass tests
# ✅ Book updated since last release
# ✅ Book/CHANGELOG in sync
```

**The release will be BLOCKED if this fails!**

### Updating for a Release

When adding a new feature:

1. **Document in book**:
   - Add chapter in appropriate section
   - Include tested examples
   - Show CLI usage

2. **Update CHANGELOG.md**:
   - List new features
   - Reference book chapters

3. **Test examples**:
   ```bash
   mdbook test
   ```

4. **Verify pre-release check**:
   ```bash
   ./scripts/check-book-updated.sh
   ```

## Deployment

The book is automatically deployed to GitHub Pages on push to `main`:

1. Push to `main` branch
2. GitHub Actions runs:
   - Tests all examples (`mdbook test`)
   - Builds book (`mdbook build`)
   - Deploys to GitHub Pages

3. Book available at: https://paiml.github.io/bashrs/

### Manual Deployment

```bash
# Build
mdbook build

# Deploy (if you have permissions)
# Usually handled by GitHub Actions
```

## Toyota Way Principles

This book follows Toyota Way principles:

### 自働化 (Jidoka) - Autonomation

- **Automated testing**: Every example is tested automatically
- **Build quality in**: Examples must compile before merge
- **Stop the line**: Failed examples block releases

### 現地現物 (Genchi Genbutsu) - Go and See

- **Real examples**: All examples use actual Rash commands
- **Tested code**: Not just documentation, but verified working code
- **Executable documentation**: Examples you can copy and run

### 改善 (Kaizen) - Continuous Improvement

- **Update with every feature**: Book evolves with code
- **User feedback**: Improve based on user questions
- **Keep examples fresh**: Re-test examples regularly

## Writing Style Guide

### Voice and Tone

- **Clear and direct**: Avoid jargon
- **Practical**: Focus on how-to, not philosophy
- **Example-driven**: Show, don't just tell

### Structure

Each chapter should have:

1. **Overview**: What will be covered
2. **Problem**: What issue does this solve?
3. **Solution**: How Rash addresses it
4. **Examples**: Tested code examples
5. **Best Practices**: Tips and recommendations
6. **See Also**: Related chapters

### Markdown Conventions

- Use `**bold**` for emphasis
- Use `code` for commands and file names
- Use `> Note:` for important asides
- Use `⚠️ Warning:` for critical information
- Use `✅ Best Practice:` for recommendations

## Troubleshooting

### "mdbook: command not found"

Install mdbook:

```bash
cargo install mdbook
```

### Examples fail to compile

1. Check Rust syntax
2. Add necessary imports
3. Mark as `no_run` if intended
4. Use `ignore` for pseudo-code

### Book doesn't update in browser

Clear browser cache or open in private/incognito mode.

### Pre-release check fails

Ensure book is updated:

```bash
# Stage book changes
git add book/

# Commit with feature changes
git commit -m "feat: add new feature + book update"
```

## FAQ

**Q: Do I need to update the book for every change?**

A: Yes, for any user-facing feature or behavior change. Bug fixes that don't change behavior can skip book updates.

**Q: What if my example doesn't compile as a test?**

A: Mark it as `no_run` or `ignore`, or restructure it as a proper test. Prefer tested examples when possible.

**Q: Can I skip the pre-release check?**

A: Not recommended, but: `export SKIP_BOOK_CHECK=1`. Only use for hotfixes or patch releases.

**Q: How do I add images?**

A: Place in `src/images/` and reference using markdown image syntax.

## Contributing

Contributions to the book are welcome!

1. Fork and clone
2. Create a branch
3. Make changes
4. Test: `mdbook test`
5. Build: `mdbook build`
6. Submit PR

See [Contributing Guide](./src/contributing/setup.md) for details.

## License

MIT License - same as Rash itself.

---

**Remember**: The book is not just documentation - it's executable, tested documentation that evolves with the code!
