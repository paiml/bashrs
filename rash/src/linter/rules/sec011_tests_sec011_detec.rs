
use super::*;

// RED Phase: Write failing tests first (EXTREME TDD)

/// RED TEST 1: Detect rm -rf without validation
#[test]
fn test_SEC011_detects_rm_rf_without_validation() {
    let script = r#"#!/bin/bash
rm -rf "$BUILD_DIR"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    assert_eq!(diag.code, "SEC011");
    assert_eq!(diag.severity, Severity::Error);
    assert!(diag.message.contains("BUILD_DIR"));
    assert!(diag.message.contains("rm -rf"));
}

/// RED TEST 2: Pass when rm -rf has validation
#[test]
fn test_SEC011_passes_rm_rf_with_validation() {
    let script = r#"#!/bin/bash
if [ -z "$BUILD_DIR" ] || [ "$BUILD_DIR" = "/" ]; then
  echo "Error: Invalid BUILD_DIR"
  exit 1
fi
rm -rf "$BUILD_DIR"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 0, "Should pass with validation");
}

/// RED TEST 3: Detect chmod -R 777 without validation
#[test]
fn test_SEC011_detects_chmod_without_validation() {
    let script = r#"#!/bin/bash
chmod -R 777 "$DIR"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    assert_eq!(diag.code, "SEC011");
    assert_eq!(diag.severity, Severity::Error);
    assert!(diag.message.contains("DIR"));
    assert!(diag.message.contains("chmod"));
}

/// RED TEST 4: Pass when chmod has validation
#[test]
fn test_SEC011_passes_chmod_with_validation() {
    let script = r#"#!/bin/bash
if [ -n "$DIR" ] && [ "$DIR" != "/" ]; then
  chmod -R 777 "$DIR"
fi
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 0, "Should pass with validation");
}

/// RED TEST 5: Detect chown -R without validation
#[test]
fn test_SEC011_detects_chown_without_validation() {
    let script = r#"#!/bin/bash
chown -R user:group "$DIR"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    assert_eq!(diag.code, "SEC011");
    assert_eq!(diag.severity, Severity::Error);
    assert!(diag.message.contains("DIR"));
    assert!(diag.message.contains("chown"));
}

/// RED TEST 6: Multiple violations in same script
#[test]
fn test_SEC011_detects_multiple_violations() {
    let script = r#"#!/bin/bash
rm -rf "$BUILD_DIR"
chmod -R 777 "$TEMP_DIR"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 2);
    assert_eq!(result.diagnostics[0].code, "SEC011");
    assert_eq!(result.diagnostics[1].code, "SEC011");
}

/// RED TEST 7: Safe operations without dangerous flags
#[test]
fn test_SEC011_passes_safe_operations() {
    let script = r#"#!/bin/bash
rm file.txt       # Not rm -rf
chmod 644 "$FILE" # Not chmod -R 777
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 0, "Safe operations should pass");
}

/// RED TEST 8: Validation with -n (non-zero check)
#[test]
fn test_SEC011_recognizes_n_validation() {
    let script = r#"#!/bin/bash
if [ -n "$VAR" ]; then
  rm -rf "$VAR"
fi
"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "Should recognize -n validation"
    );
}

// Issue #89: Inline validation with && chains
#[test]
fn test_SEC011_issue_89_inline_validation_with_and_chain() {
    // From issue #89 reproduction case
    let script = r#"[ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ] && rm -rf "$TEMP_DIR""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should recognize inline validation with && chains"
    );
}

#[test]
fn test_SEC011_issue_89_inline_n_validation() {
    let script = r#"[ -n "$VAR" ] && rm -rf "$VAR""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should recognize [ -n ] inline validation"
    );
}

#[test]
fn test_SEC011_issue_89_inline_d_validation() {
    let script = r#"[ -d "$DIR" ] && rm -rf "$DIR""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should recognize [ -d ] inline validation"
    );
}

#[test]
fn test_SEC011_issue_89_inline_if_block_validation() {
    // Another valid pattern from issue #89
    let script = r#"if [ -n "$VAR" ] && [ -d "$VAR" ]; then
rm -rf "$VAR"
fi"#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should recognize if-block validation"
    );
}

#[test]
fn test_SEC011_issue_89_still_detects_unvalidated() {
    // Should still detect rm -rf without validation
    let script = r#"rm -rf "$TEMP_DIR""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "SEC011 should still detect unvalidated rm -rf"
    );
}

#[test]
fn test_SEC011_issue_89_validation_wrong_var() {
    // Validation of different variable shouldn't count
    let script = r#"[ -n "$OTHER" ] && rm -rf "$TARGET""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "SEC011 should flag when different variable is validated"
    );
}

// Issue #105: Safe environment variables tests

#[test]
fn test_SEC011_105_user_env_var_safe() {
    // Issue #105: $USER is a safe environment variable
    let script = r#"rm -rf "/home/$USER/cache""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag $USER - it's a safe env var"
    );
}

#[test]
fn test_SEC011_105_home_env_var_safe() {
    // Issue #105: $HOME is a safe environment variable
    let script = r#"rm -rf "$HOME/.cache""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag $HOME - it's a safe env var"
    );
}

#[test]
fn test_SEC011_105_tmpdir_env_var_safe() {
    // Issue #105: $TMPDIR is a safe environment variable
    let script = r#"rm -rf "$TMPDIR/build""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag $TMPDIR - it's a safe env var"
    );
}

#[test]
fn test_SEC011_105_xdg_cache_safe() {
    // Issue #105: XDG_* variables are safe
    let script = r#"rm -rf "$XDG_CACHE_HOME/myapp""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag XDG_CACHE_HOME - it's a safe env var"
    );
}

#[test]
fn test_SEC011_105_custom_xdg_safe() {
    // Issue #105: Any XDG_* variable should be safe
    let script = r#"rm -rf "$XDG_CUSTOM_DIR/data""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag XDG_* vars - they're safe env vars"
    );
}

#[test]
fn test_SEC011_105_pwd_env_var_safe() {
    // Issue #105: $PWD is a safe environment variable
    let script = r#"rm -rf "$PWD/build""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag $PWD - it's a safe env var"
    );
}

#[test]
fn test_SEC011_105_unknown_var_still_flagged() {
    // Issue #105: Unknown variables should still be flagged
    let script = r#"rm -rf "$UNKNOWN_VAR""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "SEC011 should still flag unknown variables"
    );
}

#[test]
fn test_SEC011_105_chmod_with_home() {
    // Issue #105: chmod with $HOME should be safe
    let script = r#"chmod -R 777 "$HOME/.local""#;
    let result = check(script);

    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag chmod with $HOME"
    );
}

#[test]
fn test_SEC011_105_chown_with_user() {
    // Issue #105: chown with $USER should be safe
    let script = r#"chown -R "$USER":staff "/shared/docs""#;
    let result = check(script);

    // Note: USER is in the chown user:group part, not the path
    // The path is hardcoded "/shared/docs" - should be safe
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SEC011 should not flag chown with hardcoded path"
    );
}
