#!/bin/bash
# Install bashrs git hooks
# Run this script to set up pre-commit quality gates

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="${REPO_ROOT}/.git/hooks"
SOURCE_DIR="${REPO_ROOT}/scripts/hooks"

echo "🔧 Installing bashrs git hooks..."

# Install pre-commit hook
if [ -f "${SOURCE_DIR}/pre-commit" ]; then
    cp "${SOURCE_DIR}/pre-commit" "${HOOKS_DIR}/pre-commit"
    chmod +x "${HOOKS_DIR}/pre-commit"
    echo "  ✅ Installed pre-commit hook"
else
    echo "  ❌ Error: pre-commit hook not found at ${SOURCE_DIR}/pre-commit"
    exit 1
fi

echo ""
echo "✅ Git hooks installed successfully"
echo ""
echo "Pre-commit hook enforces:"
echo "  1. Zero clippy warnings (cargo clippy --lib -- -D warnings)"
echo "  2. Performance checks (cargo clippy --release -- -W clippy::perf)"
echo "  3. All tests passing (cargo test --lib)"
echo ""
echo "To bypass hooks temporarily (NOT RECOMMENDED):"
echo "  git commit --no-verify"
echo ""
