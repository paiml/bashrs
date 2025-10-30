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

use std::collections::{HashMap, HashSet};

/// Line-based breakpoint with optional condition and hit-count support
///
/// Represents a breakpoint at a specific line number in a bash script.
/// Can have an optional condition that must evaluate to true for the breakpoint to trigger.
/// Can also have a hit-count threshold - breakpoint only triggers after N hits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    pub line: usize,
    pub enabled: bool,
    pub condition: Option<String>,
    pub hit_count: usize,
    pub hit_threshold: Option<usize>,
}

impl Breakpoint {
    /// Create a new breakpoint at the specified line
    pub fn new(line: usize) -> Self {
        Self {
            line,
            enabled: true,
            condition: None,
            hit_count: 0,
            hit_threshold: None,
        }
    }

    /// Create a new conditional breakpoint
    pub fn with_condition(line: usize, condition: String) -> Self {
        Self {
            line,
            enabled: true,
            condition: Some(condition),
            hit_count: 0,
            hit_threshold: None,
        }
    }

    /// Create a new hit-count breakpoint
    ///
    /// The breakpoint will trigger after it has been hit `threshold` times.
    pub fn with_hit_count(line: usize, threshold: usize) -> Self {
        Self {
            line,
            enabled: true,
            condition: None,
            hit_count: 0,
            hit_threshold: Some(threshold),
        }
    }

    /// Create a new hit-count breakpoint with a condition
    ///
    /// The breakpoint will trigger after it has been hit `threshold` times AND the condition is true.
    pub fn with_hit_count_and_condition(line: usize, threshold: usize, condition: String) -> Self {
        Self {
            line,
            enabled: true,
            condition: Some(condition),
            hit_count: 0,
            hit_threshold: Some(threshold),
        }
    }

    /// Check if this breakpoint has a condition
    pub fn is_conditional(&self) -> bool {
        self.condition.is_some()
    }

    /// Evaluate the condition (if any) against provided variables
    ///
    /// Returns true if:
    /// - No condition is set (unconditional breakpoint)
    /// - Condition evaluates to true
    ///
    /// Returns false if:
    /// - Condition evaluates to false
    /// - Condition evaluation fails
    pub fn should_break(&self, variables: &std::collections::HashMap<String, String>) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.condition {
            None => true, // Unconditional breakpoint always triggers
            Some(cond) => evaluate_condition(cond, variables),
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

    /// Check if this breakpoint should trigger, incrementing hit count
    ///
    /// This method increments the hit count and returns true if:
    /// - Breakpoint is enabled
    /// - Hit count threshold is reached (if set)
    /// - Condition evaluates to true (if set)
    ///
    /// Returns false if any of these conditions fail.
    pub fn should_break_with_hit(&mut self, variables: &HashMap<String, String>) -> bool {
        if !self.enabled {
            return false;
        }

        // Increment hit count
        self.hit_count += 1;

        // Check hit threshold if set
        if let Some(threshold) = self.hit_threshold {
            if self.hit_count < threshold {
                return false;
            }
        }

        // Check condition if set
        match &self.condition {
            None => true, // No condition, trigger
            Some(cond) => evaluate_condition(cond, variables),
        }
    }

    /// Get the current hit count
    pub fn get_hit_count(&self) -> usize {
        self.hit_count
    }

    /// Reset the hit count to zero
    pub fn reset_hit_count(&mut self) {
        self.hit_count = 0;
    }
}

/// Breakpoint manager - manages collection of breakpoints
///
/// Handles setting, removing, and checking breakpoints during script execution.
#[derive(Debug, Default, Clone)]
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

/// Simple condition evaluator for breakpoint conditions
///
/// Supports basic comparisons like:
/// - "$var > 10" - Variable greater than value
/// - "$var < 5" - Variable less than value
/// - "$var == test" - Variable equals value
/// - "$var != foo" - Variable not equals value
///
/// Returns false if condition cannot be evaluated (invalid syntax, missing variable, etc.)
fn evaluate_condition(condition: &str, variables: &HashMap<String, String>) -> bool {
    let condition = condition.trim();

    // Try to parse simple comparison: $var OP value
    // Check for two-character operators first (==, !=, >=, <=)
    let (operator, var_part, value_part) = if let Some(pos) = condition.find("==") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        ("==", var_part, value_part)
    } else if let Some(pos) = condition.find("!=") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        ("!=", var_part, value_part)
    } else if let Some(pos) = condition.find(">=") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        (">=", var_part, value_part)
    } else if let Some(pos) = condition.find("<=") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        ("<=", var_part, value_part)
    } else if let Some(pos) = condition.find('>') {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 1..].trim();
        (">", var_part, value_part)
    } else if let Some(pos) = condition.find('<') {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 1..].trim();
        ("<", var_part, value_part)
    } else {
        return false; // No operator found
    };

    // Extract variable name (remove $)
    let var_name = if let Some(name) = var_part.strip_prefix('$') {
        name
    } else {
        return false; // Invalid syntax
    };

    // Get variable value
    let var_value = match variables.get(var_name) {
        Some(v) => v,
        None => return false, // Variable not found
    };

    // Perform comparison
    match operator {
        "==" => var_value == value_part,
        "!=" => var_value != value_part,
        ">" => {
            // Try numeric comparison
            if let (Ok(var_num), Ok(val_num)) = (var_value.parse::<i64>(), value_part.parse::<i64>()) {
                var_num > val_num
            } else {
                false
            }
        }
        "<" => {
            if let (Ok(var_num), Ok(val_num)) = (var_value.parse::<i64>(), value_part.parse::<i64>()) {
                var_num < val_num
            } else {
                false
            }
        }
        ">=" => {
            if let (Ok(var_num), Ok(val_num)) = (var_value.parse::<i64>(), value_part.parse::<i64>()) {
                var_num >= val_num
            } else {
                false
            }
        }
        "<=" => {
            if let (Ok(var_num), Ok(val_num)) = (var_value.parse::<i64>(), value_part.parse::<i64>()) {
                var_num <= val_num
            } else {
                false
            }
        }
        _ => false,
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

    // ===== CONDITIONAL BREAKPOINTS (REPL-007-002) =====

    /// Test: REPL-007-002-001 - Conditional breakpoint evaluates to true
    #[test]
    fn test_REPL_007_002_conditional_true() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "15".to_string());

        // Create conditional breakpoint: break if $count > 10
        let bp = Breakpoint::with_condition(5, "$count > 10".to_string());

        assert!(bp.is_conditional(), "Should be a conditional breakpoint");
        assert!(
            bp.should_break(&vars),
            "Should break when count (15) > 10"
        );
    }

    /// Test: REPL-007-002-002 - Conditional breakpoint evaluates to false
    #[test]
    fn test_REPL_007_002_conditional_false() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "5".to_string());

        // Create conditional breakpoint: break if $count > 10
        let bp = Breakpoint::with_condition(5, "$count > 10".to_string());

        assert!(!bp.should_break(&vars), "Should not break when count (5) <= 10");
    }

    /// Test: REPL-007-002-003 - Conditional breakpoint with invalid syntax
    #[test]
    fn test_REPL_007_002_conditional_invalid() {
        let vars = HashMap::new();

        // Invalid condition (missing variable)
        let bp1 = Breakpoint::with_condition(5, "$missing > 10".to_string());
        assert!(
            !bp1.should_break(&vars),
            "Should not break with missing variable"
        );

        // Invalid condition (bad syntax)
        let bp2 = Breakpoint::with_condition(5, "invalid syntax".to_string());
        assert!(
            !bp2.should_break(&vars),
            "Should not break with invalid syntax"
        );
    }

    /// Test: REPL-007-002-004 - String equality condition
    #[test]
    fn test_REPL_007_002_string_equality() {
        let mut vars = HashMap::new();
        vars.insert("status".to_string(), "running".to_string());

        // Test == operator
        let bp_eq = Breakpoint::with_condition(5, "$status == running".to_string());
        assert!(bp_eq.should_break(&vars), "Should break when status == running");

        // Test != operator
        let bp_ne = Breakpoint::with_condition(5, "$status != stopped".to_string());
        assert!(
            bp_ne.should_break(&vars),
            "Should break when status != stopped"
        );

        // Test != operator (false case)
        let bp_ne_false = Breakpoint::with_condition(5, "$status != running".to_string());
        assert!(
            !bp_ne_false.should_break(&vars),
            "Should not break when status == running (but checking !=)"
        );
    }

    /// Test: REPL-007-002-005 - Less than comparison
    #[test]
    fn test_REPL_007_002_less_than() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "5".to_string());

        // Test < operator (true)
        let bp_lt = Breakpoint::with_condition(5, "$count < 10".to_string());
        assert!(bp_lt.should_break(&vars), "Should break when count (5) < 10");

        // Test < operator (false)
        let bp_lt_false = Breakpoint::with_condition(5, "$count < 3".to_string());
        assert!(
            !bp_lt_false.should_break(&vars),
            "Should not break when count (5) >= 3"
        );
    }

    /// Test: REPL-007-002-006 - Disabled conditional breakpoint
    #[test]
    fn test_REPL_007_002_disabled_conditional() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "15".to_string());

        // Create and disable conditional breakpoint
        let mut bp = Breakpoint::with_condition(5, "$count > 10".to_string());
        bp.disable();

        assert!(
            !bp.should_break(&vars),
            "Disabled breakpoint should not trigger even if condition is true"
        );
    }

    // ===== HIT-COUNT BREAKPOINTS (REPL-007-003) =====

    /// Test: REPL-007-003-001 - Hit-count breakpoint triggers after threshold
    #[test]
    fn test_REPL_007_003_hit_count_trigger() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 3); // Break after 3 hits

        // First two hits should not trigger
        assert!(!bp.should_break_with_hit(&vars), "Should not break on hit 1");
        assert!(!bp.should_break_with_hit(&vars), "Should not break on hit 2");

        // Third hit should trigger
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 3");

        // Subsequent hits should also trigger
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 4");
    }

    /// Test: REPL-007-003-002 - Hit-count breakpoint does not trigger before threshold
    #[test]
    fn test_REPL_007_003_hit_count_not_reached() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 5); // Break after 5 hits

        // First 4 hits should not trigger
        for i in 1..=4 {
            assert!(
                !bp.should_break_with_hit(&vars),
                "Should not break on hit {}",
                i
            );
        }

        // Fifth hit should trigger
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 5");
    }

    /// Test: REPL-007-003-003 - Hit-count resets correctly
    #[test]
    fn test_REPL_007_003_hit_count_reset() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 2); // Break after 2 hits

        // Hit twice
        assert!(!bp.should_break_with_hit(&vars), "Should not break on hit 1");
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 2");

        // Reset hit count
        bp.reset_hit_count();

        // Should not trigger on first hit after reset
        assert!(!bp.should_break_with_hit(&vars), "Should not break after reset");
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 2 after reset");
    }

    /// Test: REPL-007-003-004 - Hit-count with condition (both must be true)
    #[test]
    fn test_REPL_007_003_hit_count_with_condition() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "15".to_string());

        let mut bp = Breakpoint::with_hit_count_and_condition(5, 2, "$count > 10".to_string());

        // First hit: condition true, but hit count not reached
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break: condition true but hit count = 1"
        );

        // Second hit: condition true and hit count reached
        assert!(
            bp.should_break_with_hit(&vars),
            "Should break: condition true and hit count = 2"
        );
    }

    /// Test: REPL-007-003-005 - Get current hit count
    #[test]
    fn test_REPL_007_003_get_hit_count() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 3);

        assert_eq!(bp.get_hit_count(), 0, "Initial hit count should be 0");

        bp.should_break_with_hit(&vars);
        assert_eq!(bp.get_hit_count(), 1, "Hit count should be 1 after first hit");

        bp.should_break_with_hit(&vars);
        assert_eq!(bp.get_hit_count(), 2, "Hit count should be 2 after second hit");

        bp.should_break_with_hit(&vars);
        assert_eq!(bp.get_hit_count(), 3, "Hit count should be 3 after third hit");
    }

    /// Test: REPL-007-003-006 - Disabled hit-count breakpoint never triggers
    #[test]
    fn test_REPL_007_003_disabled_hit_count() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 2);
        bp.disable();

        // Even after reaching threshold, disabled breakpoint should not trigger
        assert!(!bp.should_break_with_hit(&vars), "Should not break when disabled");
        assert!(!bp.should_break_with_hit(&vars), "Should not break when disabled");
        assert!(!bp.should_break_with_hit(&vars), "Should not break when disabled");
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

    /// Property: Conditional breakpoints with > operator
    proptest! {
        #[test]
        fn prop_conditional_greater_than(value in 0i64..1000, threshold in 0i64..1000) {
            let mut vars = HashMap::new();
            vars.insert("val".to_string(), value.to_string());

            let bp = Breakpoint::with_condition(1, format!("$val > {}", threshold));

            let should_break = bp.should_break(&vars);
            prop_assert_eq!(
                should_break,
                value > threshold,
                "Conditional > should match comparison: {} > {} = {}",
                value,
                threshold,
                value > threshold
            );
        }
    }

    /// Property: Conditional breakpoints are deterministic
    proptest! {
        #[test]
        fn prop_conditional_deterministic(value in 0i64..100, threshold in 0i64..100) {
            let mut vars = HashMap::new();
            vars.insert("val".to_string(), value.to_string());

            let bp = Breakpoint::with_condition(1, format!("$val > {}", threshold));

            // Check multiple times - should always return same result
            let result1 = bp.should_break(&vars);
            let result2 = bp.should_break(&vars);
            let result3 = bp.should_break(&vars);

            prop_assert_eq!(result1, result2, "should_break should be deterministic");
            prop_assert_eq!(result2, result3, "should_break should be deterministic");
        }
    }

    /// Property: String equality conditions
    proptest! {
        #[test]
        fn prop_conditional_string_equality(value in "[a-z]{1,10}", compare in "[a-z]{1,10}") {
            let mut vars = HashMap::new();
            vars.insert("val".to_string(), value.clone());

            // Test == operator
            let bp_eq = Breakpoint::with_condition(1, format!("$val == {}", compare));
            prop_assert_eq!(
                bp_eq.should_break(&vars),
                value == compare,
                "== operator should match string equality"
            );

            // Test != operator
            let bp_ne = Breakpoint::with_condition(1, format!("$val != {}", compare));
            prop_assert_eq!(
                bp_ne.should_break(&vars),
                value != compare,
                "!= operator should match string inequality"
            );
        }
    }

    /// Property: Disabled conditional breakpoints never trigger
    proptest! {
        #[test]
        fn prop_disabled_conditional_never_breaks(value in 0i64..100) {
            let mut vars = HashMap::new();
            vars.insert("val".to_string(), value.to_string());

            let mut bp = Breakpoint::with_condition(1, "$val > 0".to_string());
            bp.disable();

            prop_assert!(
                !bp.should_break(&vars),
                "Disabled breakpoint should never trigger"
            );
        }
    }

    /// Property: Missing variables cause condition to fail
    proptest! {
        #[test]
        fn prop_missing_variable_fails(threshold in 0i64..100) {
            let vars = HashMap::new(); // Empty - no variables

            let bp = Breakpoint::with_condition(1, format!("$missing > {}", threshold));

            prop_assert!(
                !bp.should_break(&vars),
                "Condition with missing variable should not trigger"
            );
        }
    }

    // ===== HIT-COUNT PROPERTY TESTS (REPL-007-003) =====

    /// Property: Hit-count breakpoints trigger at the correct threshold
    proptest! {
        #[test]
        fn prop_hit_count_triggers_at_threshold(threshold in 1usize..20) {
            let vars = HashMap::new();
            let mut bp = Breakpoint::with_hit_count(1, threshold);

            // Hit the breakpoint threshold-1 times (should not trigger)
            for _ in 0..threshold-1 {
                prop_assert!(
                    !bp.should_break_with_hit(&vars),
                    "Should not trigger before threshold"
                );
            }

            // Next hit should trigger
            prop_assert!(
                bp.should_break_with_hit(&vars),
                "Should trigger at threshold {}",
                threshold
            );
        }
    }

    /// Property: Hit count is always accurate
    proptest! {
        #[test]
        fn prop_hit_count_accurate(hits in 1usize..50) {
            let vars = HashMap::new();
            let mut bp = Breakpoint::with_hit_count(1, 999); // High threshold

            for i in 1..=hits {
                bp.should_break_with_hit(&vars);
                prop_assert_eq!(
                    bp.get_hit_count(),
                    i,
                    "Hit count should be accurate at {} hits",
                    i
                );
            }
        }
    }

    /// Property: Reset always sets hit count to zero
    proptest! {
        #[test]
        fn prop_reset_hit_count_works(hits in 1usize..50) {
            let vars = HashMap::new();
            let mut bp = Breakpoint::with_hit_count(1, 999);

            // Hit the breakpoint multiple times
            for _ in 0..hits {
                bp.should_break_with_hit(&vars);
            }

            // Reset
            bp.reset_hit_count();

            prop_assert_eq!(
                bp.get_hit_count(),
                0,
                "Hit count should be 0 after reset"
            );
        }
    }

    /// Property: Disabled hit-count breakpoints never trigger
    proptest! {
        #[test]
        fn prop_disabled_hit_count_never_triggers(threshold in 1usize..20, hits in 1usize..50) {
            let vars = HashMap::new();
            let mut bp = Breakpoint::with_hit_count(1, threshold);
            bp.disable();

            // Hit the breakpoint many times
            for _ in 0..hits {
                prop_assert!(
                    !bp.should_break_with_hit(&vars),
                    "Disabled breakpoint should never trigger"
                );
            }
        }
    }

    /// Property: Hit-count with condition requires both to be true
    proptest! {
        #[test]
        fn prop_hit_count_with_condition_both_required(threshold in 2usize..10, value in 0i64..100) {
            let mut vars = HashMap::new();
            vars.insert("val".to_string(), value.to_string());

            let mut bp = Breakpoint::with_hit_count_and_condition(1, threshold, "$val > 50".to_string());

            // Hit threshold-1 times
            for _ in 0..threshold-1 {
                let result = bp.should_break_with_hit(&vars);
                prop_assert!(!result, "Should not trigger before threshold");
            }

            // At threshold, should match condition
            let result = bp.should_break_with_hit(&vars);
            let expected = value > 50;
            prop_assert_eq!(
                result,
                expected,
                "At threshold {}, should match condition: {} > 50 = {}",
                threshold,
                value,
                expected
            );
        }
    }
}
