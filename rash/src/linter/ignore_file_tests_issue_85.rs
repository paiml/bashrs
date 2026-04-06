use super::*;

// ============================================================
// Issue #85: Rule code support in .bashrsignore
// ============================================================

#[test]
fn test_issue_85_rule_codes_parsed() {
    // Issue #85: .bashrsignore should support rule codes like SEC010, SC2031
    let content = r#"
SEC010
SC2031
SC2032
SC2046
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Rule codes should be stored separately from file patterns
    assert!(ignore.should_ignore_rule("SEC010"));
    assert!(ignore.should_ignore_rule("SC2031"));
    assert!(ignore.should_ignore_rule("SC2032"));
    assert!(ignore.should_ignore_rule("SC2046"));
    assert!(!ignore.should_ignore_rule("SEC001")); // Not in file
}

#[test]
fn test_issue_85_rule_codes_case_insensitive() {
    let content = "sec010\nSC2031";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Rule codes should be case-insensitive
    assert!(ignore.should_ignore_rule("SEC010"));
    assert!(ignore.should_ignore_rule("sec010"));
    assert!(ignore.should_ignore_rule("SC2031"));
    assert!(ignore.should_ignore_rule("sc2031"));
}

#[test]
fn test_issue_85_mixed_content() {
    // .bashrsignore can contain both file patterns and rule codes
    let content = r#"
# Ignore vendor scripts
vendor/**/*.sh

# Ignore specific rules (Issue #85)
SEC010
SC2031

# Exclude specific file
scripts/record-metric.sh
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // File patterns work
    assert!(matches!(
        ignore.should_ignore(Path::new("vendor/foo.sh")),
        IgnoreResult::Ignored(_)
    ));

    // Rule codes work
    assert!(ignore.should_ignore_rule("SEC010"));
    assert!(ignore.should_ignore_rule("SC2031"));
    assert!(!ignore.should_ignore_rule("DET001"));
}

#[test]
fn test_issue_85_rule_code_patterns() {
    // Test various rule code formats that should be recognized
    let content = r#"
SEC001
SEC010
SC2031
SC2046
DET001
DET002
IDEM001
IDEM002
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    assert!(ignore.should_ignore_rule("SEC001"));
    assert!(ignore.should_ignore_rule("SEC010"));
    assert!(ignore.should_ignore_rule("SC2031"));
    assert!(ignore.should_ignore_rule("SC2046"));
    assert!(ignore.should_ignore_rule("DET001"));
    assert!(ignore.should_ignore_rule("DET002"));
    assert!(ignore.should_ignore_rule("IDEM001"));
    assert!(ignore.should_ignore_rule("IDEM002"));
}

#[test]
fn test_issue_85_get_ignored_rules() {
    let content = "SEC010\nSC2031\nDET001";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    let rules = ignore.ignored_rules();
    assert_eq!(rules.len(), 3);
    assert!(rules.contains(&"SEC010".to_string()));
    assert!(rules.contains(&"SC2031".to_string()));
    assert!(rules.contains(&"DET001".to_string()));
}

// ============================================================
// Issue #109: Line-specific and file-specific rule ignoring
// ============================================================

#[test]
fn test_issue_109_file_specific_ignore() {
    // Issue #109: Ignore rule only in specific file
    let content = "SEC010:scripts/install.sh";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Should be ignored in the specific file
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 1));

    // Should NOT be ignored in other files
    assert!(!ignore.should_ignore_rule_at("SEC010", Path::new("other/file.sh"), 1));

    // Global check should return false
    assert!(!ignore.should_ignore_rule("SEC010"));
}

#[test]
fn test_issue_109_line_specific_ignore() {
    // Issue #109: Ignore rule only on specific line
    let content = "DET001:scripts/metrics.sh:42";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Should be ignored on line 42
    assert!(ignore.should_ignore_rule_at("DET001", Path::new("scripts/metrics.sh"), 42));

    // Should NOT be ignored on other lines
    assert!(!ignore.should_ignore_rule_at("DET001", Path::new("scripts/metrics.sh"), 43));

    // Should NOT be ignored in other files
    assert!(!ignore.should_ignore_rule_at("DET001", Path::new("other/file.sh"), 42));
}

#[test]
fn test_issue_109_global_still_works() {
    // Issue #109: Global ignore should still work
    let content = "SEC010";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Global ignore applies everywhere
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("any/file.sh"), 1));
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("other/path.sh"), 999));
}

#[test]
fn test_issue_109_mixed_ignores() {
    // Mix of global, file-specific, and line-specific
    let content = r#"
SEC010
SC2031:scripts/install.sh
DET001:scripts/metrics.sh:42
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // SEC010 is global - applies everywhere
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("any/file.sh"), 1));

    // SC2031 is file-specific - only in scripts/install.sh
    assert!(ignore.should_ignore_rule_at("SC2031", Path::new("scripts/install.sh"), 1));
    assert!(!ignore.should_ignore_rule_at("SC2031", Path::new("other.sh"), 1));

    // DET001 is line-specific - only on line 42 of scripts/metrics.sh
    assert!(ignore.should_ignore_rule_at("DET001", Path::new("scripts/metrics.sh"), 42));
    assert!(!ignore.should_ignore_rule_at("DET001", Path::new("scripts/metrics.sh"), 41));
}

#[test]
fn test_issue_109_path_normalization() {
    // Paths should be normalized (./prefix removed)
    let content = "SEC010:./scripts/install.sh";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Both with and without ./ should match
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 1));
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("./scripts/install.sh"), 1));
}

#[test]
fn test_issue_109_case_insensitive_rule() {
    let content = "sec010:scripts/install.sh";
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Rule codes should be case-insensitive
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 1));
    assert!(ignore.should_ignore_rule_at("sec010", Path::new("scripts/install.sh"), 1));
}

#[test]
fn test_issue_109_multiple_rules_same_file() {
    // Multiple rules can be ignored in the same file
    let content = r#"
SEC010:scripts/install.sh
SC2031:scripts/install.sh
DET001:scripts/install.sh
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 1));
    assert!(ignore.should_ignore_rule_at("SC2031", Path::new("scripts/install.sh"), 1));
    assert!(ignore.should_ignore_rule_at("DET001", Path::new("scripts/install.sh"), 1));

    // Other rules should NOT be ignored
    assert!(!ignore.should_ignore_rule_at("IDEM001", Path::new("scripts/install.sh"), 1));
}

#[test]
fn test_issue_109_multiple_lines_same_file() {
    // Multiple line-specific ignores in the same file
    let content = r#"
SEC010:scripts/install.sh:10
SEC010:scripts/install.sh:20
SEC010:scripts/install.sh:30
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 10));
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 20));
    assert!(ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 30));

    // Other lines should NOT be ignored
    assert!(!ignore.should_ignore_rule_at("SEC010", Path::new("scripts/install.sh"), 15));
}

// ============================================================
// Original tests
// ============================================================

#[test]
fn test_empty_ignore_file() {
    let ignore = IgnoreFile::empty();
    assert!(!ignore.has_patterns());
    assert_eq!(
        ignore.should_ignore(Path::new("any/file.sh")),
        IgnoreResult::NotIgnored
    );
}

#[test]
fn test_parse_simple_pattern() {
    let content = "vendor/*.sh";
    let ignore = IgnoreFile::parse(content).expect("valid pattern");

    assert!(ignore.has_patterns());
    assert_eq!(ignore.pattern_count(), 1);

    assert!(matches!(
        ignore.should_ignore(Path::new("vendor/foo.sh")),
        IgnoreResult::Ignored(_)
    ));
    assert_eq!(
        ignore.should_ignore(Path::new("src/main.sh")),
        IgnoreResult::NotIgnored
    );
}

#[test]
fn test_parse_comments_and_empty_lines() {
    let content = r#"
# This is a comment
vendor/*.sh

# Another comment

**/generated/*.sh
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // Should have 2 patterns (comments and empty lines ignored)
    assert_eq!(ignore.pattern_count(), 2);
}

#[test]
fn test_negation_pattern() {
    let content = r#"
vendor/*.sh
!vendor/important.sh
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    // vendor/foo.sh should be ignored
    assert!(matches!(
        ignore.should_ignore(Path::new("vendor/foo.sh")),
        IgnoreResult::Ignored(_)
    ));

    // vendor/important.sh should NOT be ignored (negation)
    assert_eq!(
        ignore.should_ignore(Path::new("vendor/important.sh")),
        IgnoreResult::NotIgnored
    );
}

#[test]
fn test_double_star_pattern() {
    let content = "**/generated/*.sh";
    let ignore = IgnoreFile::parse(content).expect("valid pattern");

    assert!(matches!(
        ignore.should_ignore(Path::new("src/generated/foo.sh")),
        IgnoreResult::Ignored(_)
    ));
    assert!(matches!(
        ignore.should_ignore(Path::new("deep/path/generated/bar.sh")),
        IgnoreResult::Ignored(_)
    ));
    assert_eq!(
        ignore.should_ignore(Path::new("src/main.sh")),
        IgnoreResult::NotIgnored
    );
}

#[test]
fn test_exact_file_match() {
    let content = "scripts/record-metric.sh";
    let ignore = IgnoreFile::parse(content).expect("valid pattern");

    assert!(matches!(
        ignore.should_ignore(Path::new("scripts/record-metric.sh")),
        IgnoreResult::Ignored(_)
    ));
    assert_eq!(
        ignore.should_ignore(Path::new("scripts/other.sh")),
        IgnoreResult::NotIgnored
    );
}

#[test]
fn test_issue_58_record_metric_script() {
    // Issue #58: .bashrsignore should allow excluding record-metric.sh
    let content = r#"
# Metrics recording script from paiml-mcp-agent-toolkit
# Rationale: DET002 (timestamps) and SEC010 (paths) are false positives
scripts/record-metric.sh
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    assert!(matches!(
        ignore.should_ignore(Path::new("scripts/record-metric.sh")),
        IgnoreResult::Ignored(_)
    ));
}

#[test]
fn test_invalid_pattern_error() {
    // Invalid glob pattern (unclosed bracket)
    let content = "vendor/[invalid";
    let result = IgnoreFile::parse(content);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid pattern"));
}

#[test]
fn test_load_nonexistent_file() {
    let result = IgnoreFile::load(Path::new("/nonexistent/.bashrsignore"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_multiple_patterns() {
    let content = r#"
# Exclude vendor and generated
vendor/**/*.sh
generated/**/*.sh

# But keep important ones
!vendor/critical.sh
"#;
    let ignore = IgnoreFile::parse(content).expect("valid patterns");

    assert_eq!(ignore.pattern_count(), 3); // 2 include + 1 exclude

    // vendor/foo.sh - ignored
    assert!(matches!(
        ignore.should_ignore(Path::new("vendor/foo.sh")),
        IgnoreResult::Ignored(_)
    ));

    // vendor/critical.sh - NOT ignored (negation)
    assert_eq!(
        ignore.should_ignore(Path::new("vendor/critical.sh")),
        IgnoreResult::NotIgnored
    );

    // generated/output.sh - ignored
    assert!(matches!(
        ignore.should_ignore(Path::new("generated/output.sh")),
        IgnoreResult::Ignored(_)
    ));

    // src/main.sh - not matched
    assert_eq!(
        ignore.should_ignore(Path::new("src/main.sh")),
        IgnoreResult::NotIgnored
    );
}
