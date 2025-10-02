# Makefile for Rash - Following Toyota Way: "make coverage" just works
#
# Sprint 5 - Code Coverage Infrastructure
# Following: docs/specifications/COVERAGE.md (Two-Phase Pattern)

.PHONY: help test coverage coverage-ci coverage-clean

# Default target
help:
	@echo "Rash Build Targets:"
	@echo "  make test          - Run all tests"
	@echo "  make coverage      - Generate coverage report (HTML)"
	@echo "  make coverage-ci   - Generate LCOV for CI/CD"
	@echo "  make coverage-clean - Clean coverage artifacts"

# Run all tests
test:
	cargo test --all-targets

# Generate HTML coverage report (Two-Phase Pattern)
coverage:
	@echo "Phase 1: Running tests with instrumentation..."
	cargo llvm-cov clean --workspace
	cargo llvm-cov --no-report nextest
	@echo "Phase 2: Generating HTML report..."
	cargo llvm-cov report --html --open

# Generate LCOV for CI/CD (Two-Phase Pattern)
coverage-ci:
	@echo "Phase 1: Running tests with instrumentation..."
	cargo llvm-cov clean --workspace
	cargo llvm-cov --no-report nextest --all-features
	@echo "Phase 2: Generating LCOV report..."
	cargo llvm-cov report --lcov --output-path lcov.info
	@echo "✓ Coverage report generated: lcov.info"

# Clean coverage artifacts
coverage-clean:
	cargo llvm-cov clean --workspace
	rm -f lcov.info coverage.xml
	rm -rf target/llvm-cov target/coverage
	find . -name "*.profraw" -delete
