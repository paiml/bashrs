//! Additional tests for validation/mod.rs - improving coverage from 73% to 85%+

use super::*;

// ============================================================================
// ValidationLevel Tests
// ============================================================================

#[test]
fn test_validation_level_default() {
    let level = ValidationLevel::default();
    assert_eq!(level, ValidationLevel::Minimal);
}

#[test]
fn test_validation_level_ordering() {
    assert!(ValidationLevel::None < ValidationLevel::Minimal);
    assert!(ValidationLevel::Minimal < ValidationLevel::Strict);
    assert!(ValidationLevel::Strict < ValidationLevel::Paranoid);
}

#[test]
fn test_validation_level_equality() {
    assert_eq!(ValidationLevel::None, ValidationLevel::None);
    assert_eq!(ValidationLevel::Minimal, ValidationLevel::Minimal);
    assert_eq!(ValidationLevel::Strict, ValidationLevel::Strict);
    assert_eq!(ValidationLevel::Paranoid, ValidationLevel::Paranoid);

    assert_ne!(ValidationLevel::None, ValidationLevel::Minimal);
    assert_ne!(ValidationLevel::Strict, ValidationLevel::Paranoid);
}

#[test]
fn test_validation_level_serialization() {
    let level = ValidationLevel::Strict;
    let json = serde_json::to_string(&level).expect("Failed to serialize");
    let deserialized: ValidationLevel = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(level, deserialized);
}

#[test]
fn test_validation_level_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(ValidationLevel::None);
    set.insert(ValidationLevel::Minimal);
    set.insert(ValidationLevel::Strict);
    set.insert(ValidationLevel::Paranoid);

    assert_eq!(set.len(), 4);
    assert!(set.contains(&ValidationLevel::Strict));
}

// ============================================================================
// Severity Tests
// ============================================================================

#[test]
fn test_severity_variants() {
    let error = Severity::Error;
    let warning = Severity::Warning;
    let style = Severity::Style;

    assert_eq!(error.as_str(), "error");
    assert_eq!(warning.as_str(), "warning");
    assert_eq!(style.as_str(), "style");
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Error, Severity::Error);
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_eq!(Severity::Style, Severity::Style);

    assert_ne!(Severity::Error, Severity::Warning);
    assert_ne!(Severity::Warning, Severity::Style);
}

#[test]
fn test_severity_style_variant() {
    let style = Severity::Style;
    assert_eq!(style.as_str(), "style");
}

// ============================================================================
// ValidationError Tests
// ============================================================================

#[test]
fn test_validation_error_display_minimal() {
    let error = ValidationError {
        rule: "TEST001",
        severity: Severity::Warning,
        message: "Test message".to_string(),
        suggestion: None,
        auto_fix: None,
        line: None,
        column: None,
    };

    let display = format!("{error}");
    assert!(display.contains("TEST001"));
    assert!(display.contains("warning"));
    assert!(display.contains("Test message"));
    assert!(!display.contains("Suggestion:"));
}

#[test]
fn test_validation_error_display_with_suggestion() {
    let error = ValidationError {
        rule: "TEST002",
        severity: Severity::Error,
        message: "Error occurred".to_string(),
        suggestion: Some("Try this instead".to_string()),
        auto_fix: None,
        line: Some(42),
        column: Some(10),
    };

    let display = format!("{error}");
    assert!(display.contains("Suggestion: Try this instead"));
}

#[test]
fn test_validation_error_as_error_trait() {
    let error = ValidationError {
        rule: "TEST003",
        severity: Severity::Error,
        message: "Test error".to_string(),
        suggestion: None,
        auto_fix: None,
        line: None,
        column: None,
    };

    // Test that it implements std::error::Error
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_validation_error_with_all_fields() {
    let error = ValidationError {
        rule: "SC2086",
        severity: Severity::Error,
        message: "Unquoted variable expansion".to_string(),
        suggestion: Some("Quote the variable".to_string()),
        auto_fix: Some(Fix {
            description: "Add quotes".to_string(),
            replacement: "\"$VAR\"".to_string(),
        }),
        line: Some(15),
        column: Some(8),
    };

    assert_eq!(error.rule, "SC2086");
    assert_eq!(error.severity, Severity::Error);
    assert_eq!(error.line, Some(15));
    assert_eq!(error.column, Some(8));
    assert!(error.suggestion.is_some());
    assert!(error.auto_fix.is_some());
}

// ============================================================================
// Fix Tests
// ============================================================================

#[test]
fn test_fix_creation() {
    let fix = Fix {
        description: "Replace with safer alternative".to_string(),
        replacement: "safe_code".to_string(),
    };

    assert_eq!(fix.description, "Replace with safer alternative");
    assert_eq!(fix.replacement, "safe_code");
}

#[test]
fn test_fix_clone() {
    let fix = Fix {
        description: "Test fix".to_string(),
        replacement: "fixed".to_string(),
    };

    let cloned = fix.clone();
    assert_eq!(fix.description, cloned.description);
    assert_eq!(fix.replacement, cloned.replacement);
}

// ============================================================================
// ValidatedNode Tests
// ============================================================================

#[test]
fn test_validated_node_size() {
    // Ensure the struct is exactly 8 bytes as expected
    assert_eq!(std::mem::size_of::<ValidatedNode>(), 8);
}

// ============================================================================
// IMPLEMENTED_RULES Tests
// ============================================================================

#[test]
fn test_implemented_rules_access() {
    assert!(!IMPLEMENTED_RULES.is_empty());
    assert!(IMPLEMENTED_RULES.contains(&"SC2086"));
    assert!(IMPLEMENTED_RULES.contains(&"SC2046"));
    assert!(IMPLEMENTED_RULES.contains(&"SC2006"));
}

#[test]
fn test_implemented_rules_content() {
    let rules: Vec<&str> = IMPLEMENTED_RULES.to_vec();

    // Verify specific critical rules are present
    assert!(
        rules.contains(&"SC2086"),
        "Missing SC2086 (unquoted variables)"
    );
    assert!(
        rules.contains(&"SC2046"),
        "Missing SC2046 (unquoted command substitution)"
    );
    assert!(
        rules.contains(&"SC2164"),
        "Missing SC2164 (cd without error check)"
    );
    assert!(
        rules.contains(&"SC2162"),
        "Missing SC2162 (read without -r)"
    );
}

// ============================================================================
// validate_shell_snippet Tests
// ============================================================================

#[test]
fn test_validate_shell_snippet_valid() {
    let result = validate_shell_snippet("echo \"hello world\"");
    assert!(result.is_ok());
}

#[test]
fn test_validate_shell_snippet_invalid_backticks() {
    let result = validate_shell_snippet("echo `date`");
    assert!(result.is_err());
}

#[test]
fn test_validate_shell_snippet_invalid_cd() {
    let result = validate_shell_snippet("cd /tmp");
    assert!(result.is_err());
}

#[test]
fn test_validate_shell_snippet_empty() {
    let result = validate_shell_snippet("");
    assert!(result.is_ok());
}

#[test]
fn test_validate_shell_snippet_multiline() {
    let snippet = r#"
echo "Starting"
cd /tmp || exit 1
read -r var
echo "Done"
"#;
    let result = validate_shell_snippet(snippet);
    assert!(result.is_ok());
}

// ============================================================================
// ValidationError Clone Tests
// ============================================================================

#[test]
fn test_validation_error_clone() {
    let error = ValidationError {
        rule: "TEST004",
        severity: Severity::Warning,
        message: "Clone test".to_string(),
        suggestion: Some("Fix it".to_string()),
        auto_fix: Some(Fix {
            description: "Auto fix".to_string(),
            replacement: "fixed".to_string(),
        }),
        line: Some(10),
        column: Some(5),
    };

    let cloned = error.clone();
    assert_eq!(error.rule, cloned.rule);
    assert_eq!(error.severity, cloned.severity);
    assert_eq!(error.message, cloned.message);
    assert_eq!(error.line, cloned.line);
    assert_eq!(error.column, cloned.column);
}

// ============================================================================
// ValidationLevel Copy/Clone Tests
// ============================================================================

#[test]
fn test_validation_level_copy() {
    let level = ValidationLevel::Strict;
    let copied = level;
    assert_eq!(level, copied);
}

#[test]
fn test_validation_level_clone() {
    let level = ValidationLevel::Paranoid;
    let cloned = level;
    assert_eq!(level, cloned);
}

// ============================================================================
// Severity Copy/Clone Tests
// ============================================================================

#[test]
fn test_severity_copy() {
    let sev = Severity::Error;
    let copied = sev;
    assert_eq!(sev, copied);
}

#[test]
fn test_severity_clone() {
    let sev = Severity::Warning;
    let cloned = sev;
    assert_eq!(sev, cloned);
}
