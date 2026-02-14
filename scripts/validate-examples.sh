#!/bin/sh
# Test all examples to ensure they transpile successfully
# Part of v1.0 Phase 3: Performance & Polish

set -e

EXAMPLES_DIR="examples"
OUTPUT_DIR="/tmp/rash-example-tests"
FAILED=0
PASSED=0

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

mkdir -p "$OUTPUT_DIR"

echo "================================================"
echo "Rash v1.0 Example Validation"
echo "================================================"
echo ""

# Test individual .rs files in examples/
for example in "$EXAMPLES_DIR"/*.rs; do
    if [ -f "$example" ]; then
        filename=$(basename "$example" .rs)
        output="$OUTPUT_DIR/$filename.sh"

        printf "Testing %-30s ... " "$filename"

        if cargo run --quiet --bin bashrs -- build "$example" -o "$output" 2>/dev/null; then
            # Verify output exists and is non-empty
            if [ -s "$output" ]; then
                # Run basic shellcheck if available
                if command -v shellcheck >/dev/null 2>&1; then
                    if shellcheck -s sh "$output" >/dev/null 2>&1; then
                        printf "${GREEN}✓${NC} (transpiled + shellcheck passed)\n"
                        PASSED=$((PASSED + 1))
                    else
                        printf "${RED}✗${NC} (shellcheck failed)\n"
                        FAILED=$((FAILED + 1))
                    fi
                else
                    printf "${GREEN}✓${NC} (transpiled)\n"
                    PASSED=$((PASSED + 1))
                fi
            else
                printf "${RED}✗${NC} (empty output)\n"
                FAILED=$((FAILED + 1))
            fi
        else
            printf "${RED}✗${NC} (transpilation failed)\n"
            FAILED=$((FAILED + 1))
        fi
    fi
done

# Test subdirectory examples
for subdir in "$EXAMPLES_DIR"/*/; do
    if [ -d "$subdir" ]; then
        for example in "$subdir"/*.rs; do
            if [ -f "$example" ]; then
                filename=$(basename "$example" .rs)
                dirname=$(basename "$subdir")
                output="$OUTPUT_DIR/${dirname}_${filename}.sh"

                printf "Testing %-30s ... " "$dirname/$filename"

                if cargo run --quiet --bin bashrs -- build "$example" -o "$output" 2>/dev/null; then
                    if [ -s "$output" ]; then
                        if command -v shellcheck >/dev/null 2>&1; then
                            if shellcheck -s sh "$output" >/dev/null 2>&1; then
                                printf "${GREEN}✓${NC} (transpiled + shellcheck passed)\n"
                                PASSED=$((PASSED + 1))
                            else
                                printf "${RED}✗${NC} (shellcheck failed)\n"
                                FAILED=$((FAILED + 1))
                            fi
                        else
                            printf "${GREEN}✓${NC} (transpiled)\n"
                            PASSED=$((PASSED + 1))
                        fi
                    else
                        printf "${RED}✗${NC} (empty output)\n"
                        FAILED=$((FAILED + 1))
                    fi
                else
                    printf "${RED}✗${NC} (transpilation failed)\n"
                    FAILED=$((FAILED + 1))
                fi
            fi
        done
    fi
done

echo ""
echo "================================================"
echo "Results:"
echo "  Passed: $PASSED"
echo "  Failed: $FAILED"
echo "  Total:  $((PASSED + FAILED))"
echo "================================================"

if [ "$FAILED" -eq 0 ]; then
    echo ""
    echo "${GREEN}✅ All examples transpiled successfully!${NC}"
    exit 0
else
    echo ""
    echo "${RED}❌ Some examples failed. See above for details.${NC}"
    exit 1
fi
