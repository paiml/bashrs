#!/bin/sh
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
echo "  1. Format (cargo fmt --check) - BLOCKS"
echo "  2. Clippy, perf lints (WARN only; CI enforces)"
echo "  3. Fast tests (WARN only; CI enforces)"
echo ""
echo "To bypass hooks temporarily (NOT RECOMMENDED):"
echo "  git commit --no-verify"
echo ""
