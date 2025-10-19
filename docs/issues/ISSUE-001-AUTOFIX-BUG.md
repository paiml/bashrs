# Issue #1: Bash Auto-fix Creates Invalid Syntax

**Status**: 🚨 P0 - STOP THE LINE
**Severity**: Critical
**Category**: Bash Linter Auto-fix
**Found During**: Sprint 79 - Dogfooding quality-gates.sh

---

## Bug Description

The `bashrs lint --fix` command applies fixes that create **invalid bash syntax**, making fixed scripts unusable.

### Problem 1: Extra Closing Braces

**Input**:
```bash
echo -e "${BLUE}text${NC}"
```

**Expected Output**:
```bash
echo -e "${BLUE}text${NC}"  # (no change, already correct)
```

**Actual Output**:
```bash
echo -e "${BLUE}text"${NC}"}"  # SYNTAX ERROR - extra }
```

### Problem 2: Incorrect SC2116 Fix

**Input**:
```bash
local coverage_int=$(echo "$coverage" | cut -d. -f1)
```

**Expected Output**:
```bash
local coverage_int=$(printf '%s' "$coverage" | cut -d. -f1)
# OR
coverage_int=$(cut -d. -f1 <<< "$coverage")
```

**Actual Output**:
```bash
local coverage_int="$coverage" | cut -d. -f1)  # SYNTAX ERROR - broken pipe
```

---

## Impact

- ✗ Auto-fixed scripts fail `bash -n` syntax check
- ✗ Scripts cannot execute
- ✗ Breaks dogfooding workflow
- ✗ User trust in `--fix` feature destroyed

---

## Root Cause Analysis (5 Whys)

1. **Why does auto-fix create syntax errors?**
   - Because the fix application logic doesn't properly replace text

2. **Why doesn't it properly replace text?**
   - Because it's appending fixes instead of replacing the problematic code

3. **Why is it appending instead of replacing?**
   - Because the fix application logic in `rash/src/linter/fix.rs` is incorrect

4. **Why wasn't this caught before?**
   - Because we didn't have integration tests that verify syntax validity after auto-fix

5. **Why don't we have those tests?**
   - Because we didn't follow EXTREME TDD when implementing auto-fix feature

---

## Reproduction Steps

```bash
# Create test file
cat > /tmp/test.sh <<'EOF'
#!/bin/bash
echo -e "${RED}Error${NC}"
local val=$(echo "$x" | cut -d. -f1)
EOF

# Apply auto-fix
./target/release/bashrs lint /tmp/test.sh --fix

# Verify syntax
bash -n /tmp/test.sh
# Result: syntax error
```

---

## Fix Plan (EXTREME TDD)

### Phase 1: RED - Write Failing Tests

Create `rash/tests/test_issue_001_autofix.rs`:

```rust
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ISSUE_001_autofix_preserves_syntax() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.sh");

    // ARRANGE: Create bash script with shellcheck warnings
    fs::write(&test_file, r#"#!/bin/bash
echo -e "${RED}Error${NC}"
local val=$(echo "$x" | cut -d. -f1)
rm file.txt
"#).unwrap();

    // ACT: Apply auto-fix
    Command::cargo_bin("bashrs").unwrap()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: Fixed file passes bash syntax check
    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(&test_file)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Fixed script should pass bash syntax check. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_ISSUE_001_autofix_no_extra_braces() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.sh");

    fs::write(&test_file, r#"#!/bin/bash
echo -e "${BLUE}text${NC}"
"#).unwrap();

    Command::cargo_bin("bashrs").unwrap()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    let fixed = fs::read_to_string(&test_file).unwrap();

    // Should not have double closing braces
    assert!(!fixed.contains(r#""}"}"#), "Should not have extra closing braces");
    assert!(!fixed.contains(r#""${NC}"}"#), "Should not have malformed variable refs");
}

#[test]
fn test_ISSUE_001_autofix_sc2116_correctly() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.sh");

    fs::write(&test_file, r#"#!/bin/bash
local val=$(echo "$x" | cut -d. -f1)
"#).unwrap();

    Command::cargo_bin("bashrs").unwrap()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    let fixed = fs::read_to_string(&test_file).unwrap();

    // Should have valid syntax (either printf or heredoc)
    assert!(
        fixed.contains("printf") || fixed.contains("<<<") || !fixed.contains("echo \"$x\""),
        "SC2116 fix should be syntactically valid"
    );

    // Verify syntax
    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(&test_file)
        .output()
        .unwrap();
    assert!(output.status.success());
}
```

### Phase 2: GREEN - Fix Implementation

Location: `rash/src/linter/fix.rs`

**Current (Broken) Logic**:
```rust
// Appends instead of replacing
pub fn apply_fixes(content: &str, fixes: &[Fix]) -> Result<String, Error> {
    let mut result = content.to_string();
    for fix in fixes {
        result.push_str(&fix.replacement);  // ❌ WRONG - appending
    }
    Ok(result)
}
```

**Fixed Logic**:
```rust
pub fn apply_fixes(content: &str, fixes: &[Fix]) -> Result<String, Error> {
    let mut result = content.to_string();

    // Sort fixes by position (reverse order to maintain offsets)
    let mut sorted_fixes = fixes.to_vec();
    sorted_fixes.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    for fix in sorted_fixes {
        // Replace the problematic span with the fix
        let start = fix.span.start;
        let end = fix.span.end;

        result.replace_range(start..end, &fix.replacement);
    }

    Ok(result)
}
```

### Phase 3: REFACTOR - Clean Up

- Extract helper functions
- Ensure complexity <10
- Add comments explaining fix application logic

### Phase 4: Property Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_autofix_always_valid_syntax(
        content in "[a-zA-Z0-9 ${}\\n]{10,100}"
    ) {
        // Apply fixes to random bash-like content
        // Verify result is either valid or unchanged
    }
}
```

### Phase 5: Mutation Testing

```bash
cargo mutants --file rash/src/linter/fix.rs -- --lib
# Target: ≥90% kill rate
```

---

## Acceptance Criteria

- [ ] ✅ All 3 new tests pass
- [ ] ✅ `bash -n` validates all auto-fixed scripts
- [ ] ✅ No syntax errors introduced by `--fix`
- [ ] ✅ Property tests pass (100+ cases)
- [ ] ✅ Mutation score ≥90% on fix.rs
- [ ] ✅ Integration test: quality-gates.sh auto-fix works
- [ ] ✅ All existing 808+ tests still pass

---

## Timeline

- **RED**: 30 min (write failing tests)
- **GREEN**: 1 hour (fix implementation)
- **REFACTOR**: 30 min (clean up)
- **PROPERTY**: 30 min (generative tests)
- **MUTATION**: 30 min (mutation testing)

**Total**: ~3 hours

---

## Status Updates

- [ ] Tests written (RED)
- [ ] Tests failing as expected
- [ ] Implementation fixed (GREEN)
- [ ] Tests passing
- [ ] Code refactored
- [ ] Property tests added
- [ ] Mutation testing complete
- [ ] Issue resolved

**Resolution Date**: TBD
**Fixed in Version**: v2.0.1 (pending)
