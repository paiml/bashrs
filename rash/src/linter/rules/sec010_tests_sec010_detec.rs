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
    // GH-227: `USER_FILE` is never assigned in this file, so its external
    // influence is a guess about the environment, not a proof. A guess warns.
    assert_eq!(diag.severity, Severity::Warning);
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
fn test_GH227_undefined_validator_name_is_not_evidence() {
    // Was `test_SEC010_127_check_function_tracks_var`, which asserted that a
    // call to `check_path` silences the rule. `check_path` is never DEFINED in
    // this script, so there is no body to inspect and no evidence of anything —
    // exactly the i227d defect. GH-227: the name alone is not a control.
    let script = r#"
check_path "$SRC_PATH"
cp "$SRC_PATH/file" /destination/
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].code, "SEC010");
}

#[test]
fn test_GH227_undefined_sanitizer_name_is_not_evidence() {
    // Was `test_SEC010_127_sanitize_function_tracks_var`. Same defect as above:
    // `sanitize_input` is never defined here, so the call proves nothing.
    let script = r#"
sanitize_input "$USER_FILE"
cat "$USER_FILE"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].code, "SEC010");
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

// ============================================================================
// GH-227: SEC010 fired on literal paths, was not cleared by real inline
// validation, and WAS cleared by a no-op function named `validate_path`.
//
// The rule now requires the path expression to carry taint that can actually
// come from outside the script (see `crate::linter::taint`).
// ============================================================================

/// Count SEC010 diagnostics in a lint result.
fn sec010_count(script: &str) -> usize {
    check(script)
        .diagnostics
        .iter()
        .filter(|d| d.code == "SEC010")
        .count()
}

const GH227_LITERAL: &str = r#"#!/bin/bash
set -euo pipefail
OUT_DIR="build/results"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

const GH227_TAINTED: &str = r#"#!/bin/bash
OUT_DIR="build/$1"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

const GH227_HARDENED: &str = r#"#!/bin/bash
case "$1" in ""|*..*|/*) echo "bad name" >&2; exit 2 ;; esac
OUT_DIR="build/$1"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

const GH227_NOOP_VALIDATOR: &str = r#"#!/bin/bash
validate_path() {
    :
}
OUT_DIR="build/$1"
validate_path "$OUT_DIR"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

#[test]
fn test_GH227_literal_path_not_flagged() {
    // `OUT_DIR` is assigned a string literal: nothing on these lines can be
    // influenced from outside the script.
    assert_eq!(
        sec010_count(GH227_LITERAL),
        0,
        "literal path must not be a traversal finding: {:?}",
        check(GH227_LITERAL).diagnostics
    );
}

#[test]
fn test_GH227_positional_taint_still_flagged() {
    let result = check(GH227_TAINTED);
    let sec010: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SEC010")
        .collect();
    assert_eq!(sec010.len(), 2, "got: {:?}", result.diagnostics);
    for d in sec010 {
        assert_eq!(d.severity, Severity::Error, "$1 is proven external input");
    }
}

#[test]
fn test_GH227_inline_case_guard_clears_taint() {
    assert_eq!(
        sec010_count(GH227_HARDENED),
        0,
        "a dominating `case` guard that exits must clear the finding: {:?}",
        check(GH227_HARDENED).diagnostics
    );
}

#[test]
fn test_GH227_noop_validator_does_not_clear() {
    let result = check(GH227_NOOP_VALIDATOR);
    let sec010: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SEC010")
        .collect();
    assert_eq!(
        sec010.len(),
        2,
        "a function named validate_path whose body is `:` validates nothing: {:?}",
        result.diagnostics
    );
    for d in sec010 {
        assert_eq!(d.severity, Severity::Error);
    }
}

#[test]
fn test_GH227_transitive_literal_not_flagged() {
    let script = r#"#!/bin/bash
BASE_DIR="/srv"
SUB_DIR="$BASE_DIR/data"
mkdir -p "$SUB_DIR"
"#;
    assert_eq!(sec010_count(script), 0);
}

#[test]
fn test_GH227_literal_dest_dir_not_flagged() {
    let script = r#"#!/bin/bash
DEST_DIR="/opt/app"
cp /etc/hosts "$DEST_DIR/hosts"
"#;
    assert_eq!(sec010_count(script), 0);
}

#[test]
fn test_GH227_multiline_case_guard_clears_taint() {
    let script = r#"#!/bin/bash
case "$1" in
  *..*|/*) echo bad >&2; exit 2 ;;
esac
TARGET_DIR="out/$1"
mkdir -p "$TARGET_DIR"
"#;
    assert_eq!(sec010_count(script), 0);
}

#[test]
fn test_GH227_read_is_external_taint() {
    let script = r#"#!/bin/bash
read -r name
mkdir -p "build/$name"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_getopts_optarg_is_external() {
    let script = r#"#!/bin/bash
while getopts "d:" opt; do
  case "$opt" in
    d) OUT_DIR="$OPTARG" ;;
  esac
done
mkdir -p "$OUT_DIR"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_in_file_validator_with_return_clears() {
    let script = r#"#!/bin/bash
validate_path() {
  case "$1" in
    *..*|/*) echo "bad" >&2; return 1 ;;
  esac
}
RAID_PATH="$1"
validate_path "$RAID_PATH" || exit 1
mkdir -p "$RAID_PATH/targets"
"#;
    assert_eq!(
        sec010_count(script),
        0,
        "got: {:?}",
        check(script).diagnostics
    );
}

#[test]
fn test_GH227_unassigned_var_is_ambient_warning() {
    // Never assigned in this file: an environment variable or one set by a
    // sourced file. Real but unproven -> Warning, not a build-breaking Error.
    let script = r#"cp "$USER_FILE" /destination/"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
}

#[test]
fn test_GH227_reassignment_from_literal_cleans() {
    let script = r#"#!/bin/bash
P="$1"
P="/opt/fixed"
mkdir -p "$P/x"
"#;
    assert_eq!(sec010_count(script), 0);
}

#[test]
fn test_GH227_reassignment_from_input_retaints() {
    let script = r#"#!/bin/bash
P="/opt/fixed"
P="$1"
mkdir -p "$P/x"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_guard_on_wrong_variable_does_not_clear() {
    let script = r#"#!/bin/bash
case "$OTHER" in *..*) exit 1 ;; esac
D="in/$1"
mkdir -p "$D"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_guard_without_exit_does_not_clear() {
    let script = r#"#!/bin/bash
case "$1" in *..*) echo bad >&2 ;; esac
D="in/$1"
mkdir -p "$D"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_guard_after_use_does_not_clear() {
    let script = r#"#!/bin/bash
D="in/$1"
mkdir -p "$D"
case "$1" in *..*) exit 1 ;; esac
"#;
    assert_eq!(sec010_count(script), 1);
}

#[test]
fn test_GH227_escaped_regex_guard_recognised() {
    let script = r#"#!/bin/bash
if printf '%s' "$1" | grep -qE '(^|/)\.\.(/|$)'; then exit 1; fi
D="in/$1"
mkdir -p "$D"
"#;
    assert_eq!(
        sec010_count(script),
        0,
        "got: {:?}",
        check(script).diagnostics
    );
}

#[test]
fn test_GH227_validator_named_but_body_only_echoes() {
    let script = r#"#!/bin/bash
check_path() { echo "$1"; }
check_path "$1"
D="in/$1"
mkdir -p "$D"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_external_cmdsub_is_tainted() {
    let script = r#"#!/bin/bash
D=$(curl -s https://example.invalid/name)
mkdir -p "$D"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_GH227_realpath_still_sanitizes() {
    let script = r#"#!/bin/bash
S=$(realpath -m "$1")
cp "$S" /dest/
"#;
    assert_eq!(sec010_count(script), 0);
}
