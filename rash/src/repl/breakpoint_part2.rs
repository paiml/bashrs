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
/// Parse operator from condition string and split into parts
fn parse_condition_operator(condition: &str) -> Option<(&'static str, &str, &str)> {
    // Check for two-character operators first (==, !=, >=, <=)
    if let Some(pos) = condition.find("==") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        return Some(("==", var_part, value_part));
    }
    if let Some(pos) = condition.find("!=") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        return Some(("!=", var_part, value_part));
    }
    if let Some(pos) = condition.find(">=") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        return Some((">=", var_part, value_part));
    }
    if let Some(pos) = condition.find("<=") {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 2..].trim();
        return Some(("<=", var_part, value_part));
    }
    // Single-character operators
    if let Some(pos) = condition.find('>') {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 1..].trim();
        return Some((">", var_part, value_part));
    }
    if let Some(pos) = condition.find('<') {
        let var_part = condition[..pos].trim();
        let value_part = condition[pos + 1..].trim();
        return Some(("<", var_part, value_part));
    }
    None
}

/// Extract variable name from variable part (remove $)
fn extract_variable_name(var_part: &str) -> Option<&str> {
    var_part.strip_prefix('$')
}

/// Try numeric comparison between two strings
fn try_numeric_comparison<F>(var_value: &str, value_part: &str, compare: F) -> bool
where
    F: Fn(i64, i64) -> bool,
{
    if let (Ok(var_num), Ok(val_num)) = (var_value.parse::<i64>(), value_part.parse::<i64>()) {
        compare(var_num, val_num)
    } else {
        false
    }
}

/// Perform comparison based on operator
fn perform_comparison(operator: &str, var_value: &str, value_part: &str) -> bool {
    match operator {
        "==" => var_value == value_part,
        "!=" => var_value != value_part,
        ">" => try_numeric_comparison(var_value, value_part, |a, b| a > b),
        "<" => try_numeric_comparison(var_value, value_part, |a, b| a < b),
        ">=" => try_numeric_comparison(var_value, value_part, |a, b| a >= b),
        "<=" => try_numeric_comparison(var_value, value_part, |a, b| a <= b),
        _ => false,
    }
}

fn evaluate_condition(condition: &str, variables: &HashMap<String, String>) -> bool {
    let condition = condition.trim();

    // Parse operator and split condition
    let (operator, var_part, value_part) = match parse_condition_operator(condition) {
        Some(parts) => parts,
        None => return false, // No operator found
    };

    // Extract variable name (remove $)
    let var_name = match extract_variable_name(var_part) {
        Some(name) => name,
        None => return false, // Invalid syntax
    };

    // Get variable value
    let var_value = match variables.get(var_name) {
        Some(v) => v,
        None => return false, // Variable not found
    };

    // Perform comparison
    perform_comparison(operator, var_value, value_part)
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
        assert!(
            removed,
            "Should return true when removing existing breakpoint"
        );

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
        assert_eq!(
            breakpoints,
            vec![1, 5, 10, 15],
            "Breakpoints should be sorted"
        );
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
        assert!(bp.should_break(&vars), "Should break when count (15) > 10");
    }


    include!("breakpoint_part2_incl2.rs");
