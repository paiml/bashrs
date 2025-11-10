# Issue #2: Multi-line Format Preservation

## Summary

The `--preserve-formatting` and `--skip-consolidation` flags do not preserve backslash continuations in Makefile recipes.

## Status

**✅ RESOLVED** in v6.35.0
**Priority**: P2 (Enhancement)
**Affected**: v6.34.0
**Fixed**: v6.35.0 (2025-11-10)

## Problem Statement

When purifying Makefiles with multi-line recipes that use backslash continuations:

**Input:**
```makefile
build:
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release; \
	else \
		echo "cargo not found"; \
	fi
```

**Current Output (with --preserve-formatting):**
```makefile
build:
	@if command -v cargo >/dev/null 2>&1; then cargo build --release; else echo "cargo not found"; fi
```

**Expected Output:**
```makefile
build:
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release; \
	else \
		echo "cargo not found"; \
	fi
```

## Root Cause

The parser preprocesses backslash continuations **before** building the AST:

```rust
// rash/src/make_parser/parser.rs
pub fn parse_makefile(input: &str) -> Result<MakeAst, String> {
    let preprocessed = preprocess_line_continuations(input);
    // ... continues with preprocessed input
}

fn preprocess_line_continuations(input: &str) -> String {
    // Consolidates all backslash continuations into single lines
    // By the time AST is built, original structure is lost
}
```

By the time the generator receives the AST, the original line breaks have been removed.

## Impact

**Severity**: Low
**Workaround**: Use `--max-line-length` to intelligently break long lines

**What Works**:
- ✅ All CLI flags are accepted
- ✅ Blank line preservation works
- ✅ Line length limiting works
- ✅ Functionally equivalent output

**What Doesn't Work**:
- ❌ Preserving original backslash continuations
- ❌ Maintaining original multi-line recipe formatting

## Affected Tests

- `test_make_formatting_003_preserve_formatting_keeps_multiline_format` - #[ignore]
- `test_make_formatting_009_skip_consolidation_preserves_multiline` - #[ignore]

Both tests are marked with `#[ignore]` and documented with TODOs.

## Proposed Solutions

### Option 1: Track Line Breaks in AST (Recommended)

Add metadata to track original line breaks:

```rust
pub struct MakeItem {
    Target {
        recipe: Vec<String>,
        recipe_metadata: Option<RecipeMetadata>, // NEW
        // ...
    }
}

struct RecipeMetadata {
    original_line_breaks: Vec<usize>, // Indices where backslash continuations occurred
}
```

**Pros**: Clean separation, backward compatible
**Cons**: Increases AST size slightly

### Option 2: Conditional Preprocessing

Skip preprocessing when specific options are set:

```rust
pub fn parse_makefile_with_options(
    input: &str,
    options: &ParserOptions
) -> Result<MakeAst, String> {
    let preprocessed = if options.preserve_continuations {
        input.to_string() // Skip preprocessing
    } else {
        preprocess_line_continuations(input)
    };
    // ...
}
```

**Pros**: Simpler implementation
**Cons**: Parser must handle backslash continuations, increases complexity

### Option 3: Intelligent Line Breaking (Partial Solution)

Use `--max-line-length` to break long lines intelligently:

```bash
bashrs make purify Makefile --max-line-length 80 -o output.mk
```

**Pros**: Already implemented, no parser changes
**Cons**: Doesn't preserve *original* line breaks, creates *new* ones

## Implementation Estimate

**Option 1**: 3-4 hours
- Update AST definition
- Modify parser to track line breaks
- Update generator to reconstruct
- Add tests

**Option 2**: 2-3 hours
- Add parser options
- Update parser logic
- Update generator
- Add tests

**Option 3**: Already implemented (v6.34.0)

## Decision

**Deferred to future release**

Rationale (Toyota Way):
1. **Quality over Speed**: Parser refactor requires careful design
2. **Scope Management**: Feature is 81.8% complete (9/11 tests passing)
3. **Transparency**: Document limitation clearly for users
4. **Workaround Exists**: `--max-line-length` provides similar benefit
5. **Zero Defects**: Better to document limitation than ship broken feature

## Related

- PR: feat: Add formatting options for Makefile purification (EXTREME TDD)
- Files: `rash/tests/cli_make_formatting.rs`
- Parser: `rash/src/make_parser/parser.rs:preprocess_line_continuations`
- Generator: `rash/src/make_parser/generators.rs`

## Testing

When implementing, verify with:

```bash
# Run ignored tests
cargo test --test cli_make_formatting -- --ignored

# Should see:
# test_make_formatting_003_preserve_formatting_keeps_multiline_format ... ok
# test_make_formatting_009_skip_consolidation_preserves_multiline ... ok
```

## Documentation Updates Needed

When fixed:
1. Remove `#[ignore]` from tests
2. Update book chapter: `book/src/makefile/testing.md`
3. Update CHANGELOG.md
4. Update this issue status to CLOSED

## User Communication

**Current State** (v6.34.0):
```
⚠️  Note: --preserve-formatting and --skip-consolidation preserve blank lines
but do not preserve original backslash continuations in recipes. Use
--max-line-length to intelligently break long lines instead.
```

---

**Created**: 2025-11-10
**Last Updated**: 2025-11-10
**Reporter**: Dogfooding analysis
**Assignee**: Future contributor

---

## Solution Implemented (v6.35.0)

### Overview

Parser now tracks line continuation metadata during preprocessing and reconstructs original backslash continuations when formatting options are enabled.

### Implementation

**1. AST Metadata Tracking**

Added `RecipeMetadata` structure to track line break positions and original indentation:

```rust
/// Metadata about recipe formatting (line continuations, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeMetadata {
    /// Original line breaks in the recipe
    /// Each entry: (character_position, original_indentation)
    pub line_breaks: Vec<(usize, String)>,
}
```

**2. Parser Enhancement**

Modified `preprocess_line_continuations_with_metadata()` to:
- Track break positions before consolidation
- Record original indentation of continued lines
- Map preprocessed line numbers to metadata
- Thread metadata through parser to AST

**3. Generator Reconstruction**

Updated `generate_target()` to:
- Check if `preserve_formatting` or `skip_consolidation` is enabled
- Use `recipe_metadata` to reconstruct line breaks
- Insert backslash continuations at original positions
- Apply original indentation to continued lines

### Test Coverage

- ✅ test_make_formatting_003: --preserve-formatting preserves backslashes
- ✅ test_make_formatting_009: --skip-consolidation preserves backslashes
- ✅ 6,455 library tests pass (zero regressions)
- ✅ 11 CLI integration tests pass

### Files Changed

- `rash/src/make_parser/ast.rs`: Added RecipeMetadata structure
- `rash/src/make_parser/parser.rs`: Enhanced preprocessing with metadata tracking
- `rash/src/make_parser/generators.rs`: Implemented line break reconstruction
- `rash/tests/cli_make_formatting.rs`: Un-ignored passing tests

### Example

**Input Makefile:**
```makefile
build:
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release; \
	else \
		echo "cargo not found"; \
	fi
```

**Output with --preserve-formatting:**
```makefile
build:
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release; \
	else \
		echo "cargo not found"; \
	fi
```

✅ **Backslash continuations preserved exactly as in original!**

### Development Methodology

**EXTREME TDD phases completed:**
1. ✅ RED: Verified ignored tests fail
2. ✅ GREEN: Implemented parser metadata tracking and generator reconstruction
3. ✅ REFACTOR: Clippy clean, all tests pass
4. ✅ VERIFY: 6,466 tests passing (including 2 previously ignored)

### Related

- PR: feat: Resolve Issue #2 - Multi-line format preservation
- CHANGELOG: v6.35.0 release notes
- Issue #3: Mutation coverage for generators (separate effort)

---

**Resolved**: 2025-11-10  
**Implemented By**: EXTREME TDD methodology  
**Quality**: Zero defects, zero regressions
