# CONFIG-001: Deduplicate PATH Entries

**Category**: Configuration
**Severity**: Warning
**Since**: v6.0.0
**Fixable**: Yes (automatic)

## Problem

Duplicate entries in `PATH` cause:
- Slower command lookup
- Confusion about which binary will run
- Maintenance burden

### Example Problem

```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate!
```

The third line adds `/usr/local/bin` again, which was already added in the first line.

## Detection

Rash tracks all PATH modifications and detects when the same directory is added multiple times:

```bash,no_run
bashrs config analyze messy.bashrc
```

Output:

```text
[CONFIG-001] Duplicate PATH entry
  → Line: 3
  → Path: /usr/local/bin
  → First occurrence: Line 1
  → Suggestion: Remove duplicate entry or use conditional addition
```

## Automatic Fix

Rash can automatically remove duplicates while preserving the first occurrence:

```bash,no_run
bashrs config purify messy.bashrc --output clean.bashrc
```

**Before:**

```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
```

**After:**

```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
# Duplicate removed by CONFIG-001
```

## Implementation Details

The deduplication algorithm:

```rust,no_run
use std::collections::HashSet;

/// Deduplicate PATH entries, preserving first occurrence
pub fn deduplicate_path_entries(source: &str) -> String {
    let entries = analyze_path_entries(source);
    let mut seen_paths = HashSet::new();
    let mut result = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Check if this line adds a PATH entry we've seen
        let mut skip_line = false;
        for entry in &entries {
            if entry.line == line_num && seen_paths.contains(&entry.path) {
                // Duplicate found - skip this line
                skip_line = true;
                break;
            }
        }

        if skip_line {
            continue;
        }

        // Track new paths
        for entry in &entries {
            if entry.line == line_num {
                seen_paths.insert(&entry.path);
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

// Helper function (part of actual implementation)
fn analyze_path_entries(source: &str) -> Vec<PathEntry> {
    // Implementation details...
    vec![]
}

struct PathEntry {
    line: usize,
    path: String,
}
```

## Testing

The CONFIG-001 rule has comprehensive tests:

```rust
#[test]
fn test_config_001_detect_duplicate_paths() {
    // ARRANGE
    let source = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH""#;

    // ACT
    let entries = analyze_path_entries(source);
    let issues = detect_duplicate_paths(&entries);

    // ASSERT
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "CONFIG-001");
    assert_eq!(issues[0].line, 3);
}

#[test]
fn test_config_001_deduplicate_preserves_first() {
    // ARRANGE
    let source = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH""#;

    // ACT
    let result = deduplicate_path_entries(source);

    // ASSERT
    assert_eq!(result.lines().count(), 2);
    assert!(result.contains("/usr/local/bin"));
    assert!(result.contains("/opt/homebrew/bin"));
    // Should only appear once
    assert_eq!(result.matches("/usr/local/bin").count(), 1);
}
```

## Real-World Example

Common scenario in .bashrc files:

```bash
# System default
export PATH="/usr/local/bin:$PATH"

# Homebrew
if [ -d "/opt/homebrew/bin" ]; then
    export PATH="/opt/homebrew/bin:$PATH"
fi

# Custom tools
export PATH="/usr/local/bin:$PATH"  # Oops, duplicate!

# Python tools
export PATH="$HOME/.local/bin:$PATH"
```

After purification:

```bash
# System default
export PATH="/usr/local/bin:$PATH"

# Homebrew
if [ -d "/opt/homebrew/bin" ]; then
    export PATH="/opt/homebrew/bin:$PATH"
fi

# Custom tools - removed duplicate

# Python tools
export PATH="$HOME/.local/bin:$PATH"
```

## Configuration

You can configure CONFIG-001 behavior:

```bash
# Dry-run (default) - show what would change
bashrs config purify ~/.bashrc --dry-run

# Apply fixes with backup
bashrs config purify ~/.bashrc --fix

# Skip backup (dangerous!)
bashrs config purify ~/.bashrc --fix --no-backup
```

## Related Rules

- [CONFIG-002](./config-002.md): Quote variable expansions
- [CONFIG-004](./config-004.md): Remove non-deterministic constructs

## FAQ

**Q: Why preserve the first occurrence, not the last?**

A: The first occurrence is usually the intended primary PATH. Later duplicates are often accidental.

**Q: What about conditional PATH additions?**

A: Rash preserves conditional logic. Duplicates are only removed if unconditional.

**Q: Can I disable this rule?**

A: Currently, rules cannot be disabled individually. This feature is planned for v7.1.

## See Also

- [Shell Configuration Overview](../overview.md)
- [Analyzing Config Files](../analyzing.md)
- [Purification Workflow](../purifying.md)
