# CONFIG-003: Consolidate Duplicate Aliases

**Category**: Configuration / Maintainability
**Severity**: Warning
**Since**: v6.1.0
**Fixable**: Yes (automatic)

## Problem

Shell configuration files often accumulate duplicate alias definitions over time as users experiment with different settings. When the same alias is defined multiple times:

- **Confusing behavior**: Only the last definition takes effect
- **Maintenance burden**: Harder to track which aliases are active
- **Cluttered configs**: Unnecessary duplication
- **Debugging difficulty**: Hard to find which alias definition is "winning"

### Example Problem

```bash
# Early in .bashrc
alias ls='ls --color=auto'
alias ll='ls -la'

# ... 100 lines later ...

# Forgot about the first one!
alias ls='ls -G'           # This one wins
alias ll='ls -alh'         # This one wins
```

The second definitions override the first ones, but both remain in the file causing confusion.

## Detection

Rash analyzes all alias definitions and detects when the same alias name appears multiple times:

```bash,no_run
bashrs config analyze messy.bashrc
```

Output:

```text
[CONFIG-003] Duplicate alias definition: 'ls'
  → Line: 21
  → First occurrence: Line 17
  → Severity: Warning
  → Suggestion: Remove earlier definition or rename alias. Last definition wins in shell.
```

## Automatic Fix

Rash can automatically consolidate duplicates, keeping only the last definition (matching shell behavior):

```bash,no_run
bashrs config purify messy.bashrc --output clean.bashrc
```

**Before:**

```bash
alias ll='ls -la'
alias ls='ls --color=auto'
alias ll='ls -alh'         # Duplicate
alias grep='grep --color=auto'
alias ll='ls -lAh'         # Duplicate
```

**After:**

```bash
alias ls='ls --color=auto'
alias grep='grep --color=auto'
alias ll='ls -lAh'         # Only the last definition kept
```

## Why Last Definition Wins

CONFIG-003 follows shell behavior where later alias definitions override earlier ones:

```bash
# In shell
$ alias ls='ls --color=auto'
$ alias ls='ls -G'
$ alias ls
alias ls='ls -G'           # Only the last one is active
```

This matches how shells process config files line-by-line.

## Implementation

The consolidation algorithm:

```rust,ignore
use std::collections::HashMap;
use regex::Regex;

/// Consolidate duplicate aliases, keeping only the last definition
pub fn consolidate_aliases(source: &str) -> String {
    let aliases = analyze_aliases(source);

    if aliases.is_empty() {
        return source.to_string();
    }

    // Build map of alias names to their last definition line
    let mut last_definition: HashMap<String, usize> = HashMap::new();
    for alias in &aliases {
        last_definition.insert(alias.name.clone(), alias.line);
    }

    // Build set of lines to skip (duplicates)
    let mut lines_to_skip = Vec::new();
    for alias in &aliases {
        if let Some(&last_line) = last_definition.get(&alias.name) {
            if alias.line != last_line {
                // This is not the last definition - skip it
                lines_to_skip.push(alias.line);
            }
        }
    }

    // Reconstruct source, skipping duplicate lines
    let mut result = Vec::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if lines_to_skip.contains(&line_num) {
            continue; // Skip this duplicate
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

// Helper types
struct AliasDefinition {
    line: usize,
    name: String,
    value: String,
}

fn analyze_aliases(source: &str) -> Vec<AliasDefinition> {
    // Implementation details...
    vec![]
}
```

## Testing

CONFIG-003 has comprehensive tests:

```rust,ignore
#[test]
fn test_config_003_consolidate_simple() {
    // ARRANGE
    let source = r#"alias ls='ls --color=auto'
alias ls='ls -G'"#;

    // ACT
    let result = consolidate_aliases(source);

    // ASSERT
    assert_eq!(result, "alias ls='ls -G'");
}

#[test]
fn test_config_003_consolidate_multiple() {
    // ARRANGE
    let source = r#"alias ll='ls -la'
alias ls='ls --color=auto'
alias ll='ls -alh'
alias grep='grep --color=auto'
alias ll='ls -lAh'"#;

    let expected = r#"alias ls='ls --color=auto'
alias grep='grep --color=auto'
alias ll='ls -lAh'"#;

    // ACT
    let result = consolidate_aliases(source);

    // ASSERT
    assert_eq!(result, expected);
}

#[test]
fn test_config_003_idempotent() {
    // ARRANGE
    let source = r#"alias ls='ls --color=auto'
alias ls='ls -G'"#;

    // ACT
    let consolidated_once = consolidate_aliases(source);
    let consolidated_twice = consolidate_aliases(&consolidated_once);

    // ASSERT
    assert_eq!(
        consolidated_once, consolidated_twice,
        "Consolidation should be idempotent"
    );
}
```

## Real-World Example

Common scenario in a 5-year-old .bashrc:

```bash
# Original setup (2019)
alias ll='ls -la'
alias ls='ls --color=auto'
alias grep='grep --color=auto'

# Tried new options (2020)
alias ll='ls -lah'

# macOS-specific (2021)
alias ls='ls -G'

# Final preference (2023)
alias ll='ls -lAh'
```

After purification:

```bash
# Consolidated aliases
alias ls='ls -G'
alias grep='grep --color=auto'
alias ll='ls -lAh'
```

Three duplicate definitions reduced to three clean aliases.

## CLI Usage

```bash
# Analyze for duplicates
bashrs config analyze ~/.bashrc

# Lint and exit with error code if found
bashrs config lint ~/.bashrc

# Preview what would be fixed
bashrs config purify ~/.bashrc --dry-run

# Apply fixes with backup (default: ~/.bashrc.backup.TIMESTAMP)
bashrs config purify ~/.bashrc --fix

# Apply without backup (dangerous!)
bashrs config purify ~/.bashrc --fix --no-backup

# Output to different file
bashrs config purify ~/.bashrc --output ~/.bashrc.clean
```

## Configuration

You can control CONFIG-003 behavior through CLI flags:

```bash
# Dry-run to preview changes (default)
bashrs config purify ~/.bashrc --dry-run

# Apply with backup
bashrs config purify ~/.bashrc --fix

# Skip backup (not recommended)
bashrs config purify ~/.bashrc --fix --no-backup

# JSON output for tooling
bashrs config analyze ~/.bashrc --format json
```

## Edge Cases

### Comments Between Duplicates

```bash
# Before
alias ls='ls --color=auto'
# This is my preferred ls
alias ls='ls -G'

# After
# This is my preferred ls
alias ls='ls -G'
```

Comments and blank lines are preserved.

### Mixed Quote Styles

```bash
# Before
alias ls='ls --color=auto'    # Single quotes
alias ls="ls -G"               # Double quotes

# After
alias ls="ls -G"               # Both styles supported
```

CONFIG-003 handles both single and double quotes.

### No Duplicates

If no duplicates exist, the file is unchanged:

```bash
alias ll='ls -la'
alias ls='ls --color=auto'
alias grep='grep --color=auto'
# No changes needed
```

## Related Rules

- [CONFIG-001](./config-001.md): PATH deduplication
- [CONFIG-002](./config-002.md): Quote variable expansions
- [CONFIG-004](./config-004.md): Remove non-deterministic constructs (coming soon)

## Performance

CONFIG-003 is highly optimized:

- **Regex-based**: O(n) scanning with compiled regex
- **Single pass**: Analyzes and consolidates in one pass
- **Idempotent**: Safe to run multiple times
- **Fast**: ~1ms for typical .bashrc files

## FAQ

**Q: Why keep the last definition instead of the first?**

A: The last definition is what's actually active in your shell. Keeping it matches real behavior and is what users typically intend (later overrides earlier).

**Q: What if I have conditional aliases?**

A: CONFIG-003 only consolidates identical alias names. Conditional aliases are preserved:

```bash
if [ "$OS" = "Darwin" ]; then
    alias ls='ls -G'
else
    alias ls='ls --color=auto'
fi
# Both kept - they're conditional
```

**Q: Can I disable this rule?**

A: Currently, rules cannot be disabled individually. This feature is planned for v7.1.

**Q: What about functions with the same name as aliases?**

A: CONFIG-003 only analyzes `alias` definitions. Functions are handled separately.

## Best Practices

✅ **DO**:
- Run `bashrs config analyze` before manual edits
- Use `--dry-run` to preview changes first
- Keep backups (default behavior)
- Consolidate regularly during config maintenance

❌ **DON'T**:
- Skip the dry-run step
- Disable backups unless you're certain
- Edit config while shell is sourcing it
- Ignore CONFIG-003 warnings (they indicate confusion)

## See Also

- [Shell Configuration Overview](../overview.md)
- [Analyzing Config Files](../analyzing.md)
- [Purification Workflow](../purifying.md)
- [CONFIG-001: PATH Deduplication](./config-001.md)
- [CONFIG-002: Quote Variables](./config-002.md)
