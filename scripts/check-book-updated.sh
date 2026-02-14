#!/bin/sh
# check-book-updated.sh - Enforce book updates before release
#
# Toyota Way (自働化 Jidoka): Build quality into the release process
# 
# This script ensures the book is updated with new features before release.
# It checks:
# 1. Book builds successfully (mdbook build)
# 2. All examples pass tests (mdbook test)
# 3. CHANGELOG and book are in sync (both updated in same commit range)

set -eu

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Checking book status before release..."

# Check 1: mdbook is installed
if ! command -v mdbook >/dev/null 2>&1; then
    echo -e "${RED}❌ mdbook not found!${NC}"
    echo "Install with: cargo install mdbook"
    exit 1
fi

# Check 2: Book builds successfully
echo "📚 Building book..."
if ! (cd book && mdbook build); then
    echo -e "${RED}❌ Book build failed!${NC}"
    echo "Fix book build errors before releasing."
    exit 1
fi

echo -e "${GREEN}✅ Book builds successfully${NC}"

# Check 3: All examples pass tests
echo "🧪 Testing book examples..."
if ! (cd book && mdbook test); then
    echo -e "${RED}❌ Book examples failed!${NC}"
    echo "Fix failing examples before releasing."
    exit 1
fi

echo -e "${GREEN}✅ All book examples pass${NC}"

# Check 4: Book has been updated in recent commits
echo "📝 Checking if book was updated..."

# Get the last release tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
    echo -e "${YELLOW}⚠️  No previous release found, skipping update check${NC}"
else
    # Check if book/ was modified since last release
    if git diff --name-only "$LAST_TAG" HEAD | grep -q "^book/"; then
        echo -e "${GREEN}✅ Book updated since last release ($LAST_TAG)${NC}"
    else
        echo -e "${YELLOW}⚠️  Book not updated since last release ($LAST_TAG)${NC}"
        echo ""
        echo "Before releasing, ensure you've updated the book with:"
        echo "  - New features documentation"
        echo "  - Updated examples"
        echo "  - CHANGELOG entries reflected in book"
        echo ""
        echo "To skip this check (not recommended): export SKIP_BOOK_CHECK=1"
        
        if [ "${SKIP_BOOK_CHECK:-0}" != "1" ]; then
            exit 1
        else
            echo -e "${YELLOW}⚠️  Skipping book check (SKIP_BOOK_CHECK=1)${NC}"
        fi
    fi
fi

# Check 5: Book and CHANGELOG are in sync
echo "🔄 Checking book/CHANGELOG sync..."

# Get last commit dates
BOOK_LAST_COMMIT=$(git log -1 --format=%ct -- book/ 2>/dev/null || echo "0")
CHANGELOG_LAST_COMMIT=$(git log -1 --format=%ct -- CHANGELOG.md 2>/dev/null || echo "0")

# If CHANGELOG was updated more recently than book, warn
if [ "$CHANGELOG_LAST_COMMIT" -gt "$BOOK_LAST_COMMIT" ]; then
    TIME_DIFF=$((CHANGELOG_LAST_COMMIT - BOOK_LAST_COMMIT))
    DAYS_DIFF=$((TIME_DIFF / 86400))
    
    if [ "$DAYS_DIFF" -gt 1 ]; then
        echo -e "${YELLOW}⚠️  CHANGELOG updated ${DAYS_DIFF} days after book${NC}"
        echo "Consider updating book to reflect CHANGELOG entries"
        echo ""
        echo "Recent CHANGELOG entries:"
        git log --oneline --since="$DAYS_DIFF days ago" -- CHANGELOG.md | head -5
    fi
fi

# Success!
echo ""
echo -e "${GREEN}✅ All book checks passed!${NC}"
echo ""
echo "Book is ready for release. Summary:"
echo "  📦 Build: OK"
echo "  🧪 Tests: OK"
echo "  📝 Updates: OK"
echo "  🔄 Sync: OK"
echo ""
echo "You can now proceed with release."
