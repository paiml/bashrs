#!/bin/sh
# Book Validation Pre-commit Hook
# Validates that all book examples compile successfully

set -e

echo "🔍 Validating book accuracy..."
echo ""

# Run book validation tests
if cargo test --test book_validation --quiet 2>&1 | grep -q "test result: ok"; then
    echo "✅ Book validation passed"
    echo ""
    exit 0
else
    echo "❌ Book validation failed!"
    echo ""
    echo "Please ensure:"
    echo "  1. All Rust code examples in Chapter 21+ compile successfully"
    echo "  2. Code blocks use correct language tags:"
    echo "     - Use \`\`\`rust for runnable Rust examples"
    echo "     - Use \`\`\`sh or \`\`\`bash for shell output"
    echo "     - Use \`\`\`ignore for non-compilable examples"
    echo "  3. Examples are complete programs with fn main() when needed"
    echo ""
    echo "To see detailed errors, run:"
    echo "  cargo test --test book_validation -- --nocapture"
    echo ""
    exit 1
fi
