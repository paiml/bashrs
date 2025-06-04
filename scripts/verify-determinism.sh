#!/usr/bin/env bash
# Verify deterministic transpilation

set -euo pipefail

echo "🎯 Verifying deterministic transpilation..."

# Create test cases
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

# Generate test Rust files
cat > "$TEST_DIR/test1.rs" << 'EOF'
fn main() {
    let x = 42;
    echo(x);
}

fn echo(msg: u32) {}
EOF

cat > "$TEST_DIR/test2.rs" << 'EOF'
fn main() {
    let message = "Hello World";
    let count = 3;
    echo(message);
    process_count(count);
}

fn echo(msg: &str) {}
fn process_count(n: u32) {}
EOF

# Check if rash binary exists
RASH_BIN="./target/release/rash"
if [ ! -f "$RASH_BIN" ]; then
    echo "Building rash..."
    cargo build --release --bin rash
fi

# Test each file multiple times
for test_file in "$TEST_DIR"/*.rs; do
    echo "Testing $(basename "$test_file")..."
    
    # Run transpilation multiple times
    for i in {1..10}; do
        "$RASH_BIN" build "$test_file" \
            --output "$TEST_DIR/output_$i.sh" 2>/dev/null || true
    done
    
    # Verify all outputs are identical
    for i in {2..10}; do
        if ! diff -q "$TEST_DIR/output_1.sh" "$TEST_DIR/output_$i.sh" >/dev/null 2>&1; then
            echo "❌ Non-deterministic output detected for $(basename "$test_file")!"
            echo "Difference between run 1 and run $i:"
            diff "$TEST_DIR/output_1.sh" "$TEST_DIR/output_$i.sh" || true
            exit 1
        fi
    done
    
    # Clean up outputs for next test
    rm -f "$TEST_DIR"/output_*.sh
done

echo "✅ Transpilation is deterministic across 10 runs for all test cases"