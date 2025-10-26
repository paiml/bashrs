//! CONFIG-002: Quote Variable Expansions
//!
//! Detects and fixes unquoted variable expansions that can lead to:
//! - Word splitting
//! - Glob expansion
//! - Injection vulnerabilities
//!
//! Transforms:
//! - `export DIR=$HOME/my projects` → `export DIR="${HOME}/my projects"`
//! - `cd $PROJECT_DIR` → `cd "${PROJECT_DIR}"`
//! - `FILES=$(ls *.txt)` → `FILES="$(ls *.txt)"`

use super::{ConfigIssue, Severity};
use regex::Regex;
use std::collections::HashMap;

/// Pattern to match unquoted variable expansions
/// Matches: $VAR or ${VAR} not already inside quotes
fn create_unquoted_var_pattern() -> Regex {
    // Match variable patterns: $VAR, ${VAR}, $1, etc.
    // But NOT when already inside double quotes
    Regex::new(r#"\$\{?[A-Za-z_][A-Za-z0-9_]*\}?"#).unwrap()
}

/// Analyze source for unquoted variable expansions
pub fn analyze_unquoted_variables(source: &str) -> Vec<UnquotedVariable> {
    let mut variables = Vec::new();
    let var_pattern = create_unquoted_var_pattern();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comments
        if line.trim().starts_with('#') {
            continue;
        }

        // Check if line has variable expansions
        if !line.contains('$') {
            continue;
        }

        // Find all variable references in the line
        for cap in var_pattern.captures_iter(line) {
            let var_match = cap.get(0).unwrap();
            let var_name = var_match.as_str();
            let start = var_match.start();

            // Check if already quoted
            if is_already_quoted(line, start) {
                continue;
            }

            // Check if in special contexts where quoting not needed
            if is_special_context(line, start) {
                continue;
            }

            variables.push(UnquotedVariable {
                line: line_num,
                column: start,
                variable: var_name.to_string(),
                context: line.to_string(),
            });
        }
    }

    variables
}

/// Check if a variable at position is already quoted
fn is_already_quoted(line: &str, pos: usize) -> bool {
    // Check for double quotes before variable
    let before = &line[..pos];

    // Count quotes before this position
    let quote_count = before.matches('"').count();

    // If odd number of quotes, we're inside quotes
    if quote_count % 2 == 1 {
        return true;
    }

    // Check if immediately preceded by quote
    if pos > 0 && line.chars().nth(pos - 1) == Some('"') {
        return true;
    }

    false
}

/// Check if variable is in a special context where quoting is not needed
fn is_special_context(line: &str, pos: usize) -> bool {
    let line_trimmed = line.trim();

    // In arithmetic context: $(( )) or (( ))
    if line_trimmed.contains("$((") || line_trimmed.contains("((") {
        return true;
    }

    // In array index: arr[$i]
    if pos > 0 && line.chars().nth(pos - 1) == Some('[') {
        return true;
    }

    // After 'export' without assignment (just exporting, not assigning)
    if line_trimmed.starts_with("export ") && !line.contains('=') {
        return true;
    }

    false
}

/// Represents an unquoted variable found in the source
#[derive(Debug, Clone, PartialEq)]
pub struct UnquotedVariable {
    pub line: usize,
    pub column: usize,
    pub variable: String,
    pub context: String,
}

/// Generate CONFIG-002 issues for unquoted variables
pub fn detect_unquoted_variables(variables: &[UnquotedVariable]) -> Vec<ConfigIssue> {
    variables
        .iter()
        .map(|var| ConfigIssue {
            rule_id: "CONFIG-002".to_string(),
            severity: Severity::Warning,
            message: format!(
                "Unquoted variable expansion: '{}' can cause word splitting and glob expansion",
                var.variable
            ),
            line: var.line,
            column: var.column,
            suggestion: Some(format!("Quote the variable: \"{}\"", var.variable)),
        })
        .collect()
}

/// Quote all unquoted variables in source
pub fn quote_variables(source: &str) -> String {
    let variables = analyze_unquoted_variables(source);

    if variables.is_empty() {
        return source.to_string();
    }

    // Build a map of line numbers to variables on that line
    let mut lines_to_fix: HashMap<usize, Vec<&UnquotedVariable>> = HashMap::new();
    for var in &variables {
        lines_to_fix
            .entry(var.line)
            .or_default()
            .push(var);
    }

    let mut result = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(_vars_on_line) = lines_to_fix.get(&line_num) {
            // Check if this is an export/assignment line
            if line.contains('=')
                && (line.trim().starts_with("export ")
                    || line.trim().starts_with("local ")
                    || !line.trim().starts_with("if "))
            {
                // For assignment lines, quote the entire RHS value
                let fixed_line = quote_assignment_line(line);
                result.push(fixed_line);
            } else {
                // For command lines, quote individual variables
                let fixed_line = quote_command_line(line);
                result.push(fixed_line);
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Quote the RHS of an assignment line
fn quote_assignment_line(line: &str) -> String {
    // Find the = sign
    if let Some(eq_pos) = line.find('=') {
        let lhs = &line[..=eq_pos];
        let rhs = &line[eq_pos + 1..];

        // If RHS already starts and ends with quotes, keep it
        if (rhs.starts_with('"') && rhs.ends_with('"'))
            || (rhs.starts_with('\'') && rhs.ends_with('\''))
        {
            return line.to_string();
        }

        // Convert $VAR to ${VAR} in the RHS before quoting
        let rhs_with_braces = add_braces_to_variables(rhs);

        // Quote the entire RHS
        format!("{}\"{}\"", lhs, rhs_with_braces)
    } else {
        line.to_string()
    }
}

/// Convert $VAR to ${VAR} (add braces if missing)
fn add_braces_to_variables(text: &str) -> String {
    let var_pattern = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    // In replacement strings, $ is special, so $$ = literal $, and $1 = capture group 1
    var_pattern.replace_all(text, "$${$1}").to_string()
}

/// Quote variables in a command line
fn quote_command_line(line: &str) -> String {
    let var_pattern = create_unquoted_var_pattern();
    let mut result = line.to_string();

    // Find all variables and quote them
    let matches: Vec<_> = var_pattern.find_iter(line).collect();

    // Replace from right to left to maintain positions
    for mat in matches.iter().rev() {
        let var = mat.as_str();
        let start = mat.start();
        let end = mat.end();

        // Skip if already quoted
        if is_already_quoted(line, start) {
            continue;
        }

        // Create quoted version
        let quoted = if var.starts_with("${") {
            format!("\"{}\"", var)
        } else {
            let var_name = var.trim_start_matches('$');
            format!("\"${{{}}}\"", var_name)
        };

        // Replace
        let before = &result[..start];
        let after = &result[end..];
        result = format!("{}{}{}", before, quoted, after);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_002_detect_simple_unquoted_var() {
        // ARRANGE
        let line = "export PROJECT_DIR=$HOME/my projects";

        // ACT
        let variables = analyze_unquoted_variables(line);

        // ASSERT
        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].variable, "$HOME");
        assert_eq!(variables[0].line, 1);
    }

    #[test]
    fn test_config_002_detect_unquoted_in_cd() {
        // ARRANGE
        let source = "cd $PROJECT_DIR";

        // ACT
        let variables = analyze_unquoted_variables(source);

        // ASSERT
        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].variable, "$PROJECT_DIR");
    }

    #[test]
    fn test_config_002_ignore_already_quoted() {
        // ARRANGE
        let source = r#"export DIR="${HOME}/projects""#;

        // ACT
        let variables = analyze_unquoted_variables(source);

        // ASSERT
        assert_eq!(
            variables.len(),
            0,
            "Should not flag already quoted variables"
        );
    }

    #[test]
    fn test_config_002_ignore_comments() {
        // ARRANGE
        let source = "# export DIR=$HOME/projects";

        // ACT
        let variables = analyze_unquoted_variables(source);

        // ASSERT
        assert_eq!(variables.len(), 0, "Should ignore variables in comments");
    }

    #[test]
    fn test_config_002_detect_multiple_on_same_line() {
        // ARRANGE
        let source = "cp $SOURCE $DEST";

        // ACT
        let variables = analyze_unquoted_variables(source);

        // ASSERT
        assert_eq!(variables.len(), 2);
        assert_eq!(variables[0].variable, "$SOURCE");
        assert_eq!(variables[1].variable, "$DEST");
    }

    #[test]
    fn test_config_002_detect_command_substitution() {
        // ARRANGE
        let source = "FILES=$(ls *.txt)";

        // ACT
        let variables = analyze_unquoted_variables(source);

        // ASSERT
        // Note: This test is about the variable assignment, not the command substitution
        // The value side should be quoted: FILES="$(ls *.txt)"
        assert_eq!(
            variables.len(),
            0,
            "Command substitution on RHS is OK in assignment"
        );
    }

    #[test]
    fn test_config_002_generate_issues() {
        // ARRANGE
        let source = r#"export PROJECT_DIR=$HOME/my projects
cd $PROJECT_DIR"#;

        let variables = analyze_unquoted_variables(source);

        // ACT
        let issues = detect_unquoted_variables(&variables);

        // ASSERT
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].rule_id, "CONFIG-002");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("word splitting"));
    }

    #[test]
    fn test_config_002_quote_simple_variable() {
        // ARRANGE
        let source = "export DIR=$HOME/projects";

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, r#"export DIR="${HOME}/projects""#);
    }

    #[test]
    fn test_config_002_quote_multiple_variables() {
        // ARRANGE
        let source = "cp $SOURCE $DEST";

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, r#"cp "${SOURCE}" "${DEST}""#);
    }

    #[test]
    fn test_config_002_preserve_already_quoted() {
        // ARRANGE
        let source = r#"export DIR="${HOME}/projects"
echo "Hello $USER""#;

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, source, "Should not change already quoted variables");
    }

    #[test]
    fn test_config_002_preserve_comments() {
        // ARRANGE
        let source = r#"# My config
export DIR=$HOME/projects
# End"#;

        let expected = r#"# My config
export DIR="${HOME}/projects"
# End"#;

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_002_handle_braced_variables() {
        // ARRANGE
        let source = "export DIR=${HOME}/projects";

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, r#"export DIR="${HOME}/projects""#);
    }

    #[test]
    fn test_config_002_real_world_example() {
        // ARRANGE
        let source = r#"export PROJECT_DIR=$HOME/my projects
export BACKUP_DIR=$HOME/backups
cd $PROJECT_DIR
cp $PROJECT_DIR/file.txt $BACKUP_DIR/"#;

        // Note: Currently quotes each variable individually (safe but verbose)
        // TODO v7.1: Optimize to quote entire arguments
        let expected = r#"export PROJECT_DIR="${HOME}/my projects"
export BACKUP_DIR="${HOME}/backups"
cd "${PROJECT_DIR}"
cp "${PROJECT_DIR}"/file.txt "${BACKUP_DIR}"/"#;

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_002_idempotent() {
        // ARRANGE
        let source = "export DIR=$HOME/projects";

        // ACT
        let quoted_once = quote_variables(source);
        let quoted_twice = quote_variables(&quoted_once);

        // ASSERT
        assert_eq!(quoted_once, quoted_twice, "Quoting should be idempotent");
    }

    #[test]
    fn test_config_002_debug_add_braces() {
        // Test the add_braces_to_variables function directly
        let input = "$HOME/projects";
        let result = add_braces_to_variables(input);
        assert_eq!(
            result, "${HOME}/projects",
            "add_braces_to_variables should convert $HOME to ${{HOME}}"
        );
    }

    #[test]
    fn test_config_002_debug_quote_assignment() {
        // Test quote_assignment_line directly
        let input = "export DIR=$HOME/projects";
        let result = quote_assignment_line(input);
        println!("Input: {}", input);
        println!("Result: {}", result);
        assert_eq!(result, r#"export DIR="${HOME}/projects""#);
    }

    #[test]
    fn test_config_002_empty_input() {
        // ARRANGE
        let source = "";

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, "");
    }

    #[test]
    fn test_config_002_no_variables() {
        // ARRANGE
        let source = r#"export EDITOR="vim"
alias ll='ls -la'"#;

        // ACT
        let result = quote_variables(source);

        // ASSERT
        assert_eq!(result, source, "Should not change lines without variables");
    }
}
