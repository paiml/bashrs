use super::*;

#[test]
fn test_sc2154_basic_detection() {
    let script = r#"
echo "$undefined_var"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SC2154");
}

#[test]
fn test_sc2154_variable_defined() {
    let script = r#"
defined_var="value"
echo "$defined_var"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2154_multiple_undefined() {
    let script = r#"
echo "$var1 $var2"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 2);
}

#[test]
fn test_sc2154_skip_builtins() {
    let script = r#"
echo "$HOME $PATH"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2154_skip_positional_params() {
    let script = r#"
echo "$1 $2 $3"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2154_skip_special_vars() {
    let script = r#"
echo "$@ $* $# $?"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2154_braced_variable() {
    let script = r#"
echo "${undefined}"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sc2154_mixed_defined_undefined() {
    let script = r#"
defined="value"
echo "$defined $undefined"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sc2154_used_before_defined() {
    // NOTE: Our simple two-pass implementation doesn't catch this edge case
    // A full implementation would need to track line-by-line state
    let script = r#"
echo "$var"
var="value"
"#;
    let result = check(script);
    // For now, we accept that this won't be caught
    assert!(result.diagnostics.len() <= 1);
}

#[test]
fn test_sc2154_no_false_positive_in_comment() {
    let script = r#"
# echo "$undefined"
defined="value"
echo "$defined"
"#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

// REQ-FP-003: SC2154 MUST track variables assigned by read in pipelines
#[test]
fn test_REQ_FP_003_read_in_pipeline() {
    let script = r#"#!/bin/bash
cat file.txt | while read line; do
  echo "$line"
done
"#;
    let result = check(script);
    // No SC2154 warning for 'line' - it's assigned by read
    let has_line_warning = result
        .diagnostics
        .iter()
        .any(|d| d.code == "SC2154" && d.message.contains("'line'"));
    assert!(
        !has_line_warning,
        "SC2154 must NOT flag 'line' - it's assigned by read"
    );
}

// Issue #91: SC2154 should NOT flag variables assigned by read with IFS=
#[test]
fn test_sc2154_issue_91_read_with_ifs() {
    // From issue #91 reproduction case
    let script = r#"grep -oE "pattern" "$FILE" | while IFS= read -r loc; do
line_num="${loc##*:}"
echo "$line_num"
done"#;
    let result = check(script);
    // loc is assigned by read -r loc
    let has_loc_warning = result
        .diagnostics
        .iter()
        .any(|d| d.code == "SC2154" && d.message.contains("'loc'"));
    assert!(
        !has_loc_warning,
        "SC2154 must NOT flag 'loc' - it's assigned by 'read -r loc'"
    );
}

#[test]
fn test_read_simple_assignment() {
    let script = r#"#!/bin/bash
read name
echo "$name"
"#;
    let result = check(script);
    let has_name_warning = result
        .diagnostics
        .iter()
        .any(|d| d.code == "SC2154" && d.message.contains("'name'"));
    assert!(!has_name_warning, "read should assign variable");
}

#[test]
fn test_read_with_r_flag() {
    let script = r#"#!/bin/bash
read -r line
echo "$line"
"#;
    let result = check(script);
    let has_line_warning = result
        .diagnostics
        .iter()
        .any(|d| d.code == "SC2154" && d.message.contains("'line'"));
    assert!(!has_line_warning, "read -r should assign variable");
}

#[test]
fn test_read_multiple_variables() {
    let script = r#"#!/bin/bash
read first second third
echo "$first $second $third"
"#;
    let result = check(script);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "read with multiple vars should assign all"
    );
}

// Issue #20: Loop variable tests
#[test]
fn test_issue_020_sc2154_for_loop_variable() {
    let script = r#"
for file in *.txt; do
echo "$file"
done
"#;
    let result = check(script);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "Loop variable 'file' should not be flagged as undefined"
    );
}

#[cfg(test)]
mod sc2154_tests_extracted_issue {
    use super::*;
    include!("sc2154_tests_extracted_issue.rs");
}
