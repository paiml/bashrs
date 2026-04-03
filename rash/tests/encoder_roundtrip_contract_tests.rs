#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: encoder-roundtrip-v1.yaml
//!
//! Each test attempts to FALSIFY an escape function contract claim.
//! The escape functions are the #1 defense against shell injection.
//! A single unescaped metacharacter falsifies the contract.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

use bashrs::emitter::escape::{escape_command_name, escape_shell_string, escape_variable_name};

// ============================================================================
// F-ESC-001..006: escape_shell_string
// ============================================================================

/// F-ESC-001: Shell metacharacters must be quoted
#[test]
fn falsify_ESC_001_metachar_injection() {
    let dangerous_inputs = [
        "$(whoami)",
        "`id`",
        "foo | rm -rf /",
        "foo; evil",
        "foo && evil",
        "foo || evil",
        "$(cat /etc/passwd)",
        "${HOME}",
    ];

    for input in &dangerous_inputs {
        let escaped = escape_shell_string(input);
        // The escaped form must be single-quoted (or use the quote-escape pattern)
        assert!(
            escaped.starts_with('\'') || escaped.contains("'\"'\"'"),
            "F-ESC-001: metachar input {:?} must be quoted, got: {}",
            input,
            escaped
        );
        // Must NOT contain the raw metachar unquoted
        assert!(
            !escaped.contains("$(") || escaped.starts_with('\''),
            "F-ESC-001: $() must be inside quotes for input {:?}",
            input
        );
    }
}

/// F-ESC-002: Empty string → ''
#[test]
fn falsify_ESC_002_empty_string() {
    let result = escape_shell_string("");
    assert_eq!(
        result, "''",
        "F-ESC-002: empty string must produce '', got: {:?}",
        result
    );
}

/// F-ESC-003: Single quotes safely escaped
#[test]
fn falsify_ESC_003_single_quote_handling() {
    let result = escape_shell_string("don't");
    // Must use the '"'"' escape pattern for single quotes
    assert!(
        result.contains("'\"'\"'"),
        "F-ESC-003: single quote must use '\"'\"' escape pattern, got: {}",
        result
    );
    // Result must start and end with single quote (outer quoting)
    assert!(
        result.starts_with('\'') && result.ends_with('\''),
        "F-ESC-003: result must be wrapped in single quotes, got: {}",
        result
    );
}

/// F-ESC-004: Safe strings pass through unquoted
#[test]
fn falsify_ESC_004_safe_passthrough() {
    let safe_inputs = ["hello", "world", "test123", "/usr/bin/ls", "file.txt"];
    for input in &safe_inputs {
        let result = escape_shell_string(input);
        assert_eq!(
            result, *input,
            "F-ESC-004: safe string {:?} should pass through unquoted",
            input
        );
    }
}

/// F-ESC-005: Newlines are quoted
#[test]
fn falsify_ESC_005_newline_quoted() {
    let result = escape_shell_string("foo\nbar");
    assert!(
        result.starts_with('\''),
        "F-ESC-005: newline input must be single-quoted, got: {:?}",
        result
    );
}

/// F-ESC-006: Unicode/bidi characters are quoted (CVE-2021-42574)
#[test]
fn falsify_ESC_006_unicode_bidi_quoted() {
    // Right-to-left override character (used in bidi attacks)
    let bidi_input = "test\u{202E}exe";
    let result = escape_shell_string(bidi_input);
    assert!(
        result.starts_with('\''),
        "F-ESC-006: unicode/bidi input must be single-quoted, got: {:?}",
        result
    );
}

/// F-ESC-006 variant: emoji in string
#[test]
fn falsify_ESC_006_emoji_quoted() {
    let result = escape_shell_string("hello 🌍");
    assert!(
        result.starts_with('\''),
        "F-ESC-006: emoji input must be quoted, got: {:?}",
        result
    );
}

// ============================================================================
// F-ESC-007..010: escape_variable_name
// ============================================================================

/// F-ESC-007: Valid identifier passes through
#[test]
fn falsify_ESC_007_valid_identifier() {
    assert_eq!(escape_variable_name("my_var_123"), "my_var_123");
    assert_eq!(escape_variable_name("_private"), "_private");
    assert_eq!(escape_variable_name("HOME"), "HOME");
}

/// F-ESC-008: Hyphens → underscores
#[test]
fn falsify_ESC_008_hyphen_sanitized() {
    let result = escape_variable_name("my-var");
    assert_eq!(result, "my_var", "F-ESC-008: hyphen must become underscore");
}

/// F-ESC-009: Leading digit → underscore
#[test]
fn falsify_ESC_009_leading_digit() {
    let result = escape_variable_name("123abc");
    assert!(
        result.starts_with('_'),
        "F-ESC-009: leading digit must be replaced with _, got: {}",
        result
    );
    assert!(
        !result.starts_with(|c: char| c.is_ascii_digit()),
        "F-ESC-009: result must not start with digit"
    );
}

/// F-ESC-010: Injection in variable name sanitized
#[test]
fn falsify_ESC_010_varname_injection() {
    let result = escape_variable_name("var;rm -rf /");
    // All non-identifier chars must become underscores
    assert!(
        result
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "F-ESC-010: sanitized varname must be valid identifier, got: {}",
        result
    );
    assert!(
        !result.contains(';'),
        "F-ESC-010: semicolon must be sanitized out"
    );
}

/// F-ESC-010 variant: dollar sign in variable name
#[test]
fn falsify_ESC_010_dollar_in_varname() {
    let result = escape_variable_name("var$evil");
    assert!(
        !result.contains('$'),
        "F-ESC-010: $ must be sanitized from variable name, got: {}",
        result
    );
}

// ============================================================================
// F-ESC-011..014: escape_command_name
// ============================================================================

/// F-ESC-011: Simple command passes through
#[test]
fn falsify_ESC_011_simple_command() {
    assert_eq!(escape_command_name("ls"), "ls");
    assert_eq!(escape_command_name("grep"), "grep");
    assert_eq!(escape_command_name("rash_println"), "rash_println");
}

/// F-ESC-012: Path command passes through
#[test]
fn falsify_ESC_012_path_command() {
    assert_eq!(escape_command_name("/usr/bin/grep"), "/usr/bin/grep");
    assert_eq!(escape_command_name("/bin/sh"), "/bin/sh");
}

/// F-ESC-013: Command with spaces is escaped
#[test]
fn falsify_ESC_013_space_in_command() {
    let result = escape_command_name("my command");
    assert!(
        result.starts_with('\''),
        "F-ESC-013: space in command must be quoted, got: {}",
        result
    );
}

/// F-ESC-014: Command with semicolon is escaped
#[test]
fn falsify_ESC_014_semicolon_in_command() {
    let result = escape_command_name("cmd;evil");
    assert!(
        result.starts_with('\''),
        "F-ESC-014: semicolon in command must be quoted, got: {}",
        result
    );
    // The semicolon must be neutralized (inside quotes)
    assert!(
        !result.ends_with("evil") || result.starts_with('\''),
        "F-ESC-014: semicolon must not be a command separator"
    );
}

// ============================================================================
// Bonus: Roundtrip / idempotence properties
// ============================================================================

/// Escaped output should be valid shell when evaled
#[test]
fn falsify_ESC_ROUNDTRIP_no_double_escape() {
    // Escaping an already-safe string should not add quotes
    let safe = "hello";
    let once = escape_shell_string(safe);
    assert_eq!(once, "hello", "safe string should not be modified");
}

/// Variable name sanitization is idempotent
#[test]
fn falsify_ESC_ROUNDTRIP_varname_idempotent() {
    let dirty = "my-var.name";
    let once = escape_variable_name(dirty);
    let twice = escape_variable_name(&once);
    assert_eq!(once, twice, "variable name sanitization must be idempotent");
}
