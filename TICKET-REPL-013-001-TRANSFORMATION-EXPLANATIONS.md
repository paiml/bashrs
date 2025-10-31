# TICKET: REPL-013-001

## Title
Detailed Transformation Explanations

## Priority
**P1 - High** (First task in REPL-013 Purification Explainer sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-005-003 basic explanations exist)

## Context
Enhancing the existing `explain_purification_changes()` with structured, detailed transformation explanations.

**Current State** (REPL-005-003):
- ✅ Basic string-based explanations exist
- ✅ Covers: mkdir -p, rm -f, variable quoting, $RANDOM removal, timestamps
- ⚠️ Explanations are simple strings with no structure
- ⚠️ No categorization (idempotency vs determinism vs safety)
- ⚠️ No line number tracking
- ⚠️ No detailed rationale

**Goal**: Create structured `TransformationExplanation` type with:
- Transformation category (idempotency, determinism, safety)
- Original and transformed code snippets
- Line numbers where changes occurred
- Detailed "what changed" description
- Comprehensive "why it matters" rationale

**Purpose**: Foundation for REPL-013-002 (safety rationale) and REPL-013-003 (alternatives).

## Dependencies
- ✅ REPL-005-003 (Basic purification explanations) completed
- ✅ `explain_purification_changes()` exists in purifier.rs
- ✅ `purify_bash()` function available

## Acceptance Criteria

### 1. Add `TransformationExplanation` struct

```rust
/// Category of transformation applied during purification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformationCategory {
    /// Makes code safe to re-run without side effects
    Idempotency,
    /// Makes code produce consistent results across runs
    Determinism,
    /// Prevents injection, race conditions, etc.
    Safety,
}

/// Detailed explanation of a single transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformationExplanation {
    /// Category of transformation
    pub category: TransformationCategory,
    /// Brief title of the transformation
    pub title: String,
    /// Original code snippet
    pub original: String,
    /// Transformed code snippet
    pub transformed: String,
    /// Detailed description of what changed
    pub what_changed: String,
    /// Explanation of why this change improves the code
    pub why_it_matters: String,
    /// Optional line number where transformation occurred
    pub line_number: Option<usize>,
}

impl TransformationExplanation {
    /// Create a new transformation explanation
    pub fn new(
        category: TransformationCategory,
        title: impl Into<String>,
        original: impl Into<String>,
        transformed: impl Into<String>,
        what_changed: impl Into<String>,
        why_it_matters: impl Into<String>,
    ) -> Self {
        Self {
            category,
            title: title.into(),
            original: original.into(),
            transformed: transformed.into(),
            what_changed: what_changed.into(),
            why_it_matters: why_it_matters.into(),
            line_number: None,
        }
    }

    /// Set line number where transformation occurred
    pub fn with_line_number(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }
}
```

### 2. Add `format_purification_report()` function

```rust
/// Format a detailed purification report from transformation explanations
///
/// Takes a list of transformations and formats them into a user-friendly report.
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::{TransformationExplanation, TransformationCategory, format_purification_report};
///
/// let transformations = vec![
///     TransformationExplanation::new(
///         TransformationCategory::Idempotency,
///         "mkdir -p transformation",
///         "mkdir /tmp/test",
///         "mkdir -p /tmp/test",
///         "Added -p flag to mkdir command",
///         "Prevents script failure if directory already exists"
///     ),
/// ];
///
/// let report = format_purification_report(&transformations);
/// assert!(report.contains("Idempotency"));
/// assert!(report.contains("mkdir -p"));
/// ```
pub fn format_purification_report(transformations: &[TransformationExplanation]) -> String {
    let mut output = String::new();

    if transformations.is_empty() {
        return "No transformations applied - code is already purified.".to_string();
    }

    output.push_str("📋 Purification Report\n\n");

    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<&TransformationExplanation>> =
        std::collections::HashMap::new();

    for transformation in transformations {
        let category_name = match transformation.category {
            TransformationCategory::Idempotency => "Idempotency",
            TransformationCategory::Determinism => "Determinism",
            TransformationCategory::Safety => "Safety",
        };
        by_category
            .entry(category_name.to_string())
            .or_default()
            .push(transformation);
    }

    // Display by category
    for category in ["Idempotency", "Determinism", "Safety"] {
        if let Some(transforms) = by_category.get(category) {
            output.push_str(&format!("## {} Improvements ({})\n\n", category, transforms.len()));

            for (i, t) in transforms.iter().enumerate() {
                output.push_str(&format!("### {}. {}\n", i + 1, t.title));

                if let Some(line) = t.line_number {
                    output.push_str(&format!("   📍 Line {}\n", line));
                }

                output.push_str("\n**What Changed:**\n");
                output.push_str(&format!("   {}\n\n", t.what_changed));

                output.push_str("**Original:**\n");
                output.push_str(&format!("   ```bash\n   {}\n   ```\n\n", t.original));

                output.push_str("**Transformed:**\n");
                output.push_str(&format!("   ```bash\n   {}\n   ```\n\n", t.transformed));

                output.push_str("**Why It Matters:**\n");
                output.push_str(&format!("   {}\n\n", t.why_it_matters));

                if i < transforms.len() - 1 {
                    output.push_str("---\n\n");
                }
            }
            output.push('\n');
        }
    }

    // Summary
    output.push_str(&format!(
        "📊 Total: {} transformation(s) applied\n",
        transformations.len()
    ));

    output
}
```

### 3. Enhance `explain_purification_changes()` to return structured data

```rust
/// Explain what changed during purification with detailed transformations
///
/// Returns a list of structured transformation explanations instead of plain text.
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::explain_purification_changes_detailed;
///
/// let transformations = explain_purification_changes_detailed("mkdir /tmp/test").unwrap();
/// assert!(!transformations.is_empty());
/// assert_eq!(transformations[0].category, TransformationCategory::Idempotency);
/// ```
pub fn explain_purification_changes_detailed(
    original: &str,
) -> anyhow::Result<Vec<TransformationExplanation>> {
    // Purify the bash code
    let purified = purify_bash(original)?;

    // If no changes, return empty vector
    if original.trim() == purified.trim() {
        return Ok(Vec::new());
    }

    let mut transformations = Vec::new();

    // Detect mkdir -p transformation
    if original.contains("mkdir") && !original.contains("mkdir -p") && purified.contains("mkdir -p")
    {
        transformations.push(
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir -p transformation",
                original.trim(),
                purified.trim(),
                "Added -p flag to mkdir command",
                "Prevents script failure if directory already exists. \
                 Makes the script safe to re-run without manual cleanup. \
                 The -p flag creates parent directories as needed and \
                 silently succeeds if the directory exists."
            )
        );
    }

    // Detect rm -f transformation
    if original.contains("rm ") && !original.contains("rm -f") && purified.contains("rm -f") {
        transformations.push(
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "rm -f transformation",
                original.trim(),
                purified.trim(),
                "Added -f flag to rm command",
                "Prevents script failure if file doesn't exist. \
                 Makes deletion idempotent - running multiple times \
                 produces the same result. The -f flag forces removal \
                 and silently succeeds if file is already gone."
            )
        );
    }

    // Detect variable quoting
    let original_has_unquoted = original.contains("$") && !original.contains("\"$");
    let purified_has_quoted = purified.contains("\"$");
    if original_has_unquoted && purified_has_quoted {
        transformations.push(
            TransformationExplanation::new(
                TransformationCategory::Safety,
                "Variable quoting",
                original.trim(),
                purified.trim(),
                "Added quotes around variable references",
                "Prevents word splitting and glob expansion issues. \
                 Unquoted variables can cause unexpected behavior when \
                 they contain spaces or special characters. Quoting \
                 ensures the variable value is treated as a single unit."
            )
        );
    }

    // Detect $RANDOM removal
    if original.contains("$RANDOM") && !purified.contains("$RANDOM") {
        transformations.push(
            TransformationExplanation::new(
                TransformationCategory::Determinism,
                "$RANDOM removal",
                original.trim(),
                purified.trim(),
                "Removed $RANDOM variable usage",
                "Makes script behavior deterministic and reproducible. \
                 Random values make testing and debugging difficult \
                 because the same input produces different outputs. \
                 Consider using a seed-based approach or external \
                 configuration for values that need to vary."
            )
        );
    }

    // Detect timestamp removal
    if (original.contains("date") || original.contains("$SECONDS"))
        && (!purified.contains("date") || !purified.contains("$SECONDS"))
    {
        transformations.push(
            TransformationExplanation::new(
                TransformationCategory::Determinism,
                "Timestamp removal",
                original.trim(),
                purified.trim(),
                "Removed time-based value generation",
                "Makes script output reproducible across runs. \
                 Timestamps make scripts non-deterministic because \
                 the same input at different times produces different \
                 outputs. Consider passing timestamps as parameters \
                 or using version numbers instead."
            )
        );
    }

    Ok(transformations)
}
```

### 4. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]
mod transformation_explanation_tests {
    use super::*;

    // ===== REPL-013-001: TRANSFORMATION EXPLANATION TESTS =====

    #[test]
    fn test_REPL_013_001_transformation_category_display() {
        // ARRANGE: Create categories
        let idempotency = TransformationCategory::Idempotency;
        let determinism = TransformationCategory::Determinism;
        let safety = TransformationCategory::Safety;

        // ASSERT: Categories are distinct
        assert_ne!(idempotency, determinism);
        assert_ne!(determinism, safety);
        assert_ne!(safety, idempotency);
    }

    #[test]
    fn test_REPL_013_001_transformation_explanation_new() {
        // ARRANGE & ACT: Create transformation explanation
        let explanation = TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir -p",
            "mkdir /tmp",
            "mkdir -p /tmp",
            "Added -p flag",
            "Prevents failure if exists"
        );

        // ASSERT: All fields set correctly
        assert_eq!(explanation.category, TransformationCategory::Idempotency);
        assert_eq!(explanation.title, "mkdir -p");
        assert_eq!(explanation.original, "mkdir /tmp");
        assert_eq!(explanation.transformed, "mkdir -p /tmp");
        assert_eq!(explanation.what_changed, "Added -p flag");
        assert_eq!(explanation.why_it_matters, "Prevents failure if exists");
        assert_eq!(explanation.line_number, None);
    }

    #[test]
    fn test_REPL_013_001_transformation_with_line_number() {
        // ARRANGE & ACT: Create with line number
        let explanation = TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            "echo $var",
            "echo \"$var\"",
            "Added quotes",
            "Prevents splitting"
        )
        .with_line_number(42);

        // ASSERT: Line number set
        assert_eq!(explanation.line_number, Some(42));
    }

    #[test]
    fn test_REPL_013_001_explain_mkdir_p_detailed() {
        // ARRANGE: Code that needs mkdir -p
        let original = "mkdir /tmp/test";

        // ACT: Get detailed explanations
        let transformations = explain_purification_changes_detailed(original).unwrap();

        // ASSERT: Contains mkdir -p transformation
        assert!(!transformations.is_empty());
        assert_eq!(transformations[0].category, TransformationCategory::Idempotency);
        assert!(transformations[0].title.contains("mkdir"));
        assert!(transformations[0].what_changed.contains("-p"));
        assert!(transformations[0].why_it_matters.contains("re-run"));
    }

    #[test]
    fn test_REPL_013_001_explain_quote_var_detailed() {
        // ARRANGE: Code with unquoted variable
        let original = "echo $HOME";

        // ACT: Get detailed explanations
        let transformations = explain_purification_changes_detailed(original).unwrap();

        // ASSERT: Contains quoting transformation
        assert!(!transformations.is_empty());
        let quoting = transformations.iter().find(|t| t.category == TransformationCategory::Safety);
        assert!(quoting.is_some());
        let quoting = quoting.unwrap();
        assert!(quoting.title.contains("quot"));
        assert!(quoting.why_it_matters.contains("split"));
    }

    #[test]
    fn test_REPL_013_001_explain_random_removal_detailed() {
        // ARRANGE: Code with $RANDOM
        let original = "ID=$RANDOM";

        // ACT: Get detailed explanations
        let transformations = explain_purification_changes_detailed(original).unwrap();

        // ASSERT: Contains $RANDOM removal transformation
        assert!(!transformations.is_empty());
        let random_removal = transformations.iter().find(|t| {
            t.category == TransformationCategory::Determinism && t.title.contains("RANDOM")
        });
        assert!(random_removal.is_some());
        let random_removal = random_removal.unwrap();
        assert!(random_removal.why_it_matters.contains("deterministic"));
    }

    #[test]
    fn test_REPL_013_001_format_empty_report() {
        // ARRANGE: Empty transformations
        let transformations: Vec<TransformationExplanation> = vec![];

        // ACT: Format report
        let report = format_purification_report(&transformations);

        // ASSERT: Shows "no transformations" message
        assert!(report.contains("No transformations") || report.contains("already purified"));
    }

    #[test]
    fn test_REPL_013_001_format_single_transformation() {
        // ARRANGE: Single transformation
        let transformations = vec![
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir -p",
                "mkdir /tmp",
                "mkdir -p /tmp",
                "Added -p flag",
                "Safe to re-run"
            )
            .with_line_number(5)
        ];

        // ACT: Format report
        let report = format_purification_report(&transformations);

        // ASSERT: Report contains all key information
        assert!(report.contains("Purification Report"));
        assert!(report.contains("Idempotency"));
        assert!(report.contains("mkdir -p"));
        assert!(report.contains("Line 5"));
        assert!(report.contains("What Changed"));
        assert!(report.contains("Why It Matters"));
        assert!(report.contains("Total: 1 transformation"));
    }

    #[test]
    fn test_REPL_013_001_format_multiple_categories() {
        // ARRANGE: Multiple transformations across categories
        let transformations = vec![
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir -p",
                "mkdir /tmp",
                "mkdir -p /tmp",
                "Added -p",
                "Idempotent"
            ),
            TransformationExplanation::new(
                TransformationCategory::Determinism,
                "$RANDOM removal",
                "ID=$RANDOM",
                "ID=fixed",
                "Removed random",
                "Deterministic"
            ),
            TransformationExplanation::new(
                TransformationCategory::Safety,
                "Quote var",
                "echo $var",
                "echo \"$var\"",
                "Added quotes",
                "Safe"
            ),
        ];

        // ACT: Format report
        let report = format_purification_report(&transformations);

        // ASSERT: All categories present
        assert!(report.contains("Idempotency"));
        assert!(report.contains("Determinism"));
        assert!(report.contains("Safety"));
        assert!(report.contains("Total: 3 transformation"));
    }

    #[test]
    fn test_REPL_013_001_no_changes_returns_empty() {
        // ARRANGE: Already purified code
        let original = "echo 'hello'";

        // ACT: Get explanations
        let transformations = explain_purification_changes_detailed(original).unwrap();

        // ASSERT: Empty vector for no changes
        assert!(transformations.is_empty());
    }
}
```

### 5. Property Tests

```rust
#[cfg(test)]
mod transformation_explanation_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_013_001_explanation_new_never_panics(
            title in ".{0,100}",
            original in ".{0,200}",
            transformed in ".{0,200}",
            what in ".{0,200}",
            why in ".{0,300}",
        ) {
            // Should never panic creating explanations
            let _ = TransformationExplanation::new(
                TransformationCategory::Idempotency,
                title,
                original,
                transformed,
                what,
                why
            );
        }

        #[test]
        fn prop_REPL_013_001_format_report_never_panics(
            count in 0usize..10,
        ) {
            let transformations: Vec<TransformationExplanation> = (0..count)
                .map(|i| {
                    TransformationExplanation::new(
                        TransformationCategory::Idempotency,
                        format!("Transform {}", i),
                        "original",
                        "transformed",
                        "what changed",
                        "why it matters"
                    )
                })
                .collect();

            let report = format_purification_report(&transformations);

            // Should contain transformation count
            prop_assert!(report.contains(&count.to_string()));
        }

        #[test]
        fn prop_REPL_013_001_explain_detailed_never_panics(
            input in ".*{0,500}",
        ) {
            // Should never panic on any input
            let _ = explain_purification_changes_detailed(&input);
        }

        #[test]
        fn prop_REPL_013_001_line_numbers_always_positive(
            line in 1usize..1000,
        ) {
            let explanation = TransformationExplanation::new(
                TransformationCategory::Safety,
                "test",
                "a",
                "b",
                "c",
                "d"
            )
            .with_line_number(line);

            prop_assert_eq!(explanation.line_number, Some(line));
        }
    }
}
```

### 6. Quality Gates

- [ ] ✅ All unit tests pass (≥10 tests)
- [ ] ✅ All property tests pass (≥4 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_013_001_transformation_category_display`
2. Write failing test: `test_REPL_013_001_transformation_explanation_new`
3. Write failing test: `test_REPL_013_001_transformation_with_line_number`
4. Write failing test: `test_REPL_013_001_explain_mkdir_p_detailed`
5. Write failing test: `test_REPL_013_001_explain_quote_var_detailed`
6. Write failing test: `test_REPL_013_001_explain_random_removal_detailed`
7. Write failing test: `test_REPL_013_001_format_empty_report`
8. Write failing test: `test_REPL_013_001_format_single_transformation`
9. Write failing test: `test_REPL_013_001_format_multiple_categories`
10. Write failing test: `test_REPL_013_001_no_changes_returns_empty`
11. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `TransformationCategory` enum
2. Implement `TransformationExplanation` struct
3. Implement `explain_purification_changes_detailed()` function
4. Implement `format_purification_report()` function
5. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract transformation detection helpers if needed
2. Ensure error handling is robust
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_013_001_explanation_new_never_panics`
2. Add property test: `prop_REPL_013_001_format_report_never_panics`
3. Add property test: `prop_REPL_013_001_explain_detailed_never_panics`
4. Add property test: `prop_REPL_013_001_line_numbers_always_positive`
5. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/purifier.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/purifier.rs` - Add new types and enhance function
- `rash/src/repl/mod.rs` - Export new types

### Files to Create
- None (extends existing purifier module)

### Test Files
- `rash/src/repl/purifier.rs` - Unit tests in module
- `rash/src/repl/purifier.rs` - Property tests in module

## Task Breakdown

- [ ] **Task 1**: Write RED tests for transformation explanations
- [ ] **Task 2**: Implement `TransformationCategory` and `TransformationExplanation` (GREEN)
- [ ] **Task 3**: Implement `explain_purification_changes_detailed()` (GREEN)
- [ ] **Task 4**: Implement `format_purification_report()` (GREEN)
- [ ] **Task 5**: Refactor if needed (REFACTOR phase)
- [ ] **Task 6**: Add property tests (PROPERTY phase)
- [ ] **Task 7**: Verify all quality gates
- [ ] **Task 8**: Update roadmap (mark REPL-013-001 complete)
- [ ] **Task 9**: Commit with proper message

## Example Usage

```bash
$ bashrs purify deploy.sh --explain

📋 Purification Report

## Idempotency Improvements (2)

### 1. mkdir -p transformation
   📍 Line 12

**What Changed:**
   Added -p flag to mkdir command

**Original:**
   ```bash
   mkdir /app/releases/v1.0
   ```

**Transformed:**
   ```bash
   mkdir -p /app/releases/v1.0
   ```

**Why It Matters:**
   Prevents script failure if directory already exists. Makes the script safe to re-run
   without manual cleanup. The -p flag creates parent directories as needed and silently
   succeeds if the directory exists.

---

### 2. rm -f transformation
   📍 Line 15

**What Changed:**
   Added -f flag to rm command

**Original:**
   ```bash
   rm /app/current
   ```

**Transformed:**
   ```bash
   rm -f /app/current
   ```

**Why It Matters:**
   Prevents script failure if file doesn't exist. Makes deletion idempotent - running
   multiple times produces the same result. The -f flag forces removal and silently
   succeeds if file is already gone.

## Determinism Improvements (1)

### 1. $RANDOM removal
   📍 Line 8

**What Changed:**
   Removed $RANDOM variable usage

**Original:**
   ```bash
   SESSION_ID=$RANDOM
   ```

**Transformed:**
   ```bash
   SESSION_ID="${VERSION}"
   ```

**Why It Matters:**
   Makes script behavior deterministic and reproducible. Random values make testing
   and debugging difficult because the same input produces different outputs. Consider
   using a seed-based approach or external configuration for values that need to vary.

📊 Total: 3 transformation(s) applied
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures transformation explanations are correct from first line
- Structured data types enforce consistency

### 反省 (Hansei) - Reflect and Improve
- Learn from existing `explain_purification_changes()` (REPL-005-003)
- Enhance with structured explanations and categorization

### 改善 (Kaizen) - Continuous Improvement
- Foundation for REPL-013-002 (safety rationale)
- Foundation for REPL-013-003 (alternative suggestions)

### 現地現物 (Genchi Genbutsu) - Go and See
- Detailed "what changed" shows exactly what happened
- "Why it matters" explains the impact to users

## Related Files
- `rash/src/repl/purifier.rs` - Existing purification logic (REPL-005-003)
- `rash/src/repl/explain.rs` - Bash construct explanations (REPL-005-002)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Success Criteria Summary
```
BEFORE: Simple string-based explanations
AFTER:  ✅ Structured TransformationExplanation type
        ✅ Categorization (idempotency, determinism, safety)
        ✅ Detailed "what changed" descriptions
        ✅ Comprehensive "why it matters" rationale
        ✅ Formatted report with format_purification_report()
        ✅ All quality gates passed
        ✅ Property tests validate explanation logic
        ✅ Foundation for REPL-013-002 and REPL-013-003
```

---

**Created**: 2025-10-31
**Sprint**: REPL-013 (Purification Explainer)
**Estimated Time**: 2-3 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
