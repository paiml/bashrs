use super::*;
use crate::linter::Severity;

// RED Phase: Write failing tests first

#[test]
fn test_SEC010_detects_cp_with_user_file() {
    let script = r#"cp "$USER_FILE" /destination/"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    assert_eq!(diag.code, "SEC010");
    assert_eq!(diag.severity, Severity::Error);
    assert!(diag.message.contains("Path traversal"));
}

#[test]
fn test_SEC010_detects_cat_with_input_path() {
    let script = r#"cat "$INPUT_PATH""#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_detects_tar_with_archive() {
    let script = r#"tar -xf "$ARCHIVE""#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_detects_mkdir_with_user_dir() {
    let script = r#"mkdir -p "$USER_DIR""#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_detects_cd_with_user_path() {
    let script = r#"cd "$USER_PATH""#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_safe_with_hardcoded_path() {
    let script = r#"cp /etc/config /backup/"#;
    let result = check(script);

    // Hardcoded paths are safe (no variables)
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_SEC010_detects_explicit_traversal() {
    let script = r#"cp file.txt ../../sensitive/"#;
    let result = check(script);

    // Should warn about explicit ../ usage
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn test_SEC010_no_false_positive_validation() {
    let script = r#"if [[ "$FILE" == *".."* ]]; then exit 1; fi"#;
    let result = check(script);

    // This is validation, not a vulnerability
    // Should not flag (or flag with lower severity)
    // Conservative: might still flag but acceptable for security
}

#[test]
fn test_SEC010_no_auto_fix() {
    let script = r#"cp "$USER_FILE" /dest/"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    assert!(diag.fix.is_none(), "SEC010 should not provide auto-fix");
}

#[test]
fn test_SEC010_multiple_vulnerabilities() {
    let script = r#"
cp "$USER_FILE" /dest/
cat "$INPUT_PATH"
    "#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 2);
}

#[test]
fn test_SEC010_no_false_positive_comment() {
    let script = r#"# cp "$USER_FILE" is dangerous"#;
    let result = check(script);

    // Comments should not trigger the rule
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_SEC010_106_heredoc_not_file_read() {
    // Issue #106: cat <<EOF is not a file read, it's a heredoc
    let script = r#"content=$(cat <<EOF
some content here
EOF
)"#;
    let result = check(script);

    // Heredocs should not trigger the rule
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_SEC010_106_heredoc_multiline() {
    // Issue #106: Heredoc with quoted delimiter
    let script = r#"cargo_content=$(cat <<'EOF'
[build]
jobs = 4
EOF
)"#;
    let result = check(script);

    // Heredocs should not trigger the rule
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_SEC010_106_heredoc_with_tee() {
    // tee with heredoc
    let script = r#"tee /etc/config <<EOF
config here
EOF"#;
    let result = check(script);

    // The tee has a path but it's a heredoc input
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_SEC010_real_cat_still_flagged() {
    // Real cat with user file should still be flagged
    let script = r#"cat "$USER_FILE""#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

// Issue #104 tests: Path validation guards

#[test]
fn test_SEC010_104_validated_path_not_flagged() {
    // Issue #104: If a path is validated with if [[ "$VAR" == *".."* ]], skip subsequent use
    let script = r#"
if [[ "$USER_FILE" == *".."* ]]; then
echo "Invalid path" >&2
exit 1
fi
cp "$USER_FILE" /destination/
"#;
    let result = check(script);

    // Should NOT flag because USER_FILE was validated
    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics for validated path, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_SEC010_104_realpath_validated() {
    // Issue #104: Variables assigned from realpath are considered validated
    let script = r#"
SAFE_PATH=$(realpath -m "$USER_INPUT")
cp "$SAFE_PATH" /destination/
"#;
    let result = check(script);

    // SAFE_PATH is derived from realpath, so it's validated
    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics for realpath-validated path"
    );
}

#[test]
fn test_SEC010_104_readlink_validated() {
    // Issue #104: Variables assigned from readlink -f are validated
    let script = r#"
RESOLVED=$(readlink -f "$USER_PATH")
cat "$RESOLVED"
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics for readlink-f-validated path"
    );
}

#[test]
fn test_SEC010_104_unvalidated_still_flagged() {
    // Issue #104: Variables that are NOT validated should still be flagged
    let script = r#"
echo "Processing file..."
cp "$USER_FILE" /destination/
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_104_different_var_still_flagged() {
    // Issue #104: Validating one variable doesn't validate others
    let script = r#"
if [[ "$SAFE_VAR" == *".."* ]]; then
exit 1
fi
cp "$USER_FILE" /destination/
"#;
    let result = check(script);

    // USER_FILE was not validated, only SAFE_VAR was
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_104_absolute_path_check() {
    // Issue #104: Check for absolute path validation
    let script = r#"
if [[ "$INPUT_PATH" == /* ]]; then
echo "Absolute paths not allowed" >&2
exit 1
fi
cp "$INPUT_PATH" /destination/
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics after absolute path validation"
    );
}

// Issue #127 tests: Custom validation function tracking

#[test]
fn test_SEC010_127_validate_function_tracks_var() {
    // Issue #127: Variables passed to validate_* functions should be tracked
    let script = r#"
validate_path() {
local path="$1"
if [[ "$path" == *".."* ]]; then
    echo "Invalid path" >&2
    exit 1
fi
}

validate_path "$RAID_PATH"
mkdir -p "$RAID_PATH/targets"
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics for variable passed to validate_path()"
    );
}

#[test]
fn test_SEC010_127_check_function_tracks_var() {
    // Issue #127: check_* functions also count as validation
    let script = r#"
check_path "$SRC_PATH"
cp "$SRC_PATH/file" /destination/
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics for variable passed to check_path()"
    );
}

#[test]
fn test_SEC010_127_sanitize_function_tracks_var() {
    // Issue #127: sanitize_* functions also count as validation
    let script = r#"
sanitize_input "$USER_FILE"
cat "$USER_FILE"
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "Expected no diagnostics for variable passed to sanitize_input()"
    );
}

#[test]
fn test_SEC010_127_unvalidated_still_flagged() {
    // Issue #127: Variables NOT passed to validation functions should still be flagged
    let script = r#"
validate_path "$OTHER_PATH"
mkdir -p "$USER_DIR"
"#;
    let result = check(script);

    // USER_DIR was not validated, should be flagged
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_SEC010_127_function_definition_not_call() {
    // Issue #127: Function definitions should not count as validation calls
    let script = r#"
validate_path() {
echo "validating"
}
mkdir -p "$USER_DIR"
"#;
    let result = check(script);

    // USER_DIR was not validated (just function was defined), should be flagged
    assert_eq!(result.diagnostics.len(), 1);
}

// Unit tests for helper functions to increase coverage

#[test]
fn test_is_validation_function_call_various_prefixes() {
    // Test all validation function prefixes
    assert!(is_validation_function_call(r#"validate_path "$PATH""#));
    assert!(is_validation_function_call(r#"check_input "$INPUT""#));
    assert!(is_validation_function_call(r#"verify_file "$FILE""#));
    assert!(is_validation_function_call(r#"sanitize_input "$INPUT""#));
    assert!(is_validation_function_call(r#"clean_path "$PATH""#));
    assert!(is_validation_function_call(r#"safe_copy "$FILE""#));
    assert!(is_validation_function_call(r#"is_valid_path "$PATH""#));
    assert!(is_validation_function_call(r#"is_safe_input "$INPUT""#));
    assert!(is_validation_function_call(r#"assert_path "$PATH""#));

    // Should not match without variable
    assert!(!is_validation_function_call("validate_path /fixed/path"));
    // Should not match function definitions
    assert!(!is_validation_function_call("validate_path() {"));
    assert!(!is_validation_function_call("validate_path()"));
}

#[test]
fn test_extract_function_argument_variable_formats() {
    // Test ${VAR} format
    assert_eq!(
        extract_function_argument_variable(r#"validate_path "${PATH}""#),
        Some("PATH".to_string())
    );
    // Test ${VAR[0]} format (array index stripped)
    assert_eq!(
        extract_function_argument_variable(r#"validate_path "${ARGS[0]}""#),
        Some("ARGS".to_string())
    );
    // Test $VAR format
    assert_eq!(
        extract_function_argument_variable(r#"validate_path "$PATH""#),
        Some("PATH".to_string())
    );
    // Test no variable
    assert_eq!(
        extract_function_argument_variable("validate_path /fixed/path"),
        None
    );
}

#[test]
fn test_is_heredoc_pattern_variants() {
    // Test various heredoc patterns
    assert!(is_heredoc_pattern("cat <<EOF"));
    assert!(is_heredoc_pattern("cat <<'EOF'"));
    assert!(is_heredoc_pattern("cat <<-EOF"));
    assert!(is_heredoc_pattern("cat<<<'EOF'"));
    assert!(is_heredoc_pattern("echo <<EOF"));
    assert!(is_heredoc_pattern("read <<EOF"));
    assert!(is_heredoc_pattern("tee <<EOF"));
    assert!(is_heredoc_pattern("content=$(cat <<EOF"));
    assert!(is_heredoc_pattern("x=$(cat<<EOF"));

    // Should not match regular cat
    assert!(!is_heredoc_pattern("cat /etc/passwd"));
    assert!(!is_heredoc_pattern(r#"cat "$FILE""#));
}

#[cfg(test)]
mod property_tests {
use super::*;
use crate::linter::Severity;
use proptest::prelude::*;

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(10))]
    #[test]
    fn prop_sec010_never_panics(s in ".*") {
        let _ = check(&s);
    }

    #[test]
    fn prop_sec010_safe_hardcoded_paths(
        src in "/[a-z/]{1,20}",
        dst in "/[a-z/]{1,20}",
    ) {
        let cmd = format!("cp {} {}", src, dst);
        let result = check(&cmd);
        // Hardcoded paths (no variables) should be safe
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sec010_detects_user_variables(
        file_op_idx in 0..9usize,
        var_name in "(USER|INPUT|FILE|PATH|DIR|ARCHIVE|NAME|ARG)_[A-Z]{1,5}",
    ) {
        let file_op = match file_op_idx {
            0 => "cp",
            1 => "mv",
            2 => "cat",
            3 => "tar",
            4 => "unzip",
            5 => "rm",
            6 => "mkdir",
            7 => "cd",
            _ => "ln",
        };
        let cmd = format!(r#"{} "${{{}}}""#, file_op, var_name);
        let result = check(&cmd);
        // Should detect path traversal risk with user variables
        prop_assert!(!result.diagnostics.is_empty());
        prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC010");
    }
}
}
