#!/bin/bash
# Pre-commit quality gates for bashrs
# EXTREME TDD - Zero Tolerance for Quality Violations
# This script enforces all quality gates defined in .pmat-gates.toml
#
# Installation:
#   ln -sf ../../scripts/hooks/pre-commit.sh .git/hooks/pre-commit
#
# Or run directly:
#   ./scripts/hooks/pre-commit.sh

set -e

echo "🔍 bashrs Pre-commit Quality Gates"
echo "===================================="
echo ""

# Configuration from .pmat-gates.toml
MAX_CYCLOMATIC_COMPLEXITY=10
MAX_COGNITIVE_COMPLEXITY=15
MIN_TEST_COVERAGE=85
SATD_ZERO_TOLERANCE=true
CLIPPY_STRICT=true

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILURES=0

echo "📊 Quality Gate Checks"
echo "----------------------"

# Gate 1: Clippy (BLOCKING - Zero Warnings)
echo -n "  1. Clippy (zero warnings)... "
if cargo clippy --lib -- -D warnings >/dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌${NC}"
    echo "     Run: cargo clippy --lib -- -D warnings"
    echo "     Fix: cargo clippy --lib --fix --allow-dirty"
    FAILURES=$((FAILURES + 1))
fi

# Gate 2: Performance Lints (BLOCKING)
echo -n "  2. Performance lints... "
if cargo clippy --release --lib -- -W clippy::perf >/dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌${NC}"
    echo "     Run: cargo clippy --release --lib -- -W clippy::perf"
    FAILURES=$((FAILURES + 1))
fi

# Gate 3: Test Suite (BLOCKING)
echo -n "  3. Test suite... "
if cargo test --lib >/dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌${NC}"
    echo "     Run: cargo test --lib"
    FAILURES=$((FAILURES + 1))
fi

# Gate 4: Code Complexity (WARNING - track progress, don't block)
echo -n "  4. Code complexity (max: $MAX_CYCLOMATIC_COMPLEXITY)... "
if command -v pmat &> /dev/null; then
    COMPLEXITY_OUTPUT=$(pmat analyze complexity --max-cyclomatic $MAX_CYCLOMATIC_COMPLEXITY --max-cognitive $MAX_COGNITIVE_COMPLEXITY 2>&1 || true)
    ERROR_COUNT=$(echo "$COMPLEXITY_OUTPUT" | grep -oP 'Errors: \K\d+' || echo "0")

    if [ "$ERROR_COUNT" -eq 0 ]; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${YELLOW}⚠️  ($ERROR_COUNT functions >$MAX_CYCLOMATIC_COMPLEXITY)${NC}"
        echo "     Goal: Refactor high-complexity functions with EXTREME TDD"
        echo "     Run: pmat analyze complexity"
        # NOTE: This is a WARNING, not blocking. We track progress toward goal.
    fi
else
    echo -e "${YELLOW}⚠️  (pmat not installed)${NC}"
fi

# Gate 5: SATD (WARNING - track critical/high issues in production code)
echo -n "  5. Technical debt (zero tolerance goal)... "
if command -v pmat &> /dev/null; then
    SATD_OUTPUT=$(pmat analyze satd --path rash/src 2>&1 || true)
    CRITICAL_COUNT=$(echo "$SATD_OUTPUT" | grep -c "Critical" || echo "0")
    HIGH_COUNT=$(echo "$SATD_OUTPUT" | grep -c "High" || echo "0")
    TOTAL_COUNT=$(echo "$SATD_OUTPUT" | grep -oP 'Found \K\d+' | head -1 || echo "0")

    if [ "$TOTAL_COUNT" -eq 0 ]; then
        echo -e "${GREEN}✅${NC}"
    elif [ "$CRITICAL_COUNT" -gt 0 ] || [ "$HIGH_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}⚠️  ($CRITICAL_COUNT critical, $HIGH_COUNT high)${NC}"
        echo "     Production code has Critical/High SATD - should fix soon"
        echo "     Run: pmat analyze satd --path rash/src"
        # NOTE: WARNING level - encourages fixing but doesn't block
    else
        echo -e "${YELLOW}⚠️  ($TOTAL_COUNT total)${NC}"
        echo "     Working toward zero SATD goal"
    fi
else
    echo -e "${YELLOW}⚠️  (pmat not installed)${NC}"
fi

# Gate 6: Formatting (BLOCKING)
echo -n "  6. Code formatting... "
if cargo fmt -- --check >/dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌${NC}"
    echo "     Run: cargo fmt"
    FAILURES=$((FAILURES + 1))
fi

# Gate 7: Documentation (WARNING - check files exist)
echo -n "  7. Documentation sync... "
if [ -f "CHANGELOG.md" ] && [ -f "README.md" ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️  (missing files)${NC}"
fi

echo ""
echo "Quality Gate Summary"
echo "--------------------"

if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}✅ All quality gates passed!${NC}"
    echo ""
    echo "Commit is ready to proceed."
    exit 0
else
    echo -e "${RED}❌ $FAILURES quality gate(s) failed${NC}"
    echo ""
    echo "Please fix the issues above before committing."
    echo "To bypass (EMERGENCY ONLY): git commit --no-verify"
    echo ""
    exit 1
fi
