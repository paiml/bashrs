// REPL Breakpoint System
//
// Task: REPL-007-001 - Line-based breakpoints
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Property tests: Line number validation
// - Mutation score: ≥90%
// - Complexity: <10 per function

use std::collections::HashSet;

/// Line-based breakpoint
///
/// Represents a breakpoint at a specific line number in a bash script.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    pub line: usize,
    pub enabled: bool,
}

impl Breakpoint {
    /// Create a new breakpoint at the specified line
    pub fn new(line: usize) -> Self {
        Self {
            line,
            enabled: true,
        }
    }

    /// Disable this breakpoint
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable this breakpoint
    pub fn enable(&mut self) {
        self.enabled = true;
    }
}

/// Breakpoint manager - manages collection of breakpoints
///
/// Handles setting, removing, and checking breakpoints during script execution.
#[derive(Debug, Default)]
pub struct BreakpointManager {
    breakpoints: HashSet<usize>,
}

impl BreakpointManager {
    /// Create a new empty breakpoint manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a breakpoint at the specified line
    ///
    /// Returns true if breakpoint was newly added, false if it already existed.
    pub fn set_breakpoint(&mut self, line: usize) -> bool {
        self.breakpoints.insert(line)
    }

    /// Remove a breakpoint at the specified line
    ///
    /// Returns true if breakpoint was removed, false if it didn't exist.
    pub fn remove_breakpoint(&mut self, line: usize) -> bool {
        self.breakpoints.remove(&line)
    }

    /// Check if there's a breakpoint at the specified line
    pub fn is_breakpoint_hit(&self, line: usize) -> bool {
        self.breakpoints.contains(&line)
    }

    /// Get all breakpoint line numbers
    pub fn get_breakpoints(&self) -> Vec<usize> {
        let mut lines: Vec<usize> = self.breakpoints.iter().copied().collect();
        lines.sort_unstable();
        lines
    }

    /// Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.breakpoints.clear();
    }

    /// Get the number of active breakpoints
    pub fn count(&self) -> usize {
        self.breakpoints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-007-001-001 - Set a breakpoint
    #[test]
    fn test_REPL_007_001_set_breakpoint() {
        let mut manager = BreakpointManager::new();

        // Set breakpoint at line 10
        let added = manager.set_breakpoint(10);
        assert!(added, "Should return true when adding new breakpoint");

        // Verify breakpoint is set
        assert!(
            manager.is_breakpoint_hit(10),
            "Breakpoint at line 10 should be set"
        );
        assert_eq!(manager.count(), 1, "Should have 1 breakpoint");

        // Setting same breakpoint again should return false
        let added_again = manager.set_breakpoint(10);
        assert!(
            !added_again,
            "Should return false when breakpoint already exists"
        );
        assert_eq!(manager.count(), 1, "Should still have 1 breakpoint");
    }

    /// Test: REPL-007-001-002 - Hit a breakpoint
    #[test]
    fn test_REPL_007_001_hit_breakpoint() {
        let mut manager = BreakpointManager::new();

        // Set breakpoint at line 5
        manager.set_breakpoint(5);

        // Check various lines
        assert!(
            manager.is_breakpoint_hit(5),
            "Should hit breakpoint at line 5"
        );
        assert!(
            !manager.is_breakpoint_hit(4),
            "Should not hit breakpoint at line 4"
        );
        assert!(
            !manager.is_breakpoint_hit(6),
            "Should not hit breakpoint at line 6"
        );
        assert!(
            !manager.is_breakpoint_hit(100),
            "Should not hit breakpoint at line 100"
        );
    }

    /// Test: REPL-007-001-003 - Remove a breakpoint
    #[test]
    fn test_REPL_007_001_remove_breakpoint() {
        let mut manager = BreakpointManager::new();

        // Set breakpoint at line 15
        manager.set_breakpoint(15);
        assert!(
            manager.is_breakpoint_hit(15),
            "Breakpoint at line 15 should be set"
        );

        // Remove the breakpoint
        let removed = manager.remove_breakpoint(15);
        assert!(removed, "Should return true when removing existing breakpoint");

        // Verify breakpoint is removed
        assert!(
            !manager.is_breakpoint_hit(15),
            "Breakpoint at line 15 should be removed"
        );
        assert_eq!(manager.count(), 0, "Should have 0 breakpoints");

        // Removing non-existent breakpoint should return false
        let removed_again = manager.remove_breakpoint(15);
        assert!(
            !removed_again,
            "Should return false when breakpoint doesn't exist"
        );
    }

    /// Test: REPL-007-001-004 - Multiple breakpoints
    #[test]
    fn test_REPL_007_001_multiple_breakpoints() {
        let mut manager = BreakpointManager::new();

        // Set multiple breakpoints
        manager.set_breakpoint(1);
        manager.set_breakpoint(5);
        manager.set_breakpoint(10);
        manager.set_breakpoint(15);

        assert_eq!(manager.count(), 4, "Should have 4 breakpoints");

        // Check all breakpoints are hit
        assert!(manager.is_breakpoint_hit(1));
        assert!(manager.is_breakpoint_hit(5));
        assert!(manager.is_breakpoint_hit(10));
        assert!(manager.is_breakpoint_hit(15));

        // Check non-breakpoint lines
        assert!(!manager.is_breakpoint_hit(2));
        assert!(!manager.is_breakpoint_hit(3));
    }

    /// Test: REPL-007-001-005 - Get all breakpoints
    #[test]
    fn test_REPL_007_001_get_breakpoints() {
        let mut manager = BreakpointManager::new();

        // Set breakpoints in random order
        manager.set_breakpoint(15);
        manager.set_breakpoint(5);
        manager.set_breakpoint(10);
        manager.set_breakpoint(1);

        // Get all breakpoints (should be sorted)
        let breakpoints = manager.get_breakpoints();
        assert_eq!(breakpoints, vec![1, 5, 10, 15], "Breakpoints should be sorted");
    }

    /// Test: REPL-007-001-006 - Clear all breakpoints
    #[test]
    fn test_REPL_007_001_clear_all() {
        let mut manager = BreakpointManager::new();

        // Set multiple breakpoints
        manager.set_breakpoint(1);
        manager.set_breakpoint(5);
        manager.set_breakpoint(10);
        assert_eq!(manager.count(), 3, "Should have 3 breakpoints");

        // Clear all
        manager.clear_all();
        assert_eq!(manager.count(), 0, "Should have 0 breakpoints after clear");
        assert!(!manager.is_breakpoint_hit(1));
        assert!(!manager.is_breakpoint_hit(5));
        assert!(!manager.is_breakpoint_hit(10));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: Line numbers are always valid (non-zero, reasonable range)
    proptest! {
        #[test]
        fn prop_breakpoint_line_numbers_valid(line in 1usize..10000) {
            let mut manager = BreakpointManager::new();

            // Set breakpoint at any valid line
            manager.set_breakpoint(line);

            // Should be able to hit it
            prop_assert!(
                manager.is_breakpoint_hit(line),
                "Should be able to set and hit breakpoint at line {}",
                line
            );
        }
    }

    /// Property: set_breakpoint is idempotent (setting twice is same as setting once)
    proptest! {
        #[test]
        fn prop_set_breakpoint_idempotent(line in 1usize..10000) {
            let mut manager = BreakpointManager::new();

            // Set breakpoint twice
            manager.set_breakpoint(line);
            manager.set_breakpoint(line);

            // Should only have 1 breakpoint
            prop_assert_eq!(manager.count(), 1, "Setting same breakpoint twice should result in 1 breakpoint");
            prop_assert!(manager.is_breakpoint_hit(line));
        }
    }

    /// Property: remove_breakpoint removes the breakpoint
    proptest! {
        #[test]
        fn prop_remove_breakpoint_works(line in 1usize..10000) {
            let mut manager = BreakpointManager::new();

            // Set and then remove
            manager.set_breakpoint(line);
            manager.remove_breakpoint(line);

            // Should not hit breakpoint anymore
            prop_assert!(
                !manager.is_breakpoint_hit(line),
                "After removal, breakpoint at line {} should not be hit",
                line
            );
            prop_assert_eq!(manager.count(), 0, "Count should be 0 after removing only breakpoint");
        }
    }

    /// Property: count is always accurate
    proptest! {
        #[test]
        fn prop_count_accurate(lines in prop::collection::vec(1usize..1000, 1..20)) {
            let mut manager = BreakpointManager::new();

            // Set all breakpoints
            for &line in &lines {
                manager.set_breakpoint(line);
            }

            // Count should match unique lines (HashSet removes duplicates)
            let unique_lines: std::collections::HashSet<_> = lines.iter().collect();
            prop_assert_eq!(
                manager.count(),
                unique_lines.len(),
                "Count should match number of unique breakpoints"
            );
        }
    }

    /// Property: get_breakpoints returns sorted list
    proptest! {
        #[test]
        fn prop_get_breakpoints_sorted(lines in prop::collection::vec(1usize..1000, 1..20)) {
            let mut manager = BreakpointManager::new();

            // Set breakpoints in random order
            for &line in &lines {
                manager.set_breakpoint(line);
            }

            // Get breakpoints
            let breakpoints = manager.get_breakpoints();

            // Verify sorted
            for i in 1..breakpoints.len() {
                prop_assert!(
                    breakpoints[i - 1] < breakpoints[i],
                    "Breakpoints should be sorted: {:?}",
                    breakpoints
                );
            }
        }
    }

    /// Property: clear_all removes all breakpoints
    proptest! {
        #[test]
        fn prop_clear_all_works(lines in prop::collection::vec(1usize..1000, 1..20)) {
            let mut manager = BreakpointManager::new();

            // Set multiple breakpoints
            for &line in &lines {
                manager.set_breakpoint(line);
            }

            // Clear all
            manager.clear_all();

            // Verify all removed
            prop_assert_eq!(manager.count(), 0, "Count should be 0 after clear_all");
            for &line in &lines {
                prop_assert!(
                    !manager.is_breakpoint_hit(line),
                    "No breakpoint should be hit after clear_all"
                );
            }
        }
    }

    /// Property: is_breakpoint_hit is deterministic
    proptest! {
        #[test]
        fn prop_is_breakpoint_hit_deterministic(line in 1usize..10000) {
            let mut manager = BreakpointManager::new();
            manager.set_breakpoint(line);

            // Check multiple times - should always return true
            let result1 = manager.is_breakpoint_hit(line);
            let result2 = manager.is_breakpoint_hit(line);
            let result3 = manager.is_breakpoint_hit(line);

            prop_assert_eq!(result1, result2, "is_breakpoint_hit should be deterministic");
            prop_assert_eq!(result2, result3, "is_breakpoint_hit should be deterministic");
        }
    }

    /// Property: Breakpoint struct properties
    proptest! {
        #[test]
        fn prop_breakpoint_struct(line in 1usize..10000) {
            let breakpoint = Breakpoint::new(line);

            prop_assert_eq!(breakpoint.line, line, "Breakpoint line should match");
            prop_assert!(breakpoint.enabled, "New breakpoints should be enabled");

            // Disable and check
            let mut bp = breakpoint.clone();
            bp.disable();
            prop_assert!(!bp.enabled, "Disabled breakpoint should not be enabled");

            // Re-enable and check
            bp.enable();
            prop_assert!(bp.enabled, "Re-enabled breakpoint should be enabled");
        }
    }
}
