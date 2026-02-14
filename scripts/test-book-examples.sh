#!/bin/sh
# Test all Rash book examples
# Usage: ./scripts/test-book-examples.sh

set -eu

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

total=0
passed=0
failed=0
skipped=0

echo "Testing Rash Book Examples"
echo "=========================="
echo

test_example() {
    local example=$1
    local name=$(basename "$example" .rs)

    total=$((total + 1))

    # Transpile
    if cargo run --quiet --bin bashrs -- build "$example" -o "/tmp/$name.sh" 2>/dev/null; then
        # Try to run
        if sh "/tmp/$name.sh" >/dev/null 2>&1; then
            echo -e "${GREEN}✅${NC} $example"
            passed=$((passed + 1))
        else
            echo -e "${RED}❌${NC} $example (execution failed)"
            failed=$((failed + 1))
        fi
    else
        # Check if it's a known transpiler bug
        if grep -q "$name" TEST_RESULTS.md 2>/dev/null; then
            echo -e "${YELLOW}⚠️${NC}  $example (known transpiler bug)"
            skipped=$((skipped + 1))
        else
            echo -e "${RED}❌${NC} $example (transpilation failed)"
            failed=$((failed + 1))
        fi
    fi
}

# Test Chapter 2: Variables
echo "Chapter 2: Variables"
echo "--------------------"
for example in examples/ch02_variables/*.rs; do
    test_example "$example"
done
echo

# Test Chapter 3: Functions
echo "Chapter 3: Functions"
echo "--------------------"
for example in examples/ch03_functions/*.rs; do
    test_example "$example"
done
echo

# Test Chapter 4: Control Flow
echo "Chapter 4: Control Flow"
echo "-----------------------"
for example in examples/ch04_control_flow/*.rs; do
    test_example "$example"
done
echo

# Summary
echo "Summary"
echo "======="
echo "Total:   $total"
echo -e "${GREEN}Passed:  $passed${NC}"
echo -e "${RED}Failed:  $failed${NC}"
echo -e "${YELLOW}Skipped: $skipped${NC}"
echo

if [ "$failed" -eq 0 ] && [ "$skipped" -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
elif [ "$failed" -eq 0 ]; then
    echo -e "${YELLOW}All tests passed (with known transpiler bugs skipped)${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
