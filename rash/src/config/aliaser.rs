//! CONFIG-003: Consolidate Duplicate Aliases
//!
//! Detects and consolidates duplicate alias definitions.
//! When the same alias is defined multiple times, the last definition wins
//! (matching shell behavior).
//!
//! Transforms:
//! - `alias ls='ls --color=auto'` followed by `alias ls='ls -G'` → keep only last one
//! - Multiple definitions → single definition (last one wins)

use super::{ConfigIssue, Severity};
use regex::Regex;
use std::collections::HashMap;

/// Represents an alias definition found in the source
#[derive(Debug, Clone, PartialEq)]
pub struct AliasDefinition {
    pub line: usize,
    pub column: usize,
    pub name: String,
    pub value: String,
    pub context: String,
}

/// Analyze source for alias definitions
pub fn analyze_aliases(source: &str) -> Vec<AliasDefinition> {
    let mut aliases = Vec::new();
    let alias_pattern = create_alias_pattern();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comments
        if line.trim().starts_with('#') {
            continue;
        }

        // Find alias definitions
        for cap in alias_pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let name = cap.get(1).unwrap().as_str();
            // Group 2 is the opening quote, group 3 is the value
            let value = cap.get(3).unwrap().as_str();

            aliases.push(AliasDefinition {
                line: line_num,
                column: full_match.start(),
                name: name.to_string(),
                value: value.to_string(),
                context: line.to_string(),
            });
        }
    }

    aliases
}

/// Create regex pattern for alias definitions
fn create_alias_pattern() -> Regex {
    // Matches: alias name='value' or alias name="value"
    // Note: Rust regex doesn't support backreferences, so we match the value liberally
    Regex::new(r#"alias\s+([A-Za-z_][A-Za-z0-9_]*)=(['"])([^'"]+)['"]"#).unwrap()
}

/// Detect duplicate alias definitions
pub fn detect_duplicate_aliases(aliases: &[AliasDefinition]) -> Vec<ConfigIssue> {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut issues = Vec::new();

    for alias in aliases {
        if let Some(&first_line) = seen.get(alias.name.as_str()) {
            // Duplicate found!
            issues.push(ConfigIssue {
                rule_id: "CONFIG-003".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "Duplicate alias definition: '{}' (first defined at line {})",
                    alias.name, first_line
                ),
                line: alias.line,
                column: alias.column,
                suggestion: Some(format!(
                    "Remove earlier definition or rename alias. Last definition wins in shell."
                )),
            });
        } else {
            seen.insert(&alias.name, alias.line);
        }
    }

    issues
}

/// Consolidate duplicate aliases, keeping only the last definition
pub fn consolidate_aliases(source: &str) -> String {
    let aliases = analyze_aliases(source);

    if aliases.is_empty() {
        return source.to_string();
    }

    // Build map of alias names to their last definition line
    let mut last_definition: HashMap<String, usize> = HashMap::new();
    for alias in &aliases {
        last_definition.insert(alias.name.clone(), alias.line);
    }

    // Build set of lines to skip (duplicates)
    let mut lines_to_skip = Vec::new();
    for alias in &aliases {
        if let Some(&last_line) = last_definition.get(&alias.name) {
            if alias.line != last_line {
                // This is not the last definition - skip it
                lines_to_skip.push(alias.line);
            }
        }
    }

    // Reconstruct source, skipping duplicate lines
    let mut result = Vec::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if lines_to_skip.contains(&line_num) {
            continue; // Skip this duplicate
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_config_003_detect_simple_duplicate() {
        // ARRANGE
        let source = r#"alias ls='ls --color=auto'
alias ls='ls -G'"#;

        // ACT
        let aliases = analyze_aliases(source);

        // ASSERT
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name, "ls");
        assert_eq!(aliases[0].value, "ls --color=auto");
        assert_eq!(aliases[1].name, "ls");
        assert_eq!(aliases[1].value, "ls -G");
    }

    #[test]
    fn test_config_003_detect_multiple_duplicates() {
        // ARRANGE
        let source = r#"alias ll='ls -la'
alias ls='ls --color=auto'
alias ll='ls -alh'
alias grep='grep --color=auto'
alias ll='ls -lAh'"#;

        // ACT
        let aliases = analyze_aliases(source);

        // ASSERT
        assert_eq!(aliases.len(), 5);
        // ll defined 3 times
        let ll_count = aliases.iter().filter(|a| a.name == "ll").count();
        assert_eq!(ll_count, 3);
    }

    #[test]
    fn test_config_003_ignore_comments() {
        // ARRANGE
        let source = r#"alias ls='ls --color=auto'
# alias ls='ls -G'
alias grep='grep --color=auto'"#;

        // ACT
        let aliases = analyze_aliases(source);

        // ASSERT
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name, "ls");
        assert_eq!(aliases[1].name, "grep");
    }

    #[test]
    fn test_config_003_generate_issues() {
        // ARRANGE
        let source = r#"alias ls='ls --color=auto'
alias ls='ls -G'"#;

        let aliases = analyze_aliases(source);

        // ACT
        let issues = detect_duplicate_aliases(&aliases);

        // ASSERT
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "CONFIG-003");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].line, 2);
        assert!(issues[0].message.contains("Duplicate alias"));
        assert!(issues[0].message.contains("first defined at line 1"));
    }

    #[test]
    fn test_config_003_consolidate_simple() {
        // ARRANGE
        let source = r#"alias ls='ls --color=auto'
alias ls='ls -G'"#;

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert_eq!(result, "alias ls='ls -G'");
    }

    #[test]
    fn test_config_003_consolidate_multiple() {
        // ARRANGE
        let source = r#"alias ll='ls -la'
alias ls='ls --color=auto'
alias ll='ls -alh'
alias grep='grep --color=auto'
alias ll='ls -lAh'"#;

        let expected = r#"alias ls='ls --color=auto'
alias grep='grep --color=auto'
alias ll='ls -lAh'"#;

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_003_preserve_non_duplicates() {
        // ARRANGE
        let source = r#"alias ll='ls -la'
alias ls='ls --color=auto'
alias grep='grep --color=auto'
alias vi='vim'"#;

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert_eq!(result, source);
    }

    #[test]
    fn test_config_003_preserve_comments() {
        // ARRANGE
        let source = r#"# My aliases
alias ls='ls --color=auto'
alias ls='ls -G'
# End"#;

        let expected = r#"# My aliases
alias ls='ls -G'
# End"#;

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_config_003_idempotent() {
        // ARRANGE
        let source = r#"alias ls='ls --color=auto'
alias ls='ls -G'"#;

        // ACT
        let consolidated_once = consolidate_aliases(source);
        let consolidated_twice = consolidate_aliases(&consolidated_once);

        // ASSERT
        assert_eq!(
            consolidated_once, consolidated_twice,
            "Consolidation should be idempotent"
        );
    }

    #[test]
    fn test_config_003_empty_input() {
        // ARRANGE
        let source = "";

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert_eq!(result, "");
    }

    #[test]
    fn test_config_003_no_aliases() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
echo "Hello world""#;

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert_eq!(result, source);
    }

    #[test]
    fn test_config_003_double_quotes() {
        // ARRANGE
        let source = r#"alias ls="ls --color=auto"
alias ls="ls -G""#;

        // ACT
        let aliases = analyze_aliases(source);

        // ASSERT
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].value, "ls --color=auto");
        assert_eq!(aliases[1].value, "ls -G");
    }

    #[test]
    fn test_config_003_real_world_example() {
        // ARRANGE
        let source = r#"# Navigation
alias ..='cd ..'
alias ...='cd ../..'

# List aliases
alias ll='ls -la'
alias l='ls -CF'
alias ll='ls -alh'  # Override with better format

# Tools
alias grep='grep --color=auto'
alias ll='ls -lAh'  # Final override"#;

        // ACT
        let result = consolidate_aliases(source);

        // ASSERT
        assert!(!result.contains("alias ll='ls -la'"));
        assert!(!result.contains("alias ll='ls -alh'"));
        assert!(result.contains("alias ll='ls -lAh'"));
        assert!(result.contains("alias ..='cd ..'"));
        assert!(result.contains("alias grep='grep --color=auto'"));
    }
}
