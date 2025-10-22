//! Shell configuration file analyzer
//!
//! Analyzes shell configuration files to detect issues, performance problems,
//! and provide insights.

use super::{ConfigAnalysis, ConfigIssue, ConfigType, PathEntry, PerformanceIssue};
use crate::config::{aliaser, deduplicator, quoter};
use std::path::PathBuf;

/// Analyze a shell configuration file
pub fn analyze_config(source: &str, file_path: PathBuf) -> ConfigAnalysis {
    let config_type = ConfigType::from_path(&file_path);
    let line_count = source.lines().count();
    let mut issues = Vec::new();

    // Analyze PATH entries (CONFIG-001)
    let path_entries = deduplicator::analyze_path_entries(source);
    let path_issues = deduplicator::detect_duplicate_paths(&path_entries);
    issues.extend(path_issues);

    // Analyze unquoted variables (CONFIG-002)
    let unquoted_vars = quoter::analyze_unquoted_variables(source);
    let quote_issues = quoter::detect_unquoted_variables(&unquoted_vars);
    issues.extend(quote_issues);

    // Analyze duplicate aliases (CONFIG-003)
    let aliases = aliaser::analyze_aliases(source);
    let alias_issues = aliaser::detect_duplicate_aliases(&aliases);
    issues.extend(alias_issues);

    // TODO: Add more analysis rules
    // - CONFIG-004: Non-deterministic constructs
    // - CONFIG-005: Expensive operations

    let complexity_score = calculate_complexity(source);
    let performance_issues = detect_performance_issues(source);

    ConfigAnalysis {
        file_path,
        config_type,
        line_count,
        complexity_score,
        issues,
        path_entries,
        performance_issues,
    }
}

/// Calculate complexity score (0-10)
fn calculate_complexity(source: &str) -> u8 {
    let line_count = source.lines().count();

    // Simple heuristic based on line count
    if line_count < 50 {
        3
    } else if line_count < 100 {
        5
    } else if line_count < 200 {
        7
    } else {
        9
    }
}

/// Detect performance issues (CONFIG-005)
fn detect_performance_issues(source: &str) -> Vec<PerformanceIssue> {
    let mut issues = Vec::new();
    let mut line_num = 0;

    for line in source.lines() {
        line_num += 1;

        // Detect expensive eval operations
        if line.contains("eval") && (line.contains("rbenv") || line.contains("pyenv") || line.contains("nodenv")) {
            let command = line.trim().to_string();
            issues.push(PerformanceIssue {
                line: line_num,
                command,
                estimated_cost_ms: 150, // Rough estimate
                suggestion: "Consider lazy-loading this version manager".to_string(),
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_config_basic() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
export EDITOR="vim"
alias ll='ls -la'"#;
        let path = PathBuf::from("/home/user/.bashrc");

        // ACT
        let analysis = analyze_config(source, path);

        // ASSERT
        assert_eq!(analysis.config_type, ConfigType::Bashrc);
        assert_eq!(analysis.line_count, 3);
        assert_eq!(analysis.path_entries.len(), 1);
        assert_eq!(analysis.issues.len(), 0);
    }

    #[test]
    fn test_analyze_config_with_duplicates() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH""#;
        let path = PathBuf::from("/home/user/.bashrc");

        // ACT
        let analysis = analyze_config(source, path);

        // ASSERT
        assert_eq!(analysis.path_entries.len(), 3);
        assert_eq!(analysis.issues.len(), 1); // One duplicate
        assert_eq!(analysis.issues[0].rule_id, "CONFIG-001");
    }

    #[test]
    fn test_detect_performance_issues() {
        // ARRANGE
        let source = r#"eval "$(rbenv init -)""#;

        // ACT
        let issues = detect_performance_issues(source);

        // ASSERT
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 1);
        assert!(issues[0].command.contains("rbenv"));
    }
}
