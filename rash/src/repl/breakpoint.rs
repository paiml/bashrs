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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "breakpoint_tests_repl_007.rs"]
// FIXME(PMAT-238): mod tests_extracted;
