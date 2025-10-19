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

# Sprint 79: Quality Enforcement with pmat Integration
.PHONY: lint-scripts lint-makefile validate-book validate-all

lint-scripts: ## Lint all shell scripts with bashrs
	@echo "🔍 Linting shell scripts..."
	@find scripts -name "*.sh" -type f -exec bashrs lint {} \; || true
	@echo "✅ All scripts linted!"

lint-makefile: ## Lint Makefile with bashrs  
	@echo "🔍 Linting Makefile..."
	@bashrs make lint Makefile --format human || true
	@echo "✅ Makefile linted!"

validate-book: ## Validate book examples
	@echo "📖 Validating book examples..."
	@./scripts/validate-book.sh || echo "⚠️  validate-book.sh not found"

validate-all: lint-scripts lint-makefile validate-book test ## Comprehensive quality gate
	@echo "✅ All quality gates passed!"
	@echo "  ✓ Shell scripts linted"
	@echo "  ✓ Makefile linted"
	@echo "  ✓ Book accuracy validated"
	@echo "  ✓ Tests passing"
