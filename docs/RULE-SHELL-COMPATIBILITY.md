# Rule Shell Compatibility Matrix

**Version**: v6.28.0-dev
**Date**: 2025-11-02
**Status**: CLASSIFICATION IN PROGRESS

## Overview

This document classifies all 357 linter rules by shell compatibility to enable shell-specific rule filtering.

## Shell Compatibility Levels

- **Universal**: Applies to all shells (bash, zsh, sh, ksh)
- **BashOnly**: Bash-specific features (arrays, [[ ]], etc.)
- **ZshOnly**: Zsh-specific features (glob qualifiers, parameter flags)
- **ShOnly**: POSIX sh only (strict compliance)
- **BashZsh**: Works in both bash and zsh
- **NotSh**: Works in bash/zsh/ksh but not POSIX sh

## Classification Strategy

### Rule Categories

1. **Security Rules (SEC001-SEC008)**: Universal (all shells have these issues)
2. **Determinism Rules (DET001-DET003)**: Universal ($RANDOM, timestamps universal)
3. **Idempotency Rules (IDEM001-IDEM003)**: Universal (mkdir, rm, ln universal)
4. **Makefile Rules (MAKE001-MAKE005)**: N/A (Makefile-specific)
5. **ShellCheck Rules (SC2xxx)**: Need classification (varies by shell)

## Rule Classification

### Security Rules (8 rules) - Universal

| Rule ID | Name | Compatibility | Reason |
|---------|------|---------------|--------|
| SEC001 | Command injection | Universal | All shells vulnerable |
| SEC002 | Unsafe eval | Universal | All shells have eval |
| SEC003 | Unquoted variables | Universal | All shells expand variables |
| SEC004 | User input in commands | Universal | Security issue in all shells |
| SEC005 | Unsafe PATH | Universal | All shells use PATH |
| SEC006 | Dangerous rm patterns | Universal | All shells have rm |
| SEC007 | Insecure temp files | Universal | All shells create temp files |
| SEC008 | Source untrusted files | Universal | All shells have source/. |

**Classification**: ✅ All 8 rules are Universal

### Determinism Rules (3 rules) - Universal

| Rule ID | Name | Compatibility | Reason |
|---------|------|---------------|--------|
| DET001 | $RANDOM usage | Universal | Non-determinism universal issue |
| DET002 | Timestamp usage | Universal | Timestamps in all shells |
| DET003 | Wildcard ordering | Universal | Glob expansion in all shells |

**Classification**: ✅ All 3 rules are Universal

### Idempotency Rules (3 rules) - Universal

| Rule ID | Name | Compatibility | Reason |
|---------|------|---------------|--------|
| IDEM001 | mkdir without -p | Universal | mkdir in all shells |
| IDEM002 | rm without -f | Universal | rm in all shells |
| IDEM003 | ln without -f | Universal | ln in all shells |

**Classification**: ✅ All 3 rules are Universal

### Makefile Rules (5 rules) - N/A

| Rule ID | Name | Compatibility | Reason |
|---------|------|---------------|--------|
| MAKE001 | Wildcard sorting | N/A | Makefile-specific |
| MAKE002 | mkdir -p | N/A | Makefile-specific |
| MAKE003 | Variable quoting | N/A | Makefile-specific |
| MAKE004 | .PHONY missing | N/A | Makefile-specific |
| MAKE005 | Recursive variables | N/A | Makefile-specific |

**Classification**: ✅ All 5 rules are Makefile-specific (no shell filtering needed)

### ShellCheck Rules (323 rules) - Classification Needed

**Progress**: 0/323 rules classified

#### Classification Criteria

**BashOnly** indicators:
- Arrays: `arr=()`, `${arr[@]}`, `${#arr[@]}`
- [[ ]] test syntax
- Process substitution: `<(cmd)`, `>(cmd)`
- `{1..10}` brace expansion
- `[[` regex: `=~`
- `source` command (bash-specific)
- `declare`, `local` with type flags

**ZshOnly** indicators:
- Glob qualifiers: `*.txt(N)`, `*.sh(.x)`
- Parameter expansion flags: `${(@)param}`, `${(%)param}`
- Array indexing (1-based vs 0-based)
- `setopt` command
- `autoload` command

**ShOnly** (POSIX) indicators:
- Only `[ ]` tests (no `[[ ]]`)
- No arrays
- No process substitution
- No brace expansion
- Limited parameter expansion
- `.` instead of `source`

**Universal** indicators:
- Basic quoting rules
- Command substitution: `$(cmd)`, `` `cmd` ``
- Variable expansion: `$var`, `${var}`
- Pipes, redirects
- Basic control flow: `if`, `for`, `while`, `case`

## Classification Progress

| Category | Total | Classified | Universal | BashOnly | ZshOnly | ShOnly | NotSh |
|----------|-------|------------|-----------|----------|---------|--------|-------|
| SEC | 8 | 8 | 8 | 0 | 0 | 0 | 0 |
| DET | 3 | 3 | 3 | 0 | 0 | 0 | 0 |
| IDEM | 3 | 3 | 3 | 0 | 0 | 0 | 0 |
| MAKE | 5 | 5 | N/A | N/A | N/A | N/A | N/A |
| SC2xxx | 323 | 0 | ? | ? | ? | ? | ? |
| **TOTAL** | **342** | **19** | **14** | **?** | **?** | **?** | **?** |

**Next**: Classify SC2xxx rules (323 rules remaining)

## Implementation Plan

### Phase 1: Metadata Structure (Sprint 1, Day 1)

```rust
// rash/src/linter/shell_compatibility.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellCompatibility {
    /// Works in all shells (bash, zsh, sh, ksh)
    Universal,
    /// Bash-specific features only
    BashOnly,
    /// Zsh-specific features only
    ZshOnly,
    /// POSIX sh only (strict)
    ShOnly,
    /// Works in bash and zsh
    BashZsh,
    /// Works in bash/zsh/ksh but not sh
    NotSh,
}

impl ShellCompatibility {
    pub fn applies_to(&self, shell: ShellType) -> bool {
        match (self, shell) {
            (ShellCompatibility::Universal, _) => true,
            (ShellCompatibility::BashOnly, ShellType::Bash) => true,
            (ShellCompatibility::ZshOnly, ShellType::Zsh) => true,
            (ShellCompatibility::ShOnly, ShellType::Sh) => true,
            (ShellCompatibility::BashZsh, ShellType::Bash | ShellType::Zsh) => true,
            (ShellCompatibility::NotSh, ShellType::Bash | ShellType::Zsh | ShellType::Ksh) => true,
            _ => false,
        }
    }
}
```

### Phase 2: Rule Annotation (Sprint 1, Days 1-2)

Add compatibility metadata to each rule:

```rust
// Example: rash/src/linter/rules/sc2086.rs
pub fn sc2086() -> Rule {
    Rule {
        id: "SC2086",
        name: "Quote variables to prevent word splitting",
        severity: Severity::Warning,
        compatibility: ShellCompatibility::Universal, // ← NEW
        check: check_sc2086,
        fix: Some(fix_sc2086),
    }
}
```

### Phase 3: Filtering Engine (Sprint 2)

Implement rule filtering based on shell type:

```rust
// rash/src/linter/rule_filter.rs
pub fn filter_rules_for_shell(shell: ShellType, rules: Vec<Rule>) -> Vec<Rule> {
    rules.into_iter()
        .filter(|rule| rule.compatibility.applies_to(shell))
        .collect()
}
```

### Phase 4: Zsh-Specific Rules (Sprint 3)

Add 20 new zsh-specific linter rules (ZSH001-ZSH020).

## Testing Strategy

### Classification Tests (357 tests)

Each rule gets a test verifying compatibility:

```rust
#[test]
fn test_sc2086_compatibility() {
    let rule = sc2086();
    assert_eq!(rule.compatibility, ShellCompatibility::Universal);
}
```

### Filtering Tests (20 tests)

Verify filtering works correctly for each shell type.

### Property Tests (10 tests)

Ensure filtering preserves rule ordering and consistency.

## Quality Gates

- [ ] All 357 rules classified
- [ ] 357 classification tests passing
- [ ] Filtering engine implemented
- [ ] 20 filtering tests passing
- [ ] 10 property tests passing
- [ ] Zero regressions (6021+ tests still passing)
- [ ] Documentation complete
- [ ] EXTREME TDD methodology followed

## Timeline

- **Day 1**: Implement metadata structure + classify first 50 rules
- **Day 2**: Classify remaining 273 rules + all tests passing
- **Days 3-5**: Filtering engine + zsh rules (Sprints 2-3)

## References

- ShellCheck Wiki: https://www.shellcheck.net/wiki/
- Bash Reference Manual: https://www.gnu.org/software/bash/manual/
- Zsh Documentation: https://zsh.sourceforge.io/Doc/
- POSIX Shell: https://pubs.opengroup.org/onlinepubs/9699919799/

---

**Status**: ✅ Document created, classification strategy defined
**Next**: Implement `ShellCompatibility` enum and begin rule classification
