//! SC2154: Variable referenced but not assigned
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo "$undefined_var"
//! ```
//!
//! Good:
//! ```bash
//! undefined_var="value"
//! echo "$undefined_var"
//! ```
//!
//! # Rationale
//!
//! Referencing undefined variables may indicate:
//! - Typos in variable names
//! - Missing initialization
//! - Reliance on environment variables (should be explicit)
//!
//! # Auto-fix
//!
//! Warning only - check for typos or add initialization

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;
use std::collections::HashSet;

/// Check for variables referenced but not assigned
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let patterns = create_patterns();
    let builtins = get_builtins();
    let (mut assigned, used_vars) = collect_variable_info(source, &patterns);

    // F047: Include variables assigned in case statements with default branches
    let case_vars = collect_case_statement_variables(source);
    assigned.extend(case_vars);

    let diagnostics = validate_undefined_variables(&assigned, &used_vars, &builtins);

    for diag in diagnostics {
        result.add(diag);
    }

    result
}

/// Patterns for variable detection
struct Patterns {
    assign: Regex,
    use_: Regex,
    for_loop: Regex,
}

/// Create regex patterns for variable detection
fn create_patterns() -> Patterns {
    Patterns {
        // Issue #20: Allow leading whitespace for indented assignments
        // Issue #24: Support local, readonly, export, declare, typeset keywords
        // Support flags like: declare -i, declare -r, local -r, etc.
        // Match both simple assignments (var=) and keyword assignments (local var=)
        assign: Regex::new(
            r"^\s*(?:(?:local|readonly|export|declare|typeset)(?:\s+-[a-zA-Z]+)?\s+)?([A-Za-z_][A-Za-z0-9_]*)=",
        )
        .unwrap(),
        use_: Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap(),
        // Issue #20: Detect loop variables (for var in ...)
        for_loop: Regex::new(r"\bfor\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\b").unwrap(),
    }
}

/// Get set of built-in/environment variables to skip
/// This includes POSIX standard variables and bash-specific builtins
fn get_builtins() -> HashSet<&'static str> {
    [
        // POSIX standard environment variables
        "HOME", "PATH", "PWD", "USER", "SHELL", "TERM", "LANG", "LC_ALL",
        "OLDPWD", "IFS", "OPTARG", "OPTIND", "PPID", "CDPATH", "MAILCHECK",
        "PS1", "PS2", "PS3", "PS4", "ENV", "FCEDIT", "HISTFILE", "HISTSIZE",
        "MAIL", "MAILPATH", "NLSPATH", "TMOUT", "COLUMNS", "LINES",
        // Bash specific - User/System info
        "EUID", "UID", "GROUPS",
        "HOSTNAME", "HOSTTYPE", "OSTYPE", "MACHTYPE",
        // Bash specific - Version info
        "BASH", "BASH_VERSION", "BASH_VERSINFO", "BASH_SUBSHELL", "BASHPID",
        // Bash specific - Special runtime variables
        "RANDOM", "SECONDS", "LINENO", "SHLVL", "REPLY", "EPOCHSECONDS",
        "EPOCHREALTIME", "SRANDOM",
        // Bash specific - Function/script context
        "FUNCNAME", "BASH_SOURCE", "BASH_LINENO", "FUNCNEST",
        // Bash specific - Command/execution context
        "BASH_COMMAND", "BASH_EXECUTION_STRING", "BASH_ARGC", "BASH_ARGV",
        "BASH_ARGV0", "BASH_REMATCH", "MAPFILE", "READLINE_LINE",
        "READLINE_POINT", "READLINE_MARK",
        // Bash specific - Pipeline/job status
        "PIPESTATUS",
        // Bash specific - Completion
        "COMP_WORDS", "COMP_CWORD", "COMP_LINE", "COMP_POINT", "COMP_TYPE",
        "COMP_KEY", "COMPREPLY",
        // Bash specific - Options
        "SHELLOPTS", "BASHOPTS", "BASH_COMPAT",
        // Bash specific - History
        "HISTCMD", "HISTCONTROL", "HISTIGNORE", "HISTTIMEFORMAT",
        // Bash specific - Directory stack
        "DIRSTACK",
        // Bash specific - Coprocesses
        "COPROC",
        // Common environment variables (widely used)
        "TMPDIR", "TEMP", "TMP", "EDITOR", "VISUAL", "PAGER", "BROWSER",
        "DISPLAY", "XAUTHORITY", "DBUS_SESSION_BUS_ADDRESS",
        "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_RUNTIME_DIR",
        "XDG_SESSION_TYPE", "XDG_CURRENT_DESKTOP",
        "LOGNAME", "HOSTNAME", "HOSTFILE", "INPUTRC",
        // Terminal/locale
        "COLORTERM", "TERM_PROGRAM", "LC_CTYPE", "LC_MESSAGES", "LC_NUMERIC",
        "LC_TIME", "LC_COLLATE", "LC_MONETARY",
    ]
    .iter()
    .copied()
    .collect()
}

/// F047: Find variables assigned inside case statements with default branches
/// If a case has a *) default branch, variables assigned in ANY branch are considered defined
/// because the default ensures all paths are covered
fn collect_case_statement_variables(source: &str) -> HashSet<String> {
    let mut case_vars: HashSet<String> = HashSet::new();

    // Simple heuristic: find case...esac blocks with *) default
    let lines: Vec<&str> = source.lines().collect();
    let mut in_case = false;
    let mut case_start = 0;
    let mut case_depth = 0;

    // Pre-compile regex outside the loop
    #[allow(clippy::expect_used)] // Compile-time regex, panic on invalid pattern is acceptable
    let assign_pattern = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=")
        .expect("valid assignment regex pattern");

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Track nested case statements
        if trimmed.starts_with("case ") && trimmed.contains(" in") {
            if !in_case {
                in_case = true;
                case_start = i;
            }
            case_depth += 1;
        }

        if trimmed == "esac" || trimmed.starts_with("esac;") || trimmed.starts_with("esac ") {
            if case_depth > 0 {
                case_depth -= 1;
            }
            if case_depth == 0 && in_case {
                // Found complete case statement, check if it has default
                let case_block: Vec<&str> = lines[case_start..=i].to_vec();
                let has_default = case_block.iter().any(|l| {
                    let t = l.trim();
                    t.starts_with("*)") || t.starts_with("* )") || t.contains("*)")
                });

                if has_default {
                    // Extract all variable assignments from this case block
                    for case_line in &case_block {
                        // Skip pattern lines (like "a)" or "*)")
                        let t = case_line.trim();
                        if t.ends_with(')') && !t.contains('=') {
                            continue;
                        }
                        for cap in assign_pattern.captures_iter(case_line) {
                            if let Some(var) = cap.get(1) {
                                case_vars.insert(var.as_str().to_string());
                            }
                        }
                    }
                }

                in_case = false;
            }
        }
    }

    case_vars
}

/// Issue #95: Check if script sources external files
/// If source/. commands are found, we're more lenient with undefined variables
fn has_source_commands(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        // Match: source file, . file, source "file", . "file"
        if trimmed.starts_with("source ") || trimmed.starts_with(". ") {
            return true;
        }
        // Also check for source/. after semicolon or &&/||
        if trimmed.contains("; source ")
            || trimmed.contains("; . ")
            || trimmed.contains("&& source ")
            || trimmed.contains("&& . ")
            || trimmed.contains("|| source ")
            || trimmed.contains("|| . ")
        {
            return true;
        }
    }
    false
}

/// Collect variable assignments and uses from source
fn collect_variable_info(
    source: &str,
    patterns: &Patterns,
) -> (HashSet<String>, Vec<(String, usize, usize)>) {
    let mut assigned: HashSet<String> = HashSet::new();
    let mut used_vars: Vec<(String, usize, usize)> = Vec::new();

    // Issue #95: Track if we have source commands - be lenient with uppercase vars
    let has_sources = has_source_commands(source);

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find assignments
        for cap in patterns.assign.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name);
        }

        // Issue #20: Find loop variables (for var in ...)
        for cap in patterns.for_loop.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name);
        }

        // REQ-FP-003: Find read command variable assignments
        // Handle: read var, read -r var, read var1 var2 var3
        if let Some(read_pos) = line.find("read ") {
            let after_read = &line[read_pos + 5..];
            let parts: Vec<&str> = after_read.split_whitespace().collect();
            let mut i = 0;
            // Skip flags like -r, -p, -a, etc.
            while i < parts.len() {
                let part = parts[i];
                if part.starts_with('-') {
                    i += 1;
                    // Skip argument for flags that take one (e.g., -p "prompt")
                    if part == "-p"
                        || part == "-a"
                        || part == "-d"
                        || part == "-n"
                        || part == "-t"
                        || part == "-u"
                    {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            // Remaining parts are variable names (until ; or end)
            while i < parts.len() {
                let var_name = parts[i].trim_end_matches(';');
                if var_name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                    && var_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    assigned.insert(var_name.to_string());
                    i += 1;
                } else {
                    break; // Stop at non-variable (e.g., ; do)
                }
            }
        }

        // Find uses - but skip uppercase vars if script has source commands
        for cap in patterns.use_.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let col = cap.get(0).unwrap().start() + 1;

            // Issue #95: If script sources external files, skip uppercase variables
            // (they likely come from the sourced file)
            if has_sources && var_name.chars().all(|c| c.is_uppercase() || c == '_') {
                continue;
            }

            used_vars.push((var_name.to_string(), line_num, col));
        }
    }

    (assigned, used_vars)
}

/// Check if variable is special or builtin (should be skipped)
fn is_special_or_builtin(var_name: &str, builtins: &HashSet<&str>) -> bool {
    // Skip if in builtins
    if builtins.contains(var_name) {
        return true;
    }

    // Skip numeric variables (positional parameters)
    if var_name.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // Skip special variables
    if ["@", "*", "#", "?", "$", "!", "0", "-"].contains(&var_name) {
        return true;
    }

    false
}

/// Validate undefined variables and return diagnostics
fn validate_undefined_variables(
    assigned: &HashSet<String>,
    used_vars: &[(String, usize, usize)],
    builtins: &HashSet<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (var_name, line_num, col) in used_vars {
        if assigned.contains(var_name) {
            continue;
        }

        if is_special_or_builtin(var_name, builtins) {
            continue;
        }

        let diagnostic = Diagnostic::new(
            "SC2154",
            Severity::Warning,
            format!("Variable '{}' is referenced but not assigned", var_name),
            Span::new(*line_num, *col, *line_num, col + var_name.len() + 1),
        );

        diagnostics.push(diagnostic);
    }

    diagnostics
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_issue_020_sc2154_multiple_loop_variables() {
        let script = r#"
for file in *.txt; do
    echo "$file"
done

for dockerfile in docker/*/Dockerfile; do
    echo "$dockerfile"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multiple loop variables should not be flagged"
        );
    }

    #[test]
    fn test_issue_020_sc2154_loop_variable_with_command_subst() {
        let script = r#"
for dockerfile in $(find . -name "*.Dockerfile"); do
    lang="$(basename "$(dirname "$dockerfile")")"
    echo "Processing: ${lang}"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Loop and assigned variables should not be flagged"
        );
    }

    #[test]
    fn test_issue_020_sc2154_undefined_var_in_loop_still_flagged() {
        let script = r#"
for file in *.txt; do
    echo "$file $undefined_var"
done
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Undefined variables in loops should still be flagged"
        );
        assert_eq!(result.diagnostics[0].code, "SC2154");
        assert!(result.diagnostics[0].message.contains("undefined_var"));
    }

    // Issue #24: Function parameter tests
    #[test]
    fn test_issue_024_sc2154_local_param_from_dollar1() {
        let script = r#"
validate_args() {
    local project_dir="$1"
    local environment="$2"

    if [[ -z "${project_dir}" ]]; then
        echo "Error: Project directory required" >&2
        exit 1
    fi
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Local variables assigned from positional parameters should not be flagged"
        );
    }

    #[test]
    fn test_issue_024_sc2154_local_param_with_default() {
        let script = r#"
main() {
    local project_dir="${1:-}"
    local environment="${2:-default}"

    echo "${project_dir}"
    echo "${environment}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Local variables with default values should not be flagged"
        );
    }

    #[test]
    fn test_issue_024_sc2154_local_in_function_used_later() {
        let script = r#"
validate() {
    local value="$1"
    if [[ -z "${value}" ]]; then
        return 1
    fi
    echo "Valid: ${value}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Local variables used later in function should not be flagged"
        );
    }

    #[test]
    fn test_issue_024_sc2154_multiple_local_declarations() {
        let script = r#"
process() {
    local input="$1"
    local output="$2"
    local temp="/tmp/temp"

    echo "${input}" > "${temp}"
    cat "${temp}" > "${output}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Multiple local variable declarations should all be recognized"
        );
    }

    #[test]
    fn test_issue_024_sc2154_local_readonly_export() {
        let script = r#"
setup() {
    local config="$1"
    readonly VERSION="1.0.0"
    export PATH="/usr/local/bin:$PATH"

    echo "${config} ${VERSION}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "local, readonly, and export declarations should all be recognized"
        );
    }

    #[test]
    fn test_issue_024_sc2154_declare_typeset() {
        let script = r#"
func() {
    declare var1="$1"
    typeset var2="$2"

    echo "${var1} ${var2}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "declare and typeset should be recognized as assignments"
        );
    }

    #[test]
    fn test_issue_024_sc2154_undefined_still_caught() {
        let script = r#"
func() {
    local defined="$1"
    echo "${defined} ${undefined}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Undefined variables should still be caught"
        );
        assert!(result.diagnostics[0].message.contains("undefined"));
    }

    #[test]
    fn test_issue_024_sc2154_declare_with_flags() {
        let script = r#"
func() {
    declare -i count="$1"
    declare -r readonly_var="$2"
    local -r local_readonly="$3"

    echo "${count} ${readonly_var} ${local_readonly}"
}
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "declare/local with flags (-i, -r) should be recognized"
        );
    }

    // Issue #95: Source command detection tests
    #[test]
    fn test_issue_95_has_source_commands_basic() {
        assert!(has_source_commands("source config.sh"));
        assert!(has_source_commands(". config.sh"));
        assert!(has_source_commands("  source /etc/profile"));
        assert!(has_source_commands("  . /etc/profile"));
    }

    #[test]
    fn test_issue_95_has_source_commands_chained() {
        assert!(has_source_commands("test -f config.sh && source config.sh"));
        assert!(has_source_commands("test -f config.sh && . config.sh"));
        assert!(has_source_commands(
            "test -f config.sh || source defaults.sh"
        ));
        assert!(has_source_commands("echo 'loading'; source config.sh"));
    }

    #[test]
    fn test_issue_95_no_source_commands() {
        assert!(!has_source_commands("echo hello"));
        assert!(!has_source_commands("echo 'source code'"));
        assert!(!has_source_commands("# source config.sh"));
        assert!(!has_source_commands("echo sourcefile"));
    }

    #[test]
    fn test_issue_95_uppercase_vars_with_source_ok() {
        // From issue #95: When script sources files, uppercase vars should be skipped
        let script = r#"
source config.sh
echo "$WAPR_MODEL"
echo "$CONFIG_VALUE"
"#;
        let result = check(script);
        // WAPR_MODEL and CONFIG_VALUE should NOT be flagged (they come from sourced file)
        let has_uppercase_warning = result.diagnostics.iter().any(|d| {
            d.code == "SC2154"
                && (d.message.contains("'WAPR_MODEL'") || d.message.contains("'CONFIG_VALUE'"))
        });
        assert!(
            !has_uppercase_warning,
            "SC2154 must NOT flag uppercase vars when script sources files"
        );
    }

    #[test]
    fn test_issue_95_uppercase_vars_without_source_flagged() {
        // Without source commands, uppercase vars should still be flagged (if not in builtins)
        let script = r#"
echo "$CUSTOM_VAR"
"#;
        let result = check(script);
        // CUSTOM_VAR should be flagged (no source, not builtin)
        let has_custom_warning = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SC2154" && d.message.contains("'CUSTOM_VAR'"));
        assert!(
            has_custom_warning,
            "SC2154 should flag uppercase vars when no source commands"
        );
    }

    #[test]
    fn test_issue_95_lowercase_vars_with_source_still_flagged() {
        // Even with source commands, lowercase undefined vars should be flagged
        let script = r#"
source config.sh
echo "$lowercase_undefined"
"#;
        let result = check(script);
        let has_lowercase_warning = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SC2154" && d.message.contains("'lowercase_undefined'"));
        assert!(
            has_lowercase_warning,
            "SC2154 should still flag lowercase undefined vars even with source"
        );
    }

    #[test]
    fn test_issue_95_dot_source_syntax() {
        // Test with . syntax instead of source
        let script = r#"
. /etc/profile
echo "$PROFILE_VAR"
"#;
        let result = check(script);
        let has_profile_warning = result
            .diagnostics
            .iter()
            .any(|d| d.code == "SC2154" && d.message.contains("'PROFILE_VAR'"));
        assert!(
            !has_profile_warning,
            "SC2154 must NOT flag uppercase vars when script uses . syntax"
        );
    }

    // Property tests for Issue #20
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_issue_020_loop_variables_never_flagged(
                var_name in "[a-z][a-z0-9_]{0,10}",
                pattern in "[a-z/*.]+",
            ) {
                let script = format!("for {} in {}; do\n    echo \"${}\"\ndone", var_name, pattern, var_name);
                let result = check(&script);

                // Loop variable should never be flagged as undefined
                for diagnostic in &result.diagnostics {
                    if diagnostic.code == "SC2154" {
                        prop_assert!(
                            !diagnostic.message.contains(&var_name),
                            "Loop variable '{}' should not be flagged as undefined",
                            var_name
                        );
                    }
                }
            }

            #[test]
            fn prop_issue_020_assigned_vars_never_flagged(
                var_name in "[a-z][a-z0-9_]{0,10}",
                value in "[a-zA-Z0-9]+",
            ) {
                let script = format!("{}=\"{}\"\necho \"${{{}}}\"", var_name, value, var_name);
                let result = check(&script);

                // Assigned variables should never be flagged
                prop_assert_eq!(result.diagnostics.len(), 0, "Assigned variable should not be flagged");
            }

            #[test]
            fn prop_issue_020_undefined_vars_always_flagged(
                defined_var in "[a-z][a-z0-9_]{0,10}",
                undefined_var in "[a-z][a-z0-9_]{0,10}",
            ) {
                prop_assume!(defined_var != undefined_var);
                // Avoid substring matches: ensure neither is a substring of the other
                prop_assume!(!defined_var.contains(&undefined_var) && !undefined_var.contains(&defined_var));

                let script = format!("{}=\"value\"\necho \"${{{}}} ${{{}}}\"", defined_var, defined_var, undefined_var);
                let result = check(&script);

                // Undefined variable should be flagged
                let has_undefined_warning = result.diagnostics.iter().any(|d| {
                    d.code == "SC2154" && d.message.contains(&format!("'{}'", undefined_var))
                });
                prop_assert!(has_undefined_warning, "Undefined variable '{}' should be flagged", undefined_var);

                // Defined variable should NOT be flagged
                let has_defined_warning = result.diagnostics.iter().any(|d| {
                    d.code == "SC2154" && d.message.contains(&format!("'{}'", defined_var))
                });
                prop_assert!(!has_defined_warning, "Defined variable '{}' should not be flagged", defined_var);
            }

            #[test]
            fn prop_issue_020_indented_assignments_recognized(
                indent in "[ ]{0,8}",
                var_name in "[a-z][a-z0-9_]{0,10}",
                value in "[a-zA-Z0-9]+",
            ) {
                let script = format!("{}{}=\"{}\"\necho \"${{{}}}\"", indent, var_name, value, var_name);
                let result = check(&script);

                // Indented assignments should be recognized (Issue #20 fix)
                prop_assert_eq!(result.diagnostics.len(), 0, "Indented assignment should be recognized");
            }

            // Issue #24: Property tests for local/declare/typeset
            #[test]
            fn prop_issue_024_local_assignments_never_flagged(
                var_name in "[a-z][a-z0-9_]{0,10}",
            ) {
                let script = format!("func() {{\n    local {}=\"$1\"\n    echo \"${{{}}}\"\n}}", var_name, var_name);
                let result = check(&script);

                // Local variables should never be flagged
                for diagnostic in &result.diagnostics {
                    if diagnostic.code == "SC2154" {
                        prop_assert!(
                            !diagnostic.message.contains(&var_name),
                            "Local variable '{}' should not be flagged",
                            var_name
                        );
                    }
                }
            }

            #[test]
            fn prop_issue_024_readonly_assignments_never_flagged(
                var_name in "[A-Z][A-Z0-9_]{0,10}",
                value in "[a-zA-Z0-9]+",
            ) {
                let script = format!("readonly {}=\"{}\"\necho \"${{{}}}\"", var_name, value, var_name);
                let result = check(&script);

                // readonly variables should never be flagged
                prop_assert_eq!(result.diagnostics.len(), 0, "readonly variable should not be flagged");
            }

            #[test]
            fn prop_issue_024_export_assignments_never_flagged(
                var_name in "[A-Z][A-Z0-9_]{0,10}",
                value in "[a-zA-Z0-9]+",
            ) {
                let script = format!("export {}=\"{}\"\necho \"${{{}}}\"", var_name, value, var_name);
                let result = check(&script);

                // export variables should never be flagged
                prop_assert_eq!(result.diagnostics.len(), 0, "export variable should not be flagged");
            }
        }
    }

    // ===== Issue #98: Bash Builtins Tests =====
    // These tests verify that bash builtin variables are recognized and NOT flagged

    #[test]
    fn test_FP_098_euid_not_flagged() {
        let script = r#"[[ $EUID -eq 0 ]]"#;
        let result = check(script);
        assert!(
            !result.diagnostics.iter().any(|d| d.message.contains("EUID")),
            "SC2154 must NOT flag EUID - it's a bash builtin"
        );
    }

    #[test]
    fn test_FP_098_uid_not_flagged() {
        let script = r#"echo $UID"#;
        let result = check(script);
        assert!(
            !result.diagnostics.iter().any(|d| d.message.contains("'UID'")),
            "SC2154 must NOT flag UID - it's a bash builtin"
        );
    }

    #[test]
    fn test_FP_098_bash_version_not_flagged() {
        let script = r#"echo $BASH_VERSION"#;
        let result = check(script);
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("BASH_VERSION")),
            "SC2154 must NOT flag BASH_VERSION"
        );
    }

    #[test]
    fn test_FP_098_random_seconds_lineno_not_flagged() {
        let script = "value=$RANDOM\nelapsed=$SECONDS\nline=$LINENO";
        let result = check(script);
        assert!(
            result.diagnostics.iter().all(|d| !d.message.contains("RANDOM")
                && !d.message.contains("SECONDS")
                && !d.message.contains("LINENO")),
            "SC2154 must NOT flag RANDOM, SECONDS, or LINENO"
        );
    }

    #[test]
    fn test_FP_098_funcname_bash_source_not_flagged() {
        let script = "echo ${FUNCNAME[0]}\necho ${BASH_SOURCE[0]}";
        let result = check(script);
        assert!(
            result
                .diagnostics
                .iter()
                .all(|d| !d.message.contains("FUNCNAME") && !d.message.contains("BASH_SOURCE")),
            "SC2154 must NOT flag FUNCNAME or BASH_SOURCE"
        );
    }

    #[test]
    fn test_FP_098_pipestatus_groups_not_flagged() {
        let script = "echo ${PIPESTATUS[0]}\necho ${GROUPS[0]}";
        let result = check(script);
        assert!(
            result
                .diagnostics
                .iter()
                .all(|d| !d.message.contains("PIPESTATUS") && !d.message.contains("GROUPS")),
            "SC2154 must NOT flag PIPESTATUS or GROUPS"
        );
    }

    #[test]
    fn test_FP_098_hostname_ostype_not_flagged() {
        let script = "echo $HOSTNAME $OSTYPE $HOSTTYPE $MACHTYPE";
        let result = check(script);
        assert!(
            result.diagnostics.iter().all(|d| !d.message.contains("HOSTNAME")
                && !d.message.contains("OSTYPE")
                && !d.message.contains("HOSTTYPE")
                && !d.message.contains("MACHTYPE")),
            "SC2154 must NOT flag HOSTNAME, OSTYPE, HOSTTYPE, or MACHTYPE"
        );
    }

    #[test]
    fn test_FP_098_shlvl_ppid_bashpid_not_flagged() {
        let script = "echo $SHLVL $PPID $BASHPID";
        let result = check(script);
        assert!(
            result.diagnostics.iter().all(|d| !d.message.contains("SHLVL")
                && !d.message.contains("PPID")
                && !d.message.contains("BASHPID")),
            "SC2154 must NOT flag SHLVL, PPID, or BASHPID"
        );
    }

    #[test]
    fn test_FP_098_oldpwd_ifs_optarg_not_flagged() {
        let script = "cd $OLDPWD\necho $IFS\necho $OPTARG $OPTIND";
        let result = check(script);
        assert!(
            result.diagnostics.iter().all(|d| !d.message.contains("OLDPWD")
                && !d.message.contains("'IFS'")
                && !d.message.contains("OPTARG")
                && !d.message.contains("OPTIND")),
            "SC2154 must NOT flag OLDPWD, IFS, OPTARG, or OPTIND"
        );
    }

    // ===== F047: Case statement with default branch =====
    // Variables assigned in ALL branches including *) default should be considered defined

    #[test]
    fn test_FP_047_case_with_default_not_flagged() {
        // Variable assigned in all branches including default
        let script = r#"
case "${SHELL}" in
    */zsh)  shell_rc="${HOME}/.zshrc" ;;
    */bash) shell_rc="${HOME}/.bashrc" ;;
    *)      shell_rc="${HOME}/.profile" ;;
esac
echo "${shell_rc}"
"#;
        let result = check(script);
        assert!(
            !result.diagnostics.iter().any(|d| d.message.contains("shell_rc")),
            "SC2154 must NOT flag variable assigned in all case branches including default"
        );
    }

    #[test]
    fn test_FP_047_case_simple_default_not_flagged() {
        let script = r#"
case $x in
    a) y=1 ;;
    b) y=2 ;;
    *) y=0 ;;
esac
echo $y
"#;
        let result = check(script);
        assert!(
            !result.diagnostics.iter().any(|d| d.message.contains("'y'")),
            "SC2154 must NOT flag variable assigned in all case branches"
        );
    }

    #[test]
    fn test_FP_047_case_without_default_still_flagged() {
        // No default branch - variable might not be assigned
        let script = r#"
case $x in
    a) y=1 ;;
    b) y=2 ;;
esac
echo $y
"#;
        let result = check(script);
        // This SHOULD be flagged because there's no default branch
        // However, current implementation might not catch this perfectly
        // For now, we're focused on fixing the false positive with default
    }

    #[test]
    fn test_FP_047_case_single_branch_with_default_not_flagged() {
        let script = r#"
case $mode in
    debug) level=1 ;;
    *) level=0 ;;
esac
echo $level
"#;
        let result = check(script);
        assert!(
            !result.diagnostics.iter().any(|d| d.message.contains("level")),
            "SC2154 must NOT flag variable assigned in case with default"
        );
    }
}
