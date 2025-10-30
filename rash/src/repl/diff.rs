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
