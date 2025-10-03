# Contributing to Rash

Thank you for your interest in contributing to Rash!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/bashrs.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Commit your changes: `git commit -am 'Add some feature'`
7. Push to the branch: `git push origin feature/your-feature-name`
8. Create a Pull Request

## Development Guidelines

- **EXTREME TDD**: Write failing tests first (RED), implement (GREEN), refactor
- **POSIX Compliance**: All generated shell scripts must pass `shellcheck -s sh`
- **Test Coverage**: Maintain >85% code coverage
- **Property Tests**: Add property-based tests for new features
- **Documentation**: Update docs and examples for new features

## Testing

```bash
# Run all tests
cargo test

# Run property tests
cargo test --lib

# Run with coverage
make coverage

# Validate shell output
make test-shells
```

## Code Quality

Before submitting:
- Ensure all tests pass: `cargo test`
- Check formatting: `cargo fmt`
- Run clippy: `cargo clippy`
- Validate docs: `pmat validate-docs --root .`

## Questions?

Feel free to open an issue for:
- Bug reports
- Feature requests
- Questions about the codebase
- Suggestions for improvements

We appreciate your contributions!
