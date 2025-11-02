# TICKET-005: Shell Type Detection for zsh Compatibility

**Status**: IN_PROGRESS
**Priority**: HIGH
**Assignee**: Claude Code
**Created**: 2025-11-02
**GitHub Issue**: #5
**Target Version**: v6.27.0

## Problem Statement

bashrs incorrectly lints zsh files using bash-specific rules, causing critical false positives:
- **SC2296**: "Parameter expansions can't be nested" - FALSE for valid zsh syntax
- **SC2031**: "Variable assigned in subshell" - MISLEADING for zsh scope
- **SC2046**: "Quote to prevent word splitting" - Already correctly quoted

**Root Cause**: No shell type detection - bashrs assumes all files are bash.

## Success Criteria

- [ ] Detect shell type from shebang (`#!/usr/bin/env zsh`)
- [ ] Detect shell type from file extension (`.zshrc`, `.zsh`)
- [ ] Detect shell type from ShellCheck directive (`# shellcheck shell=zsh`)
- [ ] Pass correct `--shell` flag to ShellCheck
- [ ] Zero false positives on real `.zshrc` files
- [ ] All tests pass (EXTREME TDD)
- [ ] Property tests for edge cases
- [ ] Documentation updated

## Implementation Plan

### Phase 1: RED - Write Failing Tests

Create comprehensive test suite BEFORE implementation:

**Unit Tests** (`test_shell_detection.rs`):
```rust
#[test]
fn test_detect_zsh_from_shebang() {
    let content = "#!/usr/bin/env zsh\necho hello";
    assert_eq!(detect_shell_type(Path::new("test.sh"), content), ShellType::Zsh);
}

#[test]
fn test_detect_zsh_from_extension() {
    let content = "echo hello";
    assert_eq!(detect_shell_type(Path::new(".zshrc"), content), ShellType::Zsh);
}

#[test]
fn test_detect_bash_default() {
    let content = "echo hello";
    assert_eq!(detect_shell_type(Path::new("script.sh"), content), ShellType::Bash);
}

#[test]
fn test_detect_from_shellcheck_directive() {
    let content = "# shellcheck shell=zsh\necho hello";
    assert_eq!(detect_shell_type(Path::new("test.sh"), content), ShellType::Zsh);
}
```

**Integration Tests** (`test_zsh_linting.rs`):
```rust
#[test]
fn test_zsh_parameter_expansion_no_error() {
    let zshrc = r#"#!/usr/bin/env zsh
filtered=("${(@f)"$(echo line1)"}")
"#;
    let result = lint_shell(zshrc);
    // Should NOT contain SC2296
    assert!(!result.diagnostics.iter().any(|d| d.code == "SC2296"));
}
```

### Phase 2: GREEN - Implement Shell Type Detection

**New Types** (`rash/src/linter/shell_type.rs`):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Sh,
    Ksh,
    Auto,
}

pub fn detect_shell_type(path: &Path, content: &str) -> ShellType {
    // Priority order:
    // 1. Shebang
    // 2. ShellCheck directive
    // 3. File extension
    // 4. Default (Bash)
}
```

**Detection Logic**:
1. Parse shebang: `#!/usr/bin/env zsh`, `#!/bin/zsh` → Zsh
2. Parse directive: `# shellcheck shell=zsh` → Zsh
3. Check extension: `.zshrc`, `.zsh`, `.zshenv` → Zsh
4. Check filename: `.bashrc`, `.bash_profile` → Bash
5. Default: Bash

### Phase 3: GREEN - Integrate with ShellCheck

**Update Linter** (`rash/src/linter/mod.rs`):
```rust
pub fn lint_shell_with_type(source: &str, shell_type: ShellType) -> LintResult {
    // Pass --shell flag to ShellCheck
    let shell_arg = match shell_type {
        ShellType::Zsh => "zsh",
        ShellType::Bash => "bash",
        ShellType::Sh => "sh",
        ShellType::Ksh => "ksh",
        ShellType::Auto => "auto",
    };

    // Run shellcheck with --shell argument
}
```

### Phase 4: REFACTOR - Code Cleanup

- Extract helper functions (complexity < 10)
- Add comprehensive documentation
- Ensure all error paths handled
- Clean up any code duplication

### Phase 5: Property Tests

**Property-Based Tests**:
```rust
proptest! {
    #[test]
    fn prop_shebang_detection_always_consistent(
        shell in "(bash|zsh|sh|ksh)"
    ) {
        let content = format!("#!/usr/bin/env {}\necho test", shell);
        let detected = detect_shell_type(Path::new("test.sh"), &content);
        // Verify detection is consistent
        prop_assert!(matches_shell(&detected, &shell));
    }
}
```

### Phase 6: Documentation

- Update `book/src/linting/shell-detection.md`
- Update CLI help text
- Add examples to CHANGELOG
- Update README with zsh support

### Phase 7: Release v6.27.0

- Follow Release Protocol from CLAUDE.md
- Update CHANGELOG.md
- Bump version to 6.27.0
- Create git tag
- Publish to crates.io

## Test Cases

### Valid Zsh Syntax (Should Pass)

```zsh
# TC-1: Nested parameter expansion with array flags
filtered=("${(@f)"$(echo -e "line1\nline2")"}")

# TC-2: zsh array indexing (starts at 1)
echo "${array[1]}"  # Not ${array[0]}

# TC-3: zsh-specific parameter expansion
var=${${param#prefix}%%suffix}
```

### File Detection

| File | Expected Shell | Reason |
|------|---------------|--------|
| `.zshrc` | Zsh | Extension |
| `.zshenv` | Zsh | Extension |
| `script.zsh` | Zsh | Extension |
| `.bashrc` | Bash | Extension |
| `script.sh` with `#!/bin/zsh` | Zsh | Shebang |
| `script.sh` with `# shellcheck shell=zsh` | Zsh | Directive |
| `script.sh` (no markers) | Bash | Default |

## Edge Cases

1. **Multiple indicators conflict**:
   - Shebang says bash, extension is `.zsh` → Use shebang (highest priority)

2. **Unknown shell**:
   - `#!/bin/fish` → Fall back to Bash, warn user

3. **No shebang, no extension**:
   - `script` with no markers → Default to Bash

4. **ShellCheck directive overrides**:
   - `# shellcheck shell=zsh` → Always use Zsh

## Quality Gates

- [ ] All unit tests pass (>20 tests)
- [ ] All integration tests pass
- [ ] Property tests pass (>100 cases)
- [ ] Mutation testing: >90% kill rate
- [ ] Clippy clean
- [ ] Complexity <10 for all functions
- [ ] Zero regressions (6004+ tests)
- [ ] Real `.zshrc` files lint without false positives

## Definition of Done

- [ ] Shell type detection implemented and tested
- [ ] ShellCheck integration updated
- [ ] CLI accepts `--shell` flag (optional)
- [ ] Book documentation complete
- [ ] CHANGELOG updated
- [ ] All quality gates pass
- [ ] v6.27.0 released to crates.io
- [ ] GitHub issue #5 closed

## Time Estimate

- Phase 1 (RED): 30 minutes
- Phase 2 (GREEN): 45 minutes
- Phase 3 (GREEN): 30 minutes
- Phase 4 (REFACTOR): 20 minutes
- Phase 5 (Property tests): 30 minutes
- Phase 6 (Documentation): 30 minutes
- Phase 7 (Release): 45 minutes
- **Total**: ~4 hours

## Notes

- ShellCheck already supports `--shell=zsh`, we just need to pass it correctly
- This is a **minor version bump** (new feature, backward compatible)
- Must maintain backward compatibility - existing bash linting unchanged
