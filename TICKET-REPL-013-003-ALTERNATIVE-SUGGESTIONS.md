# TICKET-REPL-013-003: Alternative Suggestions for Transformations

**Status**: IN_PROGRESS
**Priority**: HIGH
**Sprint**: REPL-013 (Purification Explainer)
**Dependencies**: REPL-013-001 ✅ (Detailed transformation explanations)
**Assignee**: Claude
**Created**: 2025-10-31

## Overview

Extend transformation explanations with alternative approaches and implementation options. Users need to understand not just what transformation was applied, but also what OTHER valid approaches exist for achieving the same safety/determinism/idempotency goals.

## Problem Statement

While transformations explain WHAT changed and WHY it's safer, they don't provide:

1. **Alternative approaches**: What other ways exist to achieve the same goal?
2. **Trade-offs**: When should you use one approach vs another?
3. **Context-specific options**: Are there better alternatives in specific scenarios?

## Solution

Add `alternatives` field to `TransformationExplanation` with multiple suggestion options, each explaining the approach, trade-offs, and when to use it.

## Technical Design

### 1. Define Alternative struct

```rust
/// A single alternative approach to the transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alternative {
    /// Brief description of this approach
    pub approach: String,

    /// Code example showing this alternative
    pub example: String,

    /// When to prefer this approach
    pub when_to_use: String,

    /// Pros of this approach
    pub pros: Vec<String>,

    /// Cons of this approach
    pub cons: Vec<String>,
}

impl Alternative {
    pub fn new(
        approach: impl Into<String>,
        example: impl Into<String>,
        when_to_use: impl Into<String>,
    ) -> Self {
        Self {
            approach: approach.into(),
            example: example.into(),
            when_to_use: when_to_use.into(),
            pros: Vec::new(),
            cons: Vec::new(),
        }
    }

    pub fn add_pro(mut self, pro: impl Into<String>) -> Self {
        self.pros.push(pro.into());
        self
    }

    pub fn add_con(mut self, con: impl Into<String>) -> Self {
        self.cons.push(con.into());
        self
    }
}
```

### 2. Extend TransformationExplanation

```rust
pub struct TransformationExplanation {
    pub category: TransformationCategory,
    pub title: String,
    pub original: String,
    pub transformed: String,
    pub what_changed: String,
    pub why_it_matters: String,
    pub line_number: Option<usize>,
    pub safety_rationale: SafetyRationale,

    // NEW: Alternative approaches
    pub alternatives: Vec<Alternative>,
}

impl TransformationExplanation {
    pub fn with_alternatives(mut self, alternatives: Vec<Alternative>) -> Self {
        self.alternatives = alternatives;
        self
    }
}
```

### 3. Generate alternatives for common transformations

```rust
/// Generate alternatives for idempotency transformations
pub fn generate_idempotency_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("mkdir") && title.contains("-p") => vec![
            Alternative::new(
                "Use install -d instead of mkdir -p",
                "install -d /path/to/dir",
                "When you need to set permissions during directory creation"
            )
            .add_pro("Can set owner/group/permissions atomically")
            .add_pro("Part of coreutils, widely available")
            .add_con("Less intuitive than mkdir for simple cases")
            .add_con("More flags to remember for common tasks"),

            Alternative::new(
                "Check before creating",
                "[ -d /path ] || mkdir /path",
                "When you want explicit control over error handling"
            )
            .add_pro("Explicit about what's happening")
            .add_pro("Can add custom logic between check and creation")
            .add_con("Not atomic - race condition between check and create")
            .add_con("More verbose than mkdir -p"),

            Alternative::new(
                "Use mkdir with error suppression",
                "mkdir /path 2>/dev/null || true",
                "When you don't care about the reason for failure"
            )
            .add_pro("Simple and concise")
            .add_pro("Idempotent")
            .add_con("Hides all errors, not just 'already exists'")
            .add_con("Can mask real problems like permission issues"),
        ],

        title if title.contains("rm") && title.contains("-f") => vec![
            Alternative::new(
                "Check before removing",
                "[ -e /path ] && rm /path",
                "When you want to know if the file existed"
            )
            .add_pro("Explicit about file existence")
            .add_pro("Can add logging or side effects")
            .add_con("Not atomic - race condition")
            .add_con("More verbose"),

            Alternative::new(
                "Use rm with error check",
                "rm /path 2>/dev/null || true",
                "When you want to suppress errors but keep other rm behavior"
            )
            .add_pro("Simple")
            .add_pro("Idempotent")
            .add_con("Hides all errors")
            .add_con("May mask permission problems"),
        ],

        title if title.contains("ln") && title.contains("-sf") => vec![
            Alternative::new(
                "Remove then create",
                "rm -f /link && ln -s /target /link",
                "When you need two separate operations"
            )
            .add_pro("Very explicit")
            .add_pro("Can add logic between removal and creation")
            .add_con("Not atomic - window where link doesn't exist")
            .add_con("More verbose"),

            Alternative::new(
                "Check before creating",
                "[ -L /link ] || ln -s /target /link",
                "When you want to preserve existing links"
            )
            .add_pro("Won't overwrite existing links")
            .add_pro("Explicit check")
            .add_con("Not idempotent if link points elsewhere")
            .add_con("Race condition between check and create"),
        ],

        _ => vec![
            Alternative::new(
                "Add explicit idempotency check",
                "[ condition ] || operation",
                "When you want fine-grained control"
            )
            .add_pro("Explicit about preconditions")
            .add_con("Not atomic")
        ],
    }
}

/// Generate alternatives for determinism transformations
pub fn generate_determinism_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("RANDOM") => vec![
            Alternative::new(
                "Use UUID for unique IDs",
                "id=$(uuidgen)  # or $(cat /proc/sys/kernel/random/uuid)",
                "When you need globally unique identifiers"
            )
            .add_pro("Guaranteed unique across machines")
            .add_pro("Standard format")
            .add_pro("Deterministic if you control the seed")
            .add_con("Requires uuidgen or /proc/sys/kernel")
            .add_con("Longer than simple numbers"),

            Alternative::new(
                "Use timestamp-based IDs",
                "id=$(date +%s%N)  # nanoseconds since epoch",
                "When you need time-ordered IDs"
            )
            .add_pro("Sortable by time")
            .add_pro("No external dependencies")
            .add_con("Not unique across machines")
            .add_con("Still non-deterministic (but reproducible with fixed time)"),

            Alternative::new(
                "Use hash of inputs",
                "id=$(echo \"$input\" | sha256sum | cut -d' ' -f1)",
                "When you want IDs derived from content"
            )
            .add_pro("Truly deterministic")
            .add_pro("Same input = same ID")
            .add_con("Requires sha256sum")
            .add_con("Collisions possible (though extremely rare)"),

            Alternative::new(
                "Use sequential counter",
                "id=$((++counter))  # with state file",
                "When you need simple incrementing IDs"
            )
            .add_pro("Simple and predictable")
            .add_pro("Compact")
            .add_con("Requires state management")
            .add_con("Not unique across processes without locking"),
        ],

        title if title.contains("timestamp") || title.contains("date") => vec![
            Alternative::new(
                "Use explicit version parameter",
                "version=$1  # Pass version as argument",
                "When version is known at invocation time"
            )
            .add_pro("Fully deterministic")
            .add_pro("Version controlled externally")
            .add_con("Requires coordination with caller"),

            Alternative::new(
                "Use git commit hash",
                "version=$(git rev-parse --short HEAD)",
                "When deploying from git repository"
            )
            .add_pro("Deterministic for given commit")
            .add_pro("Traceable to source code")
            .add_con("Requires git repository")
            .add_con("Not available in all environments"),

            Alternative::new(
                "Use build number from CI",
                "version=${BUILD_NUMBER:-dev}",
                "When running in CI/CD pipeline"
            )
            .add_pro("Integrates with CI/CD")
            .add_pro("Incrementing version numbers")
            .add_con("Requires CI environment")
            .add_con("May not be available locally"),
        ],

        _ => vec![
            Alternative::new(
                "Make value an input parameter",
                "value=$1  # Pass as argument",
                "When value should be controlled by caller"
            )
            .add_pro("Fully deterministic")
            .add_con("Requires caller to provide value")
        ],
    }
}

/// Generate alternatives for safety transformations
pub fn generate_safety_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("quot") || title.contains("variable") => vec![
            Alternative::new(
                "Use printf %q for shell-safe quoting",
                "safe=$(printf %q \"$variable\")",
                "When you need shell-escaped values"
            )
            .add_pro("Automatically escapes special characters")
            .add_pro("Safe for eval")
            .add_con("Bash-specific (not POSIX)")
            .add_con("Output may have backslashes"),

            Alternative::new(
                "Use arrays instead of strings",
                "args=(\"$var1\" \"$var2\"); command \"${args[@]}\"",
                "When handling multiple arguments"
            )
            .add_pro("Preserves word boundaries correctly")
            .add_pro("No quoting issues")
            .add_con("Bash-specific (not POSIX)")
            .add_con("More complex syntax"),

            Alternative::new(
                "Validate input before use",
                "if [[ $var =~ ^[a-zA-Z0-9_-]+$ ]]; then cmd \"$var\"; fi",
                "When you can restrict input to safe characters"
            )
            .add_pro("Explicit validation")
            .add_pro("Clear error handling")
            .add_con("May reject valid inputs")
            .add_con("Requires input constraints"),
        ],

        _ => vec![
            Alternative::new(
                "Use safer built-in alternatives",
                "# Use bash built-ins when possible",
                "When avoiding external commands"
            )
            .add_pro("No command injection risk")
            .add_con("Limited functionality")
        ],
    }
}
```

### 4. Format alternatives for display

```rust
/// Format alternatives section for reports
pub fn format_alternatives(alternatives: &[Alternative]) -> String {
    let mut output = String::new();

    if alternatives.is_empty() {
        return output;
    }

    output.push_str("Alternative Approaches:\n\n");

    for (i, alt) in alternatives.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, alt.approach));
        output.push_str(&format!("   Example: {}\n", alt.example));
        output.push_str(&format!("   When to use: {}\n", alt.when_to_use));

        if !alt.pros.is_empty() {
            output.push_str("   Pros:\n");
            for pro in &alt.pros {
                output.push_str(&format!("     + {}\n", pro));
            }
        }

        if !alt.cons.is_empty() {
            output.push_str("   Cons:\n");
            for con in &alt.cons {
                output.push_str(&format!("     - {}\n", con));
            }
        }

        output.push('\n');
    }

    output
}
```

## Unit Tests

### Test 1: mkdir Alternatives
```rust
#[test]
fn test_REPL_013_003_alternatives_mkdir() {
    // ARRANGE: mkdir transformation
    let alternatives = generate_idempotency_alternatives("mkdir → mkdir -p");

    // ASSERT: Has multiple alternatives
    assert!(!alternatives.is_empty());
    assert!(alternatives.len() >= 2);

    // ASSERT: Has install -d alternative
    assert!(alternatives.iter().any(|a| a.approach.contains("install -d")));

    // ASSERT: Each alternative has required fields
    for alt in &alternatives {
        assert!(!alt.approach.is_empty());
        assert!(!alt.example.is_empty());
        assert!(!alt.when_to_use.is_empty());
    }

    // ASSERT: Has pros and cons
    assert!(alternatives.iter().any(|a| !a.pros.is_empty()));
    assert!(alternatives.iter().any(|a| !a.cons.is_empty()));
}
```

### Test 2: $RANDOM Alternatives
```rust
#[test]
fn test_REPL_013_003_alternatives_random() {
    // ARRANGE: $RANDOM transformation
    let alternatives = generate_determinism_alternatives("Remove $RANDOM");

    // ASSERT: Has multiple alternatives
    assert!(!alternatives.is_empty());
    assert!(alternatives.len() >= 2);

    // ASSERT: Has UUID alternative
    assert!(alternatives.iter().any(|a| a.example.contains("uuid")));

    // ASSERT: Has hash-based alternative
    assert!(alternatives.iter().any(|a| a.example.contains("sha256")));

    // ASSERT: Each has pros/cons
    for alt in &alternatives {
        assert!(!alt.pros.is_empty() || !alt.cons.is_empty());
    }
}
```

### Test 3: Variable Quoting Alternatives
```rust
#[test]
fn test_REPL_013_003_alternatives_quoting() {
    // ARRANGE: Variable quoting transformation
    let alternatives = generate_safety_alternatives("Quote variables");

    // ASSERT: Has alternatives
    assert!(!alternatives.is_empty());

    // ASSERT: Has printf %q alternative
    assert!(alternatives.iter().any(|a| a.example.contains("printf %q")));

    // ASSERT: Has validation alternative
    assert!(alternatives.iter().any(|a| a.approach.contains("Validate")));
}
```

### Test 4: Alternative Builder
```rust
#[test]
fn test_REPL_013_003_alternative_builder() {
    // ARRANGE & ACT: Build alternative
    let alt = Alternative::new(
        "Use UUID",
        "id=$(uuidgen)",
        "For unique IDs"
    )
    .add_pro("Globally unique")
    .add_pro("Standard format")
    .add_con("Requires uuidgen");

    // ASSERT: All fields set
    assert_eq!(alt.approach, "Use UUID");
    assert_eq!(alt.example, "id=$(uuidgen)");
    assert_eq!(alt.when_to_use, "For unique IDs");
    assert_eq!(alt.pros.len(), 2);
    assert_eq!(alt.cons.len(), 1);
}
```

### Test 5: Format Alternatives
```rust
#[test]
fn test_REPL_013_003_format_alternatives() {
    // ARRANGE: Create alternatives
    let alternatives = vec![
        Alternative::new(
            "Approach 1",
            "example1",
            "When X"
        )
        .add_pro("Pro 1")
        .add_con("Con 1"),

        Alternative::new(
            "Approach 2",
            "example2",
            "When Y"
        )
        .add_pro("Pro 2"),
    ];

    // ACT: Format
    let formatted = format_alternatives(&alternatives);

    // ASSERT: All sections present
    assert!(formatted.contains("Alternative Approaches:"));
    assert!(formatted.contains("1. Approach 1"));
    assert!(formatted.contains("2. Approach 2"));
    assert!(formatted.contains("Example: example1"));
    assert!(formatted.contains("Pros:"));
    assert!(formatted.contains("+ Pro 1"));
    assert!(formatted.contains("Cons:"));
    assert!(formatted.contains("- Con 1"));
}
```

### Test 6: TransformationExplanation with Alternatives
```rust
#[test]
fn test_REPL_013_003_explanation_with_alternatives() {
    // ARRANGE: Create alternatives
    let alternatives = vec![
        Alternative::new("Alt 1", "ex1", "when1"),
    ];

    // ACT: Add to explanation
    let explanation = TransformationExplanation::new(
        TransformationCategory::Idempotency,
        "mkdir -p",
        "mkdir /tmp",
        "mkdir -p /tmp",
        "Added -p",
        "Idempotent"
    )
    .with_alternatives(alternatives.clone());

    // ASSERT: Alternatives attached
    assert_eq!(explanation.alternatives, alternatives);
    assert_eq!(explanation.alternatives.len(), 1);
}
```

## Property Tests

```rust
#[cfg(test)]
mod alternatives_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_alternatives_always_provided(
            transformation_type in 0usize..3,
        ) {
            let alternatives = match transformation_type {
                0 => generate_idempotency_alternatives("mkdir → mkdir -p"),
                1 => generate_determinism_alternatives("Remove $RANDOM"),
                _ => generate_safety_alternatives("Quote variables"),
            };

            // Should always provide at least one alternative
            prop_assert!(!alternatives.is_empty());

            // All alternatives should have required fields
            for alt in &alternatives {
                prop_assert!(!alt.approach.is_empty());
                prop_assert!(!alt.example.is_empty());
                prop_assert!(!alt.when_to_use.is_empty());
            }
        }

        #[test]
        fn prop_format_never_panics(
            alt_count in 0usize..10,
        ) {
            let mut alternatives = Vec::new();
            for i in 0..alt_count {
                alternatives.push(
                    Alternative::new(
                        format!("Approach {}", i),
                        format!("Example {}", i),
                        format!("When {}", i)
                    )
                );
            }

            // Should never panic
            let _ = format_alternatives(&alternatives);
        }
    }
}
```

## Quality Gates

- [ ] ✅ All unit tests pass (≥6 tests)
- [ ] ✅ All property tests pass (≥2 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function

## EXTREME TDD Methodology

### RED Phase
1. Write test: `test_REPL_013_003_alternatives_mkdir` → FAIL ❌
2. Write test: `test_REPL_013_003_alternatives_random` → FAIL ❌
3. Write test: `test_REPL_013_003_alternatives_quoting` → FAIL ❌
4. Write test: `test_REPL_013_003_alternative_builder` → FAIL ❌
5. Write test: `test_REPL_013_003_format_alternatives` → FAIL ❌
6. Write test: `test_REPL_013_003_explanation_with_alternatives` → FAIL ❌
7. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `Alternative` struct with builder pattern
2. Extend `TransformationExplanation` with alternatives field
3. Implement `generate_idempotency_alternatives()`
4. Implement `generate_determinism_alternatives()`
5. Implement `generate_safety_alternatives()`
6. Implement `format_alternatives()`
7. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract common patterns
2. Simplify alternative generation logic
3. Ensure complexity <10
4. Run clippy
5. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_alternatives_always_provided`
2. Add property test: `prop_format_never_panics`
3. Run property tests (100+ cases) → **PASS** ✅

## Acceptance Criteria

- [ ] `Alternative` struct with all fields
- [ ] Builder pattern for Alternative construction
- [ ] `TransformationExplanation` extended with alternatives field
- [ ] Generation functions for each category
- [ ] `format_alternatives()` formats alternatives
- [ ] All unit tests pass (6 tests)
- [ ] All property tests pass (2 tests)
- [ ] Zero clippy warnings
- [ ] Complexity <10

## Example Usage

```rust
// Get transformation explanations with alternatives
let explanations = explain_purification_changes_detailed("mkdir /tmp")?;

for explanation in explanations {
    println!("Transformation: {}", explanation.title);

    if !explanation.alternatives.is_empty() {
        println!("\nAlternatives:");
        println!("{}", format_alternatives(&explanation.alternatives));
    }
}
```

**Output**:
```
Transformation: mkdir → mkdir -p

Alternatives:

1. Use install -d instead of mkdir -p
   Example: install -d /path/to/dir
   When to use: When you need to set permissions during directory creation
   Pros:
     + Can set owner/group/permissions atomically
     + Part of coreutils, widely available
   Cons:
     - Less intuitive than mkdir for simple cases
     - More flags to remember for common tasks

2. Check before creating
   Example: [ -d /path ] || mkdir /path
   When to use: When you want explicit control over error handling
   Pros:
     + Explicit about what's happening
     + Can add custom logic between check and creation
   Cons:
     - Not atomic - race condition between check and create
     - More verbose than mkdir -p
```

## Timeline

- **Duration**: 4-6 hours
- **RED Phase**: 1 hour
- **GREEN Phase**: 2 hours
- **REFACTOR Phase**: 1 hour
- **PROPERTY Phase**: 1 hour
- **Documentation**: Included in ticket

## Dependencies

- ✅ REPL-013-001 (Detailed transformation explanations) - COMPLETE

## Blocked By

None

## Blocks

None (completes REPL-013 sprint milestone)

## Notes

- Alternatives should be practical and actionable
- Focus on real-world trade-offs
- Each alternative must have clear "when to use" guidance
- Pros/cons help users make informed decisions
