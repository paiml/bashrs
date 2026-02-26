//! CONFIG-004: Detect and Remove Non-Deterministic Constructs
//!
//! Analyzes shell configuration files to detect non-deterministic constructs
//! that can cause unpredictable behavior across shell sessions.

use super::{ConfigIssue, Severity};
use regex::Regex;

// Static regex for date pattern matching
#[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
static DATE_PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\(date[^)]*\)").expect("valid regex pattern"));

/// Represents a non-deterministic construct found in config
#[derive(Debug, Clone, PartialEq)]
pub struct NonDeterministicConstruct {
    pub line: usize,
    pub column: usize,
    pub construct_type: ConstructType,
    pub context: String,
}

/// Types of non-deterministic constructs
#[derive(Debug, Clone, PartialEq)]
pub enum ConstructType {
    Random,    // $RANDOM
    Timestamp, // $(date +%s), $(date), etc.
    ProcessId, // $$
    Hostname,  // $(hostname)
    Uptime,    // $(uptime)
}

impl ConstructType {
    pub fn description(&self) -> &str {
        match self {
            ConstructType::Random => "$RANDOM generates unpredictable values",
            ConstructType::Timestamp => "Timestamp generation is non-deterministic",
            ConstructType::ProcessId => "$$ (process ID) changes between sessions",
            ConstructType::Hostname => "$(hostname) may vary across environments",
            ConstructType::Uptime => "$(uptime) changes constantly",
        }
    }

    pub fn suggestion(&self) -> &str {
        match self {
            ConstructType::Random => "Use a fixed seed or remove randomness from config",
            ConstructType::Timestamp => "Use a fixed version string instead of timestamps",
            ConstructType::ProcessId => "Use a fixed session ID or remove process ID",
            ConstructType::Hostname => "Use environment-specific config files instead",
            ConstructType::Uptime => "Remove uptime-based logic from config",
        }
    }
}

/// Analyze source for non-deterministic constructs
pub fn analyze_nondeterminism(source: &str) -> Vec<NonDeterministicConstruct> {
    let mut constructs = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip already-commented lines (for idempotency)
        if line.trim().starts_with('#') {
            continue;
        }

        // Detect $RANDOM
        if let Some(col) = line.find("$RANDOM") {
            constructs.push(NonDeterministicConstruct {
                line: line_num,
                column: col,
                construct_type: ConstructType::Random,
                context: line.trim().to_string(),
            });
        }

        // Detect $(date ...) patterns
        for mat in DATE_PATTERN.find_iter(line) {
            constructs.push(NonDeterministicConstruct {
                line: line_num,
                column: mat.start(),
                construct_type: ConstructType::Timestamp,
                context: line.trim().to_string(),
            });
        }

        // Detect $$ (process ID - always non-deterministic)
        if let Some(col) = line.find("$$") {
            constructs.push(NonDeterministicConstruct {
                line: line_num,
                column: col,
                construct_type: ConstructType::ProcessId,
                context: line.trim().to_string(),
            });
        }

        // Detect $(hostname)
        if line.contains("$(hostname)") {
            constructs.push(NonDeterministicConstruct {
                line: line_num,
                column: line.find("$(hostname)").unwrap(),
                construct_type: ConstructType::Hostname,
                context: line.trim().to_string(),
            });
        }

        // Detect $(uptime)
        if line.contains("$(uptime)") {
            constructs.push(NonDeterministicConstruct {
                line: line_num,
                column: line.find("$(uptime)").unwrap(),
                construct_type: ConstructType::Uptime,
                context: line.trim().to_string(),
            });
        }
    }

    constructs
}

/// Detect non-deterministic constructs and create CONFIG-004 issues
pub fn detect_nondeterminism(constructs: &[NonDeterministicConstruct]) -> Vec<ConfigIssue> {
    constructs
        .iter()
        .map(|construct| ConfigIssue {
            rule_id: "CONFIG-004".to_string(),
            severity: Severity::Warning,
            message: format!(
                "Non-deterministic construct: {}",
                construct.construct_type.description()
            ),
            line: construct.line,
            column: construct.column,
            suggestion: Some(construct.construct_type.suggestion().to_string()),
        })
        .collect()
}

/// Remove non-deterministic constructs from source
///
/// This is a conservative implementation that comments out problematic lines
/// rather than attempting automatic fixes (which could break functionality).
pub fn remove_nondeterminism(source: &str) -> String {
    let constructs = analyze_nondeterminism(source);

    if constructs.is_empty() {
        return source.to_string();
    }

    // Build set of lines to comment out
    let mut lines_to_comment: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for construct in &constructs {
        lines_to_comment.insert(construct.line);
    }

    // Process source, commenting out problematic lines
    let mut result = Vec::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if lines_to_comment.contains(&line_num) {
            // Comment out with explanation
            result.push("# RASH: Non-deterministic construct removed".to_string());
            result.push(format!("# {}", line));
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_004_detect_random() {
        // ARRANGE
        let source = r#"export SESSION_ID=$RANDOM"#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 1);
        assert_eq!(constructs[0].construct_type, ConstructType::Random);
        assert_eq!(constructs[0].line, 1);
    }

    #[test]
    fn test_config_004_detect_timestamp() {
        // ARRANGE
        let source = r#"export BUILD_TAG="build-$(date +%s)""#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 1);
        assert_eq!(constructs[0].construct_type, ConstructType::Timestamp);
        assert_eq!(constructs[0].line, 1);
    }

    #[test]
    fn test_config_004_detect_process_id() {
        // ARRANGE
        let source = r#"export TEMP_DIR="/tmp/work-$$""#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 1);
        assert_eq!(constructs[0].construct_type, ConstructType::ProcessId);
        assert_eq!(constructs[0].line, 1);
    }

    #[test]
    fn test_config_004_detect_hostname() {
        // ARRANGE
        let source = r#"export HOST=$(hostname)"#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 1);
        assert_eq!(constructs[0].construct_type, ConstructType::Hostname);
        assert_eq!(constructs[0].line, 1);
    }

    #[test]
    fn test_config_004_detect_multiple() {
        // ARRANGE
        let source = r#"export SESSION_ID=$RANDOM
export BUILD_TAG="build-$(date +%s)"
export TEMP_DIR="/tmp/work-$$""#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 3);
        assert_eq!(constructs[0].construct_type, ConstructType::Random);
        assert_eq!(constructs[1].construct_type, ConstructType::Timestamp);
        assert_eq!(constructs[2].construct_type, ConstructType::ProcessId);
    }

    #[test]
    fn test_config_004_create_issues() {
        // ARRANGE
        let source = r#"export SESSION_ID=$RANDOM"#;
        let constructs = analyze_nondeterminism(source);

        // ACT
        let issues = detect_nondeterminism(&constructs);

        // ASSERT
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "CONFIG-004");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("Non-deterministic"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_config_004_remove_nondeterminism() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
export SESSION_ID=$RANDOM
export EDITOR=vim"#;

        // ACT
        let result = remove_nondeterminism(source);

        // ASSERT
        assert!(result.contains("export PATH"));
        assert!(result.contains("export EDITOR"));
        assert!(result.contains("# RASH: Non-deterministic construct removed"));
        assert!(result.contains("# export SESSION_ID=$RANDOM"));
    }

    #[test]
    fn test_config_004_no_constructs() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
export EDITOR=vim"#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 0);
    }

    #[test]
    fn test_config_004_idempotent() {
        // ARRANGE
        let source = r#"export SESSION_ID=$RANDOM"#;

        // ACT
        let removed_once = remove_nondeterminism(source);
        let removed_twice = remove_nondeterminism(&removed_once);

        // ASSERT
        assert_eq!(removed_once, removed_twice, "Removal should be idempotent");
    }

    #[test]
    fn test_config_004_process_id_not_in_variable() {
        // ARRANGE
        let source = r#"export VAR$$NAME=value"#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT: Should still detect $$ even in variable name
        assert_eq!(constructs.len(), 1);
        assert_eq!(constructs[0].construct_type, ConstructType::ProcessId);
    }

    #[test]
    fn test_config_004_timestamp_variations() {
        // ARRANGE
        let source = r#"export T1=$(date)
export T2=$(date +%Y-%m-%d)
export T3=$(date -u +%s)"#;

        // ACT
        let constructs = analyze_nondeterminism(source);

        // ASSERT
        assert_eq!(constructs.len(), 3);
        assert!(constructs
            .iter()
            .all(|c| c.construct_type == ConstructType::Timestamp));
    }
}
