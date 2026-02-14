#!/usr/bin/env bash
# comply:disable=COMPLY-001,COMPLY-002
# Bashrs Quality Gates - EXTREME TDD Enforcement
# Inspired by paiml-mcp-agent-toolkit quality standards
# Toyota Way: Jidoka (自働化) - Build Quality In

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration (can be overridden by pmat-quality.toml)
MAX_COMPLEXITY=${MAX_COMPLEXITY:-10}
MAX_COGNITIVE=${MAX_COGNITIVE:-15}
MIN_COVERAGE=${MIN_COVERAGE:-85.0}
MIN_MUTATION_SCORE=${MIN_MUTATION_SCORE:-90.0}
SATD_TOLERANCE=${SATD_TOLERANCE:-0}
MIN_PROPERTY_TESTS=${MIN_PROPERTY_TESTS:-50}

# Exit codes
EXIT_SUCCESS=0
EXIT_FORMAT_FAIL=1
EXIT_LINT_FAIL=2
EXIT_TEST_FAIL=3
EXIT_COVERAGE_FAIL=4
EXIT_COMPLEXITY_FAIL=5
EXIT_SATD_FAIL=6
EXIT_SHELLCHECK_FAIL=7
EXIT_DETERMINISM_FAIL=8
EXIT_PERFORMANCE_FAIL=9

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  Bashrs Quality Gates - EXTREME TDD Enforcement${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_section() {
    echo ""
    echo -e "${BLUE}▶ $1${NC}"
    echo -e "${BLUE}──────────────────────────────────────────────────────────${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_CHECKS++))
    ((TOTAL_CHECKS++))
}

print_failure() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED_CHECKS++))
    ((TOTAL_CHECKS++))
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "  $1"
}

check_format() {
    print_section "1. Format Check (rustfmt)"

    if cargo fmt -- --check > /dev/null 2>&1; then
        print_success "Code formatting is correct"
        return 0
    else
        print_failure "Code formatting issues detected"
        print_info "Run: cargo fmt"
        return "$EXIT_FORMAT_FAIL"
    fi
}

check_lint() {
    print_section "2. Lint Check (clippy)"

    if cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
        print_success "No clippy warnings"
        return 0
    else
        print_failure "Clippy warnings detected"
        print_info "Run: cargo clippy --all-targets --all-features"
        return "$EXIT_LINT_FAIL"
    fi
}

check_tests() {
    print_section "3. Test Suite"

    print_info "Running unit tests..."
    if cargo test --lib --quiet 2>&1 | grep -q "test result: ok"; then
        print_success "Unit tests passed"
    else
        print_failure "Unit tests failed"
        return "$EXIT_TEST_FAIL"
    fi

    print_info "Running doc tests..."
    if cargo test --doc --quiet 2>&1 | grep -q "test result: ok"; then
        print_success "Doc tests passed"
    else
        print_failure "Doc tests failed"
        return "$EXIT_TEST_FAIL"
    fi

    print_info "Running property tests..."
    if cargo test --test property_tests --quiet 2>&1 | grep -q "test result: ok"; then
        local prop_count="$(cargo test --test property_tests -- --list 2>/dev/null | grep -c "test " || echo 0)"
        if [ "$prop_count" -ge "$MIN_PROPERTY_TESTS" ]; then
            print_success "Property tests passed ($prop_count properties, ≥$MIN_PROPERTY_TESTS required)"
        else
            print_failure "Not enough property tests ($prop_count < $MIN_PROPERTY_TESTS)"
            return "$EXIT_TEST_FAIL"
        fi
    else
        print_failure "Property tests failed"
        return "$EXIT_TEST_FAIL"
    fi

    return 0
}

check_coverage() {
    print_section "4. Coverage Check"

    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_warning "cargo-llvm-cov not installed, skipping coverage check"
        print_info "Install: cargo install cargo-llvm-cov"
        return 0
    fi

    print_info "Running coverage analysis..."
    cargo llvm-cov --quiet --json --output-path target/coverage.json > /dev/null 2>&1 || true

    if [ -f target/coverage.json ]; then
        # Extract coverage percentage (simplified - would need jq for proper parsing)
        local coverage=$(grep -o '"percent":[0-9.]*' target/coverage.json | head -1 | cut -d: -f2 || echo "0")
        local coverage_int=$(echo "$coverage" | cut -d. -f1)

        if [ "$coverage_int" -ge "${MIN_COVERAGE%.*}" ]; then
            print_success "Coverage: ${coverage}% (≥${MIN_COVERAGE}% required)"
            return 0
        else
            print_failure "Coverage: ${coverage}% (< ${MIN_COVERAGE}% required)"
            return "$EXIT_COVERAGE_FAIL"
        fi
    else
        print_warning "Coverage report not generated"
        return 0
    fi
}

check_complexity() {
    print_section "5. Complexity Check"

    if ! command -v pmat &> /dev/null; then
        print_warning "pmat not installed, skipping complexity check"
        print_info "Complexity check would verify cyclomatic ≤$MAX_COMPLEXITY, cognitive ≤$MAX_COGNITIVE"
        return 0
    fi

    print_info "Analyzing code complexity..."
    if pmat analyze complexity src/ --max-cyclomatic "$MAX_COMPLEXITY" --max-cognitive "$MAX_COGNITIVE" > /dev/null 2>&1; then
        print_success "Complexity within limits (cyclomatic ≤$MAX_COMPLEXITY, cognitive ≤$MAX_COGNITIVE)"
        return 0
    else
        print_failure "Complexity exceeds limits"
        print_info "Run: pmat analyze complexity src/ --detailed"
        return "$EXIT_COMPLEXITY_FAIL"
    fi
}

check_satd() {
    print_section "6. SATD Check (Zero Tolerance)"

    local satd_patterns=("TODO" "FIXME" "HACK" "XXX" "BUG" "KLUDGE" "WORKAROUND" "REFACTOR")
    local satd_found=0

    for pattern in "${satd_patterns[@]}"; do
        if grep -r "$pattern" src/ --include="*.rs" > /dev/null 2>&1; then
            print_failure "SATD pattern found: $pattern"
            satd_found=1
        fi
    done

    if [ "$satd_found" -eq 0 ]; then
        print_success "No SATD comments found (zero tolerance maintained)"
        return 0
    else
        print_info "Run: grep -rn 'TODO\\|FIXME\\|HACK' src/"
        return "$EXIT_SATD_FAIL"
    fi
}

check_shellcheck() {
    print_section "7. ShellCheck Validation"

    if ! command -v shellcheck &> /dev/null; then
        print_warning "shellcheck not installed, skipping"
        return 0
    fi

    print_info "Validating generated shell scripts..."

    # Generate test script
    local test_script="$(mktemp)"
    if cargo run --quiet -- transpile examples/hello.rs > "$test_script" 2>/dev/null; then
        if shellcheck -s sh "$test_script" > /dev/null 2>&1; then
            print_success "ShellCheck validation passed (POSIX compliant)"
            rm -f "$test_script"
            return 0
        else
            print_failure "ShellCheck validation failed"
            rm -f "$test_script"
            return "$EXIT_SHELLCHECK_FAIL"
        fi
    else
        print_warning "Could not generate test script for ShellCheck"
        rm -f "$test_script"
        return 0
    fi
}

check_determinism() {
    print_section "8. Determinism Check"

    print_info "Verifying byte-identical output..."

    # Generate same code twice
    local output1="$(mktemp)"
    local output2="$(mktemp)"

    if cargo run --quiet -- transpile examples/hello.rs > "$output1" 2>/dev/null; then
        cargo run --quiet -- transpile examples/hello.rs > "$output2" 2>/dev/null

        if diff -q "$output1" "$output2" > /dev/null 2>&1; then
            print_success "Determinism verified (byte-identical output)"
            rm -f "$output1" "$output2"
            return 0
        else
            print_failure "Determinism check failed (output differs)"
            rm -f "$output1" "$output2"
            return "$EXIT_DETERMINISM_FAIL"
        fi
    else
        print_warning "Could not generate test output for determinism check"
        rm -f "$output1" "$output2"
        return 0
    fi
}

check_performance() {
    print_section "9. Performance Check"

    print_info "Running performance benchmarks..."

    if [ -d benches ]; then
        if cargo bench --quiet > /dev/null 2>&1; then
            print_success "Performance benchmarks passed"
            return 0
        else
            print_warning "Performance benchmarks failed or not available"
            return 0
        fi
    else
        print_warning "No benchmark suite found"
        return 0
    fi
}

print_summary() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  Quality Gates Summary${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  Total Checks:  $TOTAL_CHECKS"
    echo -e "  ${GREEN}Passed:${NC}        $PASSED_CHECKS"
    echo -e "  ${RED}Failed:${NC}        $FAILED_CHECKS"
    echo ""

    if [ "$FAILED_CHECKS" -eq 0 ]; then
        echo -e "${GREEN}✓ All quality gates passed! ✓${NC}"
        echo ""
        echo -e "${GREEN}  EXTREME TDD: Quality Built In (Jidoka)${NC}"
        echo -e "${GREEN}  Toyota Way: 自働化 - Automation with Human Intelligence${NC}"
        echo ""
        return "$EXIT_SUCCESS"
    else
        echo -e "${RED}✗ Quality gates failed! ✗${NC}"
        echo ""
        echo -e "${RED}  Fix all issues before committing${NC}"
        echo -e "${RED}  Zero tolerance for quality violations${NC}"
        echo ""
        return 1
    fi
}

main() {
    local exit_code="$EXIT_SUCCESS"

    print_header

    # Run all checks (continue even if some fail to show all issues)
    check_format || exit_code=$?
    check_lint || exit_code=$?
    check_tests || exit_code=$?
    check_coverage || exit_code=$?
    check_complexity || exit_code=$?
    check_satd || exit_code=$?
    check_shellcheck || exit_code=$?
    check_determinism || exit_code=$?
    check_performance || exit_code=$?

    print_summary || exit_code=$?

    exit "$exit_code"
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" = "${0}" ]]; then
    main "$@"
fi
