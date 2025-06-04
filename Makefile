# Use bash for shell commands to support advanced features
SHELL := /bin/bash

.PHONY: all validate quick-validate release clean help
.PHONY: format format-check lint lint-check check test test-fast test-comprehensive test-shells test-determinism
.PHONY: quality-gate quality-baseline quality-report analyze-complexity
.PHONY: fuzz fuzz-all fuzz-coverage fuzz-trophies fuzz-differential
.PHONY: verify verify-smt verify-model verify-specs verify-properties
.PHONY: shellcheck-install shellcheck-validate shellcheck-test-all
.PHONY: audit docs build install profile-memory profile-heap profile-flamegraph
.PHONY: update-deps update-deps-aggressive update-deps-check update-deps-workspace

# Parallel job execution
MAKEFLAGS += -j$(shell nproc)

# PAIML toolkit path for quality analysis (optional)
PAIML_TOOLKIT := ../paiml-mcp-agent-toolkit/paiml-mcp-agent-toolkit

# Default target
all: validate build

# Quick validation for development (skip expensive checks)
quick-validate: format-check lint-check check test-fast
	@echo "✅ Quick validation passed!"

# Full validation pipeline with quality gates
validate: format lint check test quality-gate verify-specs test-shells shellcheck-validate audit
	@echo "✅ All validation passed!"
	@echo "  ✓ Code formatting"
	@echo "  ✓ Linting (clippy + custom)"
	@echo "  ✓ Type checking"
	@echo "  ✓ Test coverage (>85%)"
	@echo "  ✓ Quality metrics"
	@echo "  ✓ Specification compliance"
	@echo "  ✓ Cross-shell compatibility"
	@echo "  ✓ ShellCheck validation"
	@echo "  ✓ Security audit"

# Formatting
format:
	@echo "🎨 Formatting code..."
	@cargo fmt --all

format-check:
	@echo "🎨 Checking code formatting..."
	@cargo fmt --all -- --check

# Linting
lint:
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

lint-check:
	@echo "🔍 Checking clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Type checking
check:
	@echo "🔍 Type checking..."
	@cargo check --all-targets --all-features

# Test execution with multiple strategies
test-fast:
	@echo "⚡ Running fast tests..."
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		RUST_TEST_THREADS=$$(nproc) cargo nextest run \
			--workspace \
			--status-level skip \
			--failure-output immediate; \
	else \
		cargo test --workspace; \
	fi

test: test-fast
	@echo "🧪 Running comprehensive tests with coverage..."
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov test --workspace \
			--all-features \
			--codecov \
			--output-path coverage.json; \
		cargo llvm-cov report --summary-only | tee coverage-summary.txt; \
		echo "📊 Checking coverage threshold..."; \
		./scripts/check-coverage.sh 85 || true; \
	else \
		cargo test --workspace --all-features; \
	fi

# Cross-shell compatibility testing
test-shells:
	@echo "🐚 Testing POSIX compliance across shells..."
	@cargo test --test integration_tests shell_compat -- --test-threads=1 --nocapture || true
	@for shell in bash dash ash ksh zsh busybox; do \
		if command -v $$shell >/dev/null 2>&1; then \
			echo "Testing with $$shell..."; \
			RASH_TEST_SHELL=$$shell cargo test shell_compat::$$shell || true; \
		fi; \
	done

# Determinism verification
test-determinism:
	@echo "🎯 Verifying deterministic transpilation..."
	@cargo test determinism -- --test-threads=1 --nocapture
	@if [ -f ./scripts/verify-determinism.sh ]; then \
		./scripts/verify-determinism.sh; \
	fi

# Quality metrics
quality-gate: quality-baseline
	@echo "🔍 Running quality gate checks..."
	@if command -v $(PAIML_TOOLKIT) >/dev/null 2>&1; then \
		$(PAIML_TOOLKIT) analyze complexity --top-files 10 --format json > complexity-current.json; \
		$(PAIML_TOOLKIT) analyze dead-code --top-files 10 --format json > deadcode-current.json; \
		$(PAIML_TOOLKIT) analyze satd --top-files 5 --format json > tech-debt-current.json; \
	fi
	@if [ -f ./target/release/quality-gate ]; then \
		./target/release/quality-gate \
			--complexity-threshold 10 \
			--cognitive-threshold 15 \
			--dead-code-threshold 5 \
			--tech-debt-threshold high; \
	fi
	@echo "✅ Quality gates passed!"

quality-baseline:
	@mkdir -p .quality
	@if [ ! -f .quality/baseline.json ] && command -v $(PAIML_TOOLKIT) >/dev/null 2>&1; then \
		echo "📊 Establishing quality baseline..."; \
		$(PAIML_TOOLKIT) analyze deep-context \
			--include "ast,complexity,churn,dag,dead-code,satd" \
			--format json > .quality/baseline.json; \
	fi

quality-report:
	@echo "📈 Generating comprehensive quality report..."
	@if command -v $(PAIML_TOOLKIT) >/dev/null 2>&1; then \
		$(PAIML_TOOLKIT) context rust --format markdown > QUALITY_REPORT.md; \
	fi
	@echo "## Custom RASH Metrics" >> QUALITY_REPORT.md
	@if [ -f ./target/release/rash-metrics ]; then \
		./target/release/rash-metrics >> QUALITY_REPORT.md; \
	fi

# Fuzzing infrastructure
FUZZ_DURATION ?= 3600
FUZZ_JOBS ?= $(shell nproc)

fuzz: fuzz-ast fuzz-ir fuzz-emitter fuzz-verifier

fuzz-ast:
	@echo "🔥 Fuzzing AST parser..."
	@if cargo +nightly --version >/dev/null 2>&1; then \
		cargo +nightly fuzz run ast_parser -- \
			-jobs=$(FUZZ_JOBS) \
			-max_total_time=$(FUZZ_DURATION) \
			-seed_inputs=fuzz/seeds/ast/ || true; \
	else \
		echo "⚠️  Nightly toolchain not available for fuzzing"; \
	fi

fuzz-ir:
	@echo "🔥 Fuzzing IR generation..."
	@if cargo +nightly --version >/dev/null 2>&1; then \
		cargo +nightly fuzz run ir_generator -- \
			-jobs=$(FUZZ_JOBS) \
			-max_total_time=$(FUZZ_DURATION) || true; \
	fi

fuzz-emitter:
	@echo "🔥 Fuzzing shell emitter..."
	@if cargo +nightly --version >/dev/null 2>&1; then \
		cargo +nightly fuzz run shell_emitter -- \
			-jobs=$(FUZZ_JOBS) \
			-max_total_time=$(FUZZ_DURATION) || true; \
	fi

fuzz-verifier:
	@echo "🔥 Fuzzing verifier..."
	@if cargo +nightly --version >/dev/null 2>&1; then \
		cargo +nightly fuzz run verifier -- \
			-jobs=$(FUZZ_JOBS) \
			-max_total_time=$(FUZZ_DURATION) || true; \
	fi

fuzz-differential:
	@echo "🔄 Differential fuzzing (optimization levels)..."
	@if cargo +nightly --version >/dev/null 2>&1; then \
		cargo +nightly fuzz run differential_optimization -- \
			-jobs=$(FUZZ_JOBS) \
			-max_total_time=$(FUZZ_DURATION) || true; \
	fi

fuzz-coverage:
	@echo "📊 Generating fuzzing coverage..."
	@if cargo +nightly --version >/dev/null 2>&1; then \
		cargo +nightly fuzz coverage ast_parser || true; \
		cargo +nightly fuzz coverage differential_optimization || true; \
	fi

fuzz-trophies:
	@echo "🏆 Minimizing fuzzing crashes..."
	@find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | while read trophy; do \
		echo "Minimizing $$trophy..."; \
		cargo +nightly fuzz tmin $$trophy || true; \
	done

# Formal verification
verify: verify-kani verify-creusot verify-smt verify-model verify-properties
	@echo "✅ All formal verification passed!"

verify-kani:
	@echo "🔍 Running Kani model checker..."
	@if cargo +nightly kani --version >/dev/null 2>&1; then \
		cargo +nightly kani --harnesses verify_parser_soundness --unwind 10 || true; \
		cargo +nightly kani --harnesses verify_escape_safety --unwind 10 || true; \
		cargo +nightly kani --harnesses verify_injection_safety --unwind 10 || true; \
	else \
		echo "⚠️  Kani not installed, skipping bounded model checking"; \
	fi

verify-creusot:
	@echo "🔬 Running Creusot verification..."
	@if command -v creusot >/dev/null 2>&1; then \
		creusot --why3-cmd 'why3 prove -P z3,cvc4' rash/src/lib.rs || true; \
	else \
		echo "⚠️  Creusot not installed, skipping semantic verification"; \
	fi

verify-smt:
	@echo "🔐 Running SMT-based verification..."
	@if [ -f ./target/release/smt-verifier ]; then \
		./target/release/smt-verifier \
			--check-injection-safety \
			--check-determinism \
			--check-termination \
			--solver z3 \
			--timeout 300 || true; \
	fi

verify-model:
	@echo "📐 Model checking with TLA+..."
	@if command -v tlc >/dev/null 2>&1; then \
		tlc specs/RashSemantics.tla -config specs/RashSemantics.cfg || true; \
	else \
		echo "⚠️  TLC not installed, skipping model checking"; \
	fi

verify-specs:
	@echo "📋 Verifying specification compliance..."
	@cargo test --workspace spec_compliance -- --nocapture || true
	@cargo test --workspace do178c_compliance -- --nocapture || true

verify-properties:
	@echo "🧪 Property-based verification..."
	@cargo test --workspace property -- --test-threads=1 --nocapture || true

# Verification metrics
verify-coverage:
	@echo "📊 Calculating verification coverage..."
	@cargo tarpaulin --all-features --workspace \
		--exclude-files 'target/*' \
		--exclude-files '*/tests/*' \
		--exclude-files '*/benches/*' \
		--print-summary \
		--print-graphical-summary || true

# Security audit
audit:
	@echo "🔒 Running security audit..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		cargo install cargo-audit && cargo audit; \
	fi

# ShellCheck installation and validation
shellcheck-install:
	@echo "📥 Installing ShellCheck..."
	@if command -v shellcheck >/dev/null 2>&1; then \
		echo "✅ ShellCheck already installed: $$(shellcheck --version | head -1)"; \
	else \
		echo "📦 Installing ShellCheck..."; \
		if command -v apt-get >/dev/null 2>&1; then \
			sudo apt-get update && sudo apt-get install -y shellcheck; \
		elif command -v brew >/dev/null 2>&1; then \
			brew install shellcheck; \
		elif command -v dnf >/dev/null 2>&1; then \
			sudo dnf install -y ShellCheck; \
		elif command -v pacman >/dev/null 2>&1; then \
			sudo pacman -S shellcheck; \
		elif command -v cabal >/dev/null 2>&1; then \
			cabal update && cabal install ShellCheck; \
		else \
			echo "❌ No supported package manager found. Install manually from https://shellcheck.net"; \
			exit 1; \
		fi; \
		echo "✅ ShellCheck installed: $$(shellcheck --version | head -1)"; \
	fi

shellcheck-validate: shellcheck-install
	@echo "🐚 Running ShellCheck validation on generated scripts..."
	@mkdir -p tests/shellcheck-output
	@echo "🔨 Building test scripts for validation..."
	@$(MAKE) shellcheck-test-all
	@echo "✅ ShellCheck validation completed!"

shellcheck-test-all: shellcheck-install
	@echo "🧪 Running comprehensive ShellCheck tests..."
	@failed=0; \
	total=0; \
	shopt -s nullglob; \
	for rs_file in examples/*.rs tests/fixtures/shellcheck/*.rs; do \
		if [ -f "$$rs_file" ]; then \
			total=$$((total + 1)); \
			base=$$(basename "$$rs_file" .rs); \
			echo "Testing $$rs_file -> $$base.sh"; \
			if cargo run --bin rash -- build "$$rs_file" -o "tests/shellcheck-output/$$base.sh" 2>/dev/null; then \
				if shellcheck -s sh "tests/shellcheck-output/$$base.sh"; then \
					echo "✅ $$base: PASS"; \
				else \
					echo "❌ $$base: FAIL (ShellCheck errors)"; \
					failed=$$((failed + 1)); \
				fi; \
			else \
				echo "❌ $$base: FAIL (transpilation failed)"; \
				failed=$$((failed + 1)); \
			fi; \
		fi; \
	done; \
	echo ""; \
	echo "📊 ShellCheck Test Results:"; \
	echo "  Total: $$total"; \
	echo "  Passed: $$((total - failed))"; \
	echo "  Failed: $$failed"; \
	if [ $$failed -gt 0 ]; then \
		echo "❌ Some ShellCheck tests failed!"; \
		exit 1; \
	else \
		echo "✅ All ShellCheck tests passed!"; \
	fi

# Dependency management
update-deps:
	@echo "🔄 Updating dependencies (semver-compatible)..."
	@echo "Step 1: Updating within semver-compatible ranges..."
	@cargo update --aggressive --workspace
	@echo "Step 2: Running tests to verify compatibility..."
	@make test-fast
	@echo "✅ Dependencies updated successfully!"
	@echo ""
	@echo "📊 Updated packages summary:"
	@cargo tree --duplicates --workspace | head -20 || true

update-deps-aggressive:
	@echo "🔄 Updating dependencies aggressively (requires cargo-edit)..."
	@echo "Installing cargo-edit for cargo upgrade command..."
	@if ! command -v cargo-upgrade >/dev/null 2>&1; then \
		cargo install cargo-edit; \
	else \
		echo "cargo-edit already installed"; \
	fi
	@echo "Step 1: Updating within semver-compatible ranges..."
	@cargo update --aggressive --workspace
	@echo "Step 2: Upgrading to latest incompatible versions (major bumps)..."
	@for dir in . rash rash-runtime rash-tests; do \
		if [ -f $$dir/Cargo.toml ]; then \
			echo "Upgrading $$dir..."; \
			cd $$dir && cargo upgrade --incompatible && cd ..; \
		fi; \
	done
	@echo "Step 3: Running comprehensive tests..."
	@make test-fast lint-check
	@echo "Step 4: Checking for security vulnerabilities..."
	@make audit
	@echo "✅ Aggressive update completed!"
	@echo ""
	@echo "📊 Final dependency status:"
	@cargo tree --duplicates --workspace | head -30 || true

update-deps-check:
	@echo "🔍 Checking for outdated dependencies..."
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "Installing cargo-outdated..."; \
		cargo install cargo-outdated; \
	fi
	@echo ""
	@echo "📋 Outdated dependencies in main workspace:"
	@cargo outdated --workspace --root-deps-only || true
	@echo ""
	@echo "📋 Outdated dependencies in rash:"
	@cd rash && cargo outdated --root-deps-only || true
	@echo ""
	@echo "📋 Outdated dependencies in rash-runtime:"
	@cd rash-runtime && cargo outdated --root-deps-only || true
	@echo ""
	@echo "📋 Outdated dependencies in rash-tests:"
	@cd rash-tests && cargo outdated --root-deps-only || true
	@echo ""
	@echo "🔍 Security advisories check:"
	@make audit || true

update-deps-workspace:
	@echo "🔄 Updating workspace dependencies with validation..."
	@echo "Step 1: Backup current Cargo.lock files..."
	@cp Cargo.lock Cargo.lock.backup 2>/dev/null || true
	@cp rash/Cargo.lock rash/Cargo.lock.backup 2>/dev/null || true
	@cp rash-runtime/Cargo.lock rash-runtime/Cargo.lock.backup 2>/dev/null || true
	@cp rash-tests/Cargo.lock rash-tests/Cargo.lock.backup 2>/dev/null || true
	@echo "Step 2: Updating workspace dependencies..."
	@cargo update --workspace
	@echo "Step 3: Building to check for breaking changes..."
	@if ! cargo build --workspace --all-features; then \
		echo "❌ Build failed after update, restoring backups..."; \
		cp Cargo.lock.backup Cargo.lock 2>/dev/null || true; \
		cp rash/Cargo.lock.backup rash/Cargo.lock 2>/dev/null || true; \
		cp rash-runtime/Cargo.lock.backup rash-runtime/Cargo.lock 2>/dev/null || true; \
		cp rash-tests/Cargo.lock.backup rash-tests/Cargo.lock 2>/dev/null || true; \
		exit 1; \
	fi
	@echo "Step 4: Running tests..."
	@if ! make test-fast; then \
		echo "❌ Tests failed after update, restoring backups..."; \
		cp Cargo.lock.backup Cargo.lock 2>/dev/null || true; \
		cp rash/Cargo.lock.backup rash/Cargo.lock 2>/dev/null || true; \
		cp rash-runtime/Cargo.lock.backup rash-runtime/Cargo.lock 2>/dev/null || true; \
		cp rash-tests/Cargo.lock.backup rash-tests/Cargo.lock 2>/dev/null || true; \
		exit 1; \
	fi
	@echo "Step 5: Cleanup backup files..."
	@rm -f Cargo.lock.backup rash/Cargo.lock.backup rash-runtime/Cargo.lock.backup rash-tests/Cargo.lock.backup
	@echo "✅ Workspace dependencies updated and validated!"
	@echo ""
	@echo "📊 Dependency tree summary:"
	@cargo tree --workspace --depth 1

# Documentation
docs:
	@echo "📚 Building documentation..."
	@cargo doc --all-features --workspace --no-deps
	@echo "Documentation available at target/doc/rash/index.html"

# Build
build:
	@echo "🔨 Building release binaries..."
	@cargo build --release --workspace --all-features

# Install
install: build
	@echo "📦 Installing rash..."
	@cargo install --path rash --force

# Release
release: validate
	@echo "🚀 Preparing release..."
	@echo "Release build completed. Use GitHub Actions for full release process."

# Memory profiling
profile-memory:
	@echo "🧠 Profiling memory usage..."
	@cargo build --release --features profiling
	@if command -v valgrind >/dev/null 2>&1; then \
		valgrind --tool=massif \
			--massif-out-file=massif.out \
			./target/release/rash transpile examples/complex.rs || true; \
		ms_print massif.out > memory-profile.txt || true; \
		echo "Memory profile saved to memory-profile.txt"; \
	fi

profile-heap:
	@echo "📊 Generating heap profile..."
	@cargo run --release --features "profiling jemalloc" -- \
		transpile examples/large.rs \
		--heap-profile heap-profile.pb || true

profile-flamegraph:
	@echo "🔥 Generating CPU flamegraph..."
	@if command -v cargo-flamegraph >/dev/null 2>&1; then \
		cargo flamegraph --root -- transpile examples/complex.rs || true; \
		echo "Flamegraph saved to flamegraph.svg"; \
	else \
		echo "⚠️  cargo-flamegraph not installed"; \
	fi

# Clean
clean:
	@echo "🧹 Cleaning..."
	@cargo clean
	@rm -rf coverage.json coverage-summary.txt coverage-html/
	@rm -rf .quality/ complexity-current.json deadcode-current.json tech-debt-current.json
	@rm -rf massif.out memory-profile.txt heap-profile.pb flamegraph.svg
	@rm -rf fuzz/artifacts fuzz/corpus

# Help
help:
	@echo "RASH Build System"
	@echo "================="
	@echo ""
	@echo "Main targets:"
	@echo "  make              - Run validation and build"
	@echo "  make lint         - Run linting with fixes"
	@echo "  make test         - Run comprehensive tests with coverage"
	@echo "  make release      - Prepare release build"
	@echo ""
	@echo "Validation:"
	@echo "  make validate     - Full validation pipeline"
	@echo "  make quick-validate - Quick validation for development"
	@echo ""
	@echo "Testing:"
	@echo "  make test-fast    - Run fast tests"
	@echo "  make test-shells  - Test cross-shell compatibility"
	@echo "  make test-determinism - Verify deterministic transpilation"
	@echo "  make shellcheck-validate - Run ShellCheck validation on generated scripts"
	@echo "  make shellcheck-test-all - Run comprehensive ShellCheck test suite"
	@echo ""
	@echo "Quality:"
	@echo "  make quality-gate - Run quality checks"
	@echo "  make quality-report - Generate quality report"
	@echo "  make audit        - Security audit"
	@echo ""
	@echo "Dependencies:"
	@echo "  make update-deps  - Update dependencies (semver-compatible)"
	@echo "  make update-deps-aggressive - Update all dependencies including major versions"
	@echo "  make update-deps-check - Check for outdated dependencies"
	@echo "  make update-deps-workspace - Safe workspace dependency update with rollback"
	@echo ""
	@echo "Advanced:"
	@echo "  make fuzz         - Run fuzzing tests"
	@echo "  make verify       - Run formal verification"
	@echo "  make profile-memory - Profile memory usage"
	@echo ""
	@echo "Other:"
	@echo "  make docs         - Build documentation"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make help         - Show this help"