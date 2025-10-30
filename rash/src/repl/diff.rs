// REPL Diff Display Module
//
// Task: REPL-005-002 - Show original vs purified side-by-side
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Property tests: 2+ generators
// - Complexity: <10 per function

use crate::repl::purifier::purify_bash;

/// Display original and purified bash side-by-side
///
/// # Examples
///
/// ```
/// use bashrs::repl::diff::display_diff;
///
/// let original = "mkdir /tmp/test";
/// let result = display_diff(original);
/// assert!(result.is_ok());
/// ```
pub fn display_diff(original: &str) -> anyhow::Result<String> {
    // Purify the bash code
    let purified = purify_bash(original)?;

    // Build side-by-side diff display
    let mut output = String::new();
    output.push_str("Original → Purified\n");
    output.push_str("─────────────────────\n");

    // Show original with - marker
    output.push_str("- ");
    output.push_str(original);
    output.push('\n');

    // Show purified with + marker
    output.push_str("+ ");
    output.push_str(&purified);
    output.push('\n');

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-005-002-001 - Basic diff display
    #[test]
    fn test_REPL_005_002_diff_display() {
        let original = "mkdir /tmp/test";
        let result = display_diff(original);

        assert!(result.is_ok(), "Should display diff: {:?}", result);
        let diff = result.unwrap();

        // Should show original and purified side-by-side
        assert!(
            diff.contains("mkdir /tmp/test") && diff.contains("mkdir -p"),
            "Should show both original and purified: {}",
            diff
        );
    }

    /// Test: REPL-005-002-002 - Diff highlighting with markers
    #[test]
    fn test_REPL_005_002_diff_highlighting() {
        let original = "echo $RANDOM";
        let result = display_diff(original);

        assert!(result.is_ok(), "Should display diff with highlighting: {:?}", result);
        let diff = result.unwrap();

        // Should have markers or indicators for changes
        assert!(
            diff.contains("-") || diff.contains("+") || diff.contains("|"),
            "Should have diff markers: {}",
            diff
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: display_diff should never panic on any input
    proptest! {
        #[test]
        fn prop_diff_never_panics(input in ".*{0,1000}") {
            // Test that diff display gracefully handles any input without panicking
            let _ = display_diff(&input);
            // If we get here without panic, test passes
        }
    }

    /// Property: display_diff should be deterministic
    proptest! {
        #[test]
        fn prop_diff_deterministic(input in "[a-z ]{1,50}") {
            // Same input should always produce same output
            let result1 = display_diff(&input);
            let result2 = display_diff(&input);

            match (result1, result2) {
                (Ok(out1), Ok(out2)) => {
                    prop_assert_eq!(out1, out2, "Diff display should be deterministic");
                }
                (Err(_), Err(_)) => {
                    // Both failed - consistent behavior
                }
                _ => {
                    prop_assert!(false, "Inconsistent results for same input");
                }
            }
        }
    }

    /// Property: diff output always contains markers
    proptest! {
        #[test]
        fn prop_diff_has_markers(input in "[a-z ]{1,30}") {
            if let Ok(diff) = display_diff(&input) {
                // If diff succeeded, should have - and + markers
                prop_assert!(
                    diff.contains("-") && diff.contains("+"),
                    "Diff should have - and + markers: {}",
                    diff
                );
            }
        }
    }

    /// Property: diff output preserves original input
    proptest! {
        #[test]
        fn prop_diff_preserves_original(input in "[a-z ]{1,30}") {
            if let Ok(diff) = display_diff(&input) {
                // Original input should appear in diff output
                prop_assert!(
                    diff.contains(&input),
                    "Diff should contain original input '{}': {}",
                    input,
                    diff
                );
            }
        }
    }

    /// Property: diff output always has header
    proptest! {
        #[test]
        fn prop_diff_has_header(input in "[a-z ]{1,30}") {
            if let Ok(diff) = display_diff(&input) {
                // Should have header showing original vs purified
                prop_assert!(
                    diff.contains("Original") || diff.contains("Purified"),
                    "Diff should have header: {}",
                    diff
                );
            }
        }
    }
}
