// REPL Variable Assignment and Expansion Module
//
// Task: REPL-007-001 - Variable assignment and expansion
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 15+ scenarios
// - Integration tests: Variable workflows with assert_cmd
// - Mutation score: ≥90%
// - Complexity: <10 per function

use regex::Regex;
use std::collections::HashMap;

/// Parse a variable assignment from a line
///
/// Recognizes patterns like:
/// - `x=5`
/// - `name="hello world"`
/// - `PATH=/usr/bin`
///
/// Returns Some((name, value)) if the line is a valid assignment,
/// None otherwise.
///
/// # Examples
///
/// ```
/// use bashrs::repl::variables::parse_assignment;
///
/// assert_eq!(parse_assignment("x=5"), Some(("x".to_string(), "5".to_string())));
/// assert_eq!(parse_assignment("name=\"test\""), Some(("name".to_string(), "test".to_string())));
/// assert_eq!(parse_assignment("echo hello"), None);
/// ```
pub fn parse_assignment(line: &str) -> Option<(String, String)> {
    let line = line.trim();

    // Match pattern: VARIABLE_NAME=VALUE
    // Variable names: [A-Za-z_][A-Za-z0-9_]*
    let re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$").ok()?;

    let captures = re.captures(line)?;

    let name = captures.get(1)?.as_str().to_string();
    let value = captures.get(2)?.as_str();

    // Handle quoted values
    let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        // Remove quotes
        value[1..value.len() - 1].to_string()
    } else if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        // Remove single quotes
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    };

    Some((name, value))
}

/// Expand variables in a command string
///
/// Supports:
/// - Simple variables: `$var`
/// - Braced variables: `${var}`
/// - Unknown variables expand to empty string
///
/// # Examples
///
/// ```
/// use bashrs::repl::variables::expand_variables;
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("x".to_string(), "42".to_string());
/// vars.insert("name".to_string(), "Alice".to_string());
///
/// assert_eq!(expand_variables("echo $x", &vars), "echo 42");
/// assert_eq!(expand_variables("hello ${name}", &vars), "hello Alice");
/// assert_eq!(expand_variables("$unknown is empty", &vars), " is empty");
/// ```
pub fn expand_variables(command: &str, variables: &HashMap<String, String>) -> String {
    let mut result = command.to_string();

    // First, expand braced variables ${var}
    #[allow(clippy::expect_used)] // Safe: hardcoded regex pattern is valid
    let braced_re =
        Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").expect("hardcoded regex is valid");
    result = braced_re
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            variables.get(var_name).cloned().unwrap_or_default()
        })
        .to_string();

    // Then, expand simple variables $var
    #[allow(clippy::expect_used)] // Safe: hardcoded regex pattern is valid
    let simple_re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").expect("hardcoded regex is valid");
    result = simple_re
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            variables.get(var_name).cloned().unwrap_or_default()
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== RED PHASE: Unit Tests (These should FAIL initially) =====

    #[test]
    fn test_REPL_007_001_parse_assignment_simple() {
        let result = parse_assignment("x=5");

        assert_eq!(result, Some(("x".to_string(), "5".to_string())));
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_double_quotes() {
        let result = parse_assignment("name=\"hello world\"");

        assert_eq!(
            result,
            Some(("name".to_string(), "hello world".to_string()))
        );
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_single_quotes() {
        let result = parse_assignment("path='/usr/bin'");

        assert_eq!(result, Some(("path".to_string(), "/usr/bin".to_string())));
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_no_quotes() {
        let result = parse_assignment("USER=alice");

        assert_eq!(result, Some(("USER".to_string(), "alice".to_string())));
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_empty_value() {
        let result = parse_assignment("empty=");

        assert_eq!(result, Some(("empty".to_string(), String::new())));
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_underscore_name() {
        let result = parse_assignment("_private=secret");

        assert_eq!(result, Some(("_private".to_string(), "secret".to_string())));
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_number_in_value() {
        let result = parse_assignment("count=42");

        assert_eq!(result, Some(("count".to_string(), "42".to_string())));
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_not_an_assignment() {
        let result = parse_assignment("echo hello");

        assert_eq!(result, None);
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_command_with_equals() {
        let result = parse_assignment("test -f file=test.txt");

        // This should NOT be parsed as an assignment because "test" is not a valid var name pattern
        assert_eq!(result, None);
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_invalid_var_name() {
        let result = parse_assignment("123invalid=value");

        assert_eq!(result, None);
    }

    #[test]
    fn test_REPL_007_001_expand_simple_variable() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "42".to_string());

        let result = expand_variables("echo $x", &vars);

        assert_eq!(result, "echo 42");
    }

    #[test]
    fn test_REPL_007_001_expand_braced_variable() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());

        let result = expand_variables("hello ${name}", &vars);

        assert_eq!(result, "hello Alice");
    }

    #[test]
    fn test_REPL_007_001_expand_multiple_variables() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "1".to_string());
        vars.insert("y".to_string(), "2".to_string());

        let result = expand_variables("$x + $y = 3", &vars);

        assert_eq!(result, "1 + 2 = 3");
    }

    #[test]
    fn test_REPL_007_001_expand_unknown_variable() {
        let vars = HashMap::new();

        let result = expand_variables("echo $unknown", &vars);

        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_REPL_007_001_expand_mixed_variables() {
        let mut vars = HashMap::new();
        vars.insert("simple".to_string(), "S".to_string());
        vars.insert("braced".to_string(), "B".to_string());

        let result = expand_variables("$simple and ${braced}", &vars);

        assert_eq!(result, "S and B");
    }

    #[test]
    fn test_REPL_007_001_expand_no_variables() {
        let vars = HashMap::new();

        let result = expand_variables("echo hello world", &vars);

        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_REPL_007_001_parse_assignment_with_spaces() {
        let result = parse_assignment("  var = value  ");

        // Trim should handle leading/trailing spaces, but spaces around = are not valid
        assert_eq!(result, None);
    }

    #[test]
    fn test_REPL_007_001_expand_variable_at_end() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "end".to_string());

        let result = expand_variables("value: $x", &vars);

        assert_eq!(result, "value: end");
    }

    // ===== PROPERTY TESTING PHASE =====
    //
    // Property-based tests verify that our functions work correctly
    // for 100+ generated test cases

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Generate valid bash variable names
        // Pattern: [A-Za-z_][A-Za-z0-9_]*
        fn valid_var_name() -> impl Strategy<Value = String> {
            "[A-Za-z_][A-Za-z0-9_]{0,19}"
        }

        // Property: Valid variable names always parse successfully
        proptest! {
            #[test]
            fn prop_REPL_007_001_valid_names_parse(
                name in valid_var_name(),
                value in "[a-zA-Z0-9_./:-][a-zA-Z0-9_./:-]{0,49}"  // No spaces to avoid trim issues
            ) {
                let assignment = format!("{}={}", name, value);
                let result = parse_assignment(&assignment);

                prop_assert!(result.is_some(), "Valid assignment should parse: {}", assignment);
                let (parsed_name, parsed_value) = result.unwrap();
                prop_assert_eq!(parsed_name, name);
                prop_assert_eq!(parsed_value, value);
            }
        }

        // Property: Expansion is deterministic - same input always produces same output
        proptest! {
            #[test]
            fn prop_REPL_007_001_expansion_deterministic(
                command in "echo \\$[a-z]{1,10}",
                var_name in "[a-z]{1,10}",
                var_value in ".*{0,20}"
            ) {
                let mut vars = HashMap::new();
                vars.insert(var_name, var_value);

                let result1 = expand_variables(&command, &vars);
                let result2 = expand_variables(&command, &vars);

                prop_assert_eq!(result1, result2, "Expansion must be deterministic");
            }
        }

        // Property: Unknown variables always expand to empty string
        proptest! {
            #[test]
            fn prop_REPL_007_001_unknown_vars_empty(
                command in "echo \\$[a-z]{1,10}"
            ) {
                let vars = HashMap::new(); // Empty - all variables unknown

                let result = expand_variables(&command, &vars);

                // Should contain "echo " but the variable should be gone
                prop_assert!(result.starts_with("echo "), "Should preserve command");
                prop_assert!(!result.contains('$'), "Variables should be expanded (removed)");
            }
        }

        // Property: Assignment + expansion roundtrip preserves values
        proptest! {
            #[test]
            fn prop_REPL_007_001_roundtrip(
                name in valid_var_name(),
                value in "[a-zA-Z0-9 ]{1,30}"
            ) {
                // Assign
                let assignment = format!("{}={}", name, value);
                let parsed = parse_assignment(&assignment);
                prop_assert!(parsed.is_some());

                let (parsed_name, parsed_value) = parsed.unwrap();

                // Store in variables map
                let mut vars = HashMap::new();
                vars.insert(parsed_name.clone(), parsed_value.clone());

                // Expand - both simple and braced syntax
                let simple_expansion = format!("${}", parsed_name);
                let simple_result = expand_variables(&simple_expansion, &vars);

                let braced_expansion = format!("${{{}}}", parsed_name);
                let braced_result = expand_variables(&braced_expansion, &vars);

                // Verify roundtrip
                prop_assert_eq!(simple_result, parsed_value.clone(), "Simple expansion roundtrip failed");
                prop_assert_eq!(braced_result, parsed_value, "Braced expansion roundtrip failed");
            }
        }

        // Property: Multiple variables expand independently
        proptest! {
            #[test]
            fn prop_REPL_007_001_multiple_vars_independent(
                name1 in valid_var_name(),
                value1 in "[a-z]{1,10}",
                name2 in valid_var_name(),
                value2 in "[a-z]{1,10}"
            ) {
                // Skip if names are the same
                prop_assume!(name1 != name2);

                let mut vars = HashMap::new();
                vars.insert(name1.clone(), value1.clone());
                vars.insert(name2.clone(), value2.clone());

                let command = format!("${} and ${}", name1, name2);
                let result = expand_variables(&command, &vars);

                prop_assert!(result.contains(&value1), "Should contain first value");
                prop_assert!(result.contains(&value2), "Should contain second value");
                prop_assert_eq!(result, format!("{} and {}", value1, value2));
            }
        }

        // Property: Quoted values have quotes removed
        proptest! {
            #[test]
            fn prop_REPL_007_001_quotes_removed(
                name in valid_var_name(),
                value in "[a-zA-Z0-9 ]{1,20}"
            ) {
                // Test double quotes
                let double_quoted = format!("{}=\"{}\"", name, &value);
                let result = parse_assignment(&double_quoted);
                prop_assert!(result.is_some());
                let (_, parsed_value) = result.unwrap();
                prop_assert_eq!(parsed_value, value.clone(), "Double quotes should be removed");

                // Test single quotes
                let single_quoted = format!("{}='{}'", name, &value);
                let result = parse_assignment(&single_quoted);
                prop_assert!(result.is_some());
                let (_, parsed_value) = result.unwrap();
                prop_assert_eq!(parsed_value, value, "Single quotes should be removed");
            }
        }

        // Property: Empty values are valid
        proptest! {
            #[test]
            fn prop_REPL_007_001_empty_values_valid(
                name in valid_var_name()
            ) {
                let assignment = format!("{}=", name);
                let result = parse_assignment(&assignment);

                prop_assert!(result.is_some());
                let (parsed_name, parsed_value) = result.unwrap();
                prop_assert_eq!(parsed_name, name);
                prop_assert_eq!(parsed_value, "");
            }
        }
    }
}
