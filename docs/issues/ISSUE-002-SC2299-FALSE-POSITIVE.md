# ISSUE-002: SC2299 False Positive on Default Value Expansion

**Status**: RESOLVED ✅
**Severity**: Low (false positive, not a security issue)
**Component**: Makefile Linter
**Found During**: bashrs dogfooding (self-validation)
**Date**: 2025-11-25
**Resolved**: 2025-11-25 (commit e6d139c62)

## Description

bashrs incorrectly flags `${var:-default}` (default value expansion) as SC2299 "Parameter expansions can't use variables in offset/length".

SC2299 should only trigger for substring operations like `${var:$start:$length}`, not for default value expansion `${var:-default}`.

## Reproduction

```makefile
# In Makefile - this is VALID POSIX syntax
test-property:
	@THREADS=$${PROPTEST_THREADS:-$$(nproc 2>/dev/null || echo 4)}; \
	echo "$$THREADS"
```

Running:
```bash
bashrs make lint Makefile
```

Output:
```
✗ 335:1-98 [error] SC2299: Parameter expansions can't use variables in offset/length
```

## Expected Behavior

This code should **NOT** trigger SC2299 because:
1. `${var:-default}` is default value expansion, not substring/offset expansion
2. This is valid POSIX shell syntax
3. ShellCheck (the reference implementation) does not flag this as SC2299

## Actual Behavior

bashrs flags this as an error, treating the `:-` as if it were `:$offset:$length`.

## Impact

- 2 false positive errors in bashrs's own Makefile (lines 335, 345)
- May cause confusion for users with valid Makefiles
- Does not prevent valid Makefiles from working

## Workaround

None needed - the Makefile works correctly despite the warning.

## Root Cause Analysis

The Makefile linter likely:
1. Sees `${...}` pattern with `:` character
2. Incorrectly classifies `:-` as an offset marker
3. Doesn't distinguish between `:-` (default value) and `:N` (offset)

## Fix Recommendation

Update the SC2299 detection logic to:
1. Only trigger for `${var:$offset}` or `${var:$offset:$length}` patterns
2. Exclude `${var:-...}`, `${var:+...}`, `${var:=...}`, `${var:?...}` (parameter modifiers)
3. Add tests for all POSIX parameter expansion forms

## References

- ShellCheck SC2299: https://www.shellcheck.net/wiki/SC2299
- POSIX Parameter Expansion: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_02

---

**Discovered by**: bashrs dogfooding (self-validation)
**Related**: docs/dogfooding/BASHRS_DOGFOODING.md

---

## Resolution

**Fixed in commit**: e6d139c62

**Solution**: Updated the SC2299 regex to use alternation that excludes POSIX parameter modifiers:

```rust
// Old regex (buggy):
r"\$\{[a-zA-Z_][a-zA-Z0-9_]*:[^}]*\$"

// New regex (fixed):
r"\$\{[a-zA-Z_][a-zA-Z0-9_]*:(\$|[^-+=?}][^}]*\$)"
```

The new regex:
1. Matches `${var:$offset}` - variable directly after colon (invalid)
2. Matches `${var:0:$len}` - variable in length position (invalid)
3. Excludes `${var:-...}`, `${var:+...}`, `${var:=...}`, `${var:?...}` (valid POSIX)

**Tests added**: 6 new tests for all POSIX parameter modifiers with variables

**Result**: Makefile now passes with 0 errors (was 2 SC2299 errors)
