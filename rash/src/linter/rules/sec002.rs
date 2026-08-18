//! SEC002: Unquoted Variable in Command
//!
//! **Rule**: Detect unquoted variables in potentially dangerous commands
//!
//! **Why this matters**:
//! Unquoted variables can lead to command injection if they contain spaces
//! or special characters. This is especially dangerous in commands that
//! interact with the network, filesystem, or execute other commands.
//!
//! **Auto-fix**: Safe (add quotes)
//!
//! ## Examples
//!
//! ❌ **UNSAFE**:
//! ```bash
//! curl $URL
//! wget $FILE_PATH
//! ssh $HOST
//! git clone $REPO
//! ```
//!
//! ✅ **SAFE** (auto-fixable):
//! ```bash
//! curl "${URL}"
//! wget "${FILE_PATH}"
//! ssh "${HOST}"
//! git clone "${REPO}"
//! ```

use crate::linter::shell_words::{self, SimpleCommand, WordRole};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Dangerous commands that should never have unquoted variables
const DANGEROUS_COMMANDS: &[&str] = &[
    "curl", "wget", "ssh", "scp", "git", "rsync", "docker", "kubectl",
];

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// One reportable unquoted expansion.
struct Offence {
    /// 1-indexed byte column of the `$`.
    col: usize,
    /// 1-indexed byte column one past the expansion.
    end_col: usize,
    /// The dangerous command this expansion is an argument of.
    cmd: &'static str,
    /// The expansion exactly as written, e.g. `$url` or `${URL:-x}`.
    text: String,
}

/// Resolve a command name to the `DANGEROUS_COMMANDS` entry it matches.
fn dangerous_command(name: &str) -> Option<&'static str> {
    DANGEROUS_COMMANDS.iter().copied().find(|&c| c == name)
}

/// The leftmost unquoted expansion sitting in argument position of a dangerous
/// command. Expansions in command position are excluded on purpose: word
/// splitting there is intentional (`sh_c='sh -c'; $sh_c 'docker version'`) and
/// SC2183 already reports it with the appropriate severity. See GH-229.
fn command_offence(cmd: &SimpleCommand) -> Option<Offence> {
    let name = dangerous_command(cmd.name.as_deref()?)?;
    cmd.words
        .iter()
        .filter(|w| w.role == WordRole::Argument)
        .flat_map(|w| w.expansions.iter())
        .find(|e| !e.quoted)
        .map(|e| Offence {
            col: e.col,
            end_col: e.end_col,
            cmd: name,
            text: e.text.clone(),
        })
}

/// At most one finding per physical line - the leftmost offence.
fn first_offence(line: &str) -> Option<Offence> {
    shell_words::simple_commands(line)
        .iter()
        .filter_map(command_offence)
        .min_by_key(|o| o.col)
}

/// Create a SEC002 diagnostic for unquoted variable.
///
/// The span covers the whole expansion and uses byte columns, so
/// `autofix_apply::apply_single_fix` splices the replacement over exactly the
/// expansion it replaces (GH-228). The replacement quotes the source text
/// verbatim, so modifiers such as `${URL:-default}` survive the fix.
fn create_sec002_diagnostic(o: &Offence, line: usize) -> Diagnostic {
    let span = Span::new(line, o.col, line, o.end_col);

    Diagnostic::new(
        "SEC002",
        Severity::Error,
        format!(
            "Unquoted variable {} in {} command - add quotes",
            o.text, o.cmd
        ),
        span,
    )
    .with_fix(Fix::new(format!("\"{}\"", o.text)))
}

/// Check for unquoted variables in dangerous commands
pub fn check(source: &str) -> LintResult {
    if source.is_empty() {
        return LintResult::new();
    }
    // Contract: safety-classifier-v1.yaml precondition (pv codegen)
    contract_pre_classify_injection!(source);
    let mut result = LintResult::new();

    for (idx, line) in source.lines().enumerate() {
        // Skip comments
        if is_comment_line(line) {
            continue;
        }
        if let Some(o) = first_offence(line) {
            result.add(create_sec002_diagnostic(&o, idx + 1));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====
    // Establish invariants before refactoring

    #[test]
    fn prop_sec002_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# curl $URL",
            "  # wget $FILE",
            "\t# ssh $HOST",
            "# git clone $REPO",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Comments should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sec002_quoted_variables_never_diagnosed() {
        // Property: Properly quoted variables should never be diagnosed
        let test_cases = vec![
            r#"curl "${URL}""#,
            "wget \"$FILE_PATH\"",
            "ssh '$HOST'",
            r#"git clone "${REPO}""#,
            "docker run \"$IMAGE\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Quoted variables should be OK: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sec002_unquoted_dangerous_always_diagnosed() {
        // Property: Unquoted variables in dangerous commands should always be diagnosed
        let test_cases = vec![
            ("curl $URL", "curl"),
            ("wget $FILE", "wget"),
            ("ssh $HOST", "ssh"),
            ("git clone $REPO", "git"),
            ("docker run $IMAGE", "docker"),
        ];

        for (code, cmd) in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                1,
                "Unquoted {} should be diagnosed: {}",
                cmd,
                code
            );
            assert!(result.diagnostics[0].message.contains(cmd));
        }
    }

    #[test]
    fn prop_sec002_safe_commands_never_diagnosed() {
        // Property: Non-dangerous commands should not be diagnosed
        let test_cases = vec!["echo $VAR", "printf $FORMAT", "cat $FILE", "ls $DIR"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Safe commands should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sec002_diagnostic_code_always_sec002() {
        // Property: All diagnostics must have code "SEC002"
        let code = "curl $A\nwget $B\nssh $C";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SEC002");
        }
    }

    #[test]
    fn prop_sec002_diagnostic_severity_always_error() {
        // Property: All diagnostics must be Error severity
        let code = "curl $A\nwget $B";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Error);
        }
    }

    #[test]
    fn prop_sec002_all_diagnostics_have_fix() {
        // Property: All SEC002 diagnostics must provide a fix
        let code = "curl $URL\nwget $FILE";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert!(
                diagnostic.fix.is_some(),
                "All SEC002 diagnostics should have a fix"
            );
        }
    }

    #[test]
    fn prop_sec002_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sec002_only_one_diagnostic_per_line() {
        // Property: Only report first unquoted variable per line
        let code = "curl $URL $BACKUP";
        let result = check(code);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only report once per line"
        );
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_SEC002_detects_unquoted_curl() {
        let script = "curl $URL";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC002");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("curl"));
    }

    #[test]
    fn test_SEC002_detects_unquoted_wget() {
        let script = "wget $FILE_PATH";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC002_detects_unquoted_ssh() {
        let script = "ssh $HOST";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC002_no_warning_with_quotes() {
        let script = r#"curl "${URL}""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC002_no_warning_with_double_quotes() {
        let script = "wget \"$FILE_PATH\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC002_provides_fix() {
        let script = "curl $URL";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // GH-228: the fix is spliced over the span by `bashrs lint --fix`, so it must
        // be the real expansion. The old literal placeholder "$VAR" produced
        // `curl "$VAR"URL` on disk.
        assert_eq!(fix.replacement, "\"$URL\"");
    }

    #[test]
    fn test_SEC002_no_false_positive_comment() {
        let script = "# curl $URL";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Mutation Coverage Tests - Following SEC001 pattern (100% kill rate) =====

    #[test]
    fn test_mutation_sec002_unquoted_var_start_col_exact() {
        // MUTATION: Line 84:35 - replace + with * in line_num + 1
        // MUTATION: Line 84:63 - Tests start column calculation
        let bash_code = "curl $URL"; // $ at column 6
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // With correct arithmetic: start_col = 6
        // With mutation (+ → *): would produce incorrect column
        assert_eq!(
            span.start_col, 6,
            "Start column must use correct calculation"
        );
    }

    #[test]
    fn test_mutation_sec002_unquoted_var_end_col_exact() {
        // MUTATION: Line 84:63 - replace + with * in col + 1
        // MUTATION: Line 84:63 - replace + with - in col + 1
        // Tests end column calculation
        let bash_code = "curl $URL"; // `$URL` spans byte columns 6..10
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // GH-228: a 1-char span made `--fix` splice over the `$` alone and corrupt
        // the source. The span must cover the whole expansion, as SC2086 already does.
        assert_eq!(
            span.end_col, 10,
            "End column must cover the whole expansion so --fix splices correctly"
        );
    }

    #[test]
    fn test_mutation_sec002_line_num_calculation() {
        // MUTATION: Line 84:35 - replace + with * in line_num + 1
        // Tests line number calculation for multiline input
        let bash_code = "# comment\ncurl $URL"; // curl on line 2
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With +1: line 2
        // With *1: line 0
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Line number must use +1, not *1"
        );
    }

    #[test]
    fn test_mutation_sec002_column_with_offset() {
        // Tests column calculations with leading whitespace
        // Also catches Line 59:13 col += 1 mutation
        let bash_code = "    curl $URL"; // $ at column 10 (4 spaces + "curl " = 9, $ at 10)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_col, 10, "Must account for leading whitespace");
        assert_eq!(span.end_col, 14, "End must cover $URL");
    }

    #[test]
    fn test_mutation_sec002_column_tracking_accuracy() {
        // MUTATION: Line 59:13 - replace += with *= in col += 1
        // Test that column tracking is accurate for variables at various positions
        let bash_code = "curl       $URL"; // $ at column 12 (extra spaces)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With col += 1: correctly tracks to column 12
        // With col *= 1: would produce incorrect tracking
        assert_eq!(
            result.diagnostics[0].span.start_col, 12,
            "Column tracking must increment correctly"
        );
    }

    #[test]
    fn test_mutation_sec002_quote_detection_single_quotes() {
        // MUTATION: Line 62:20 - replace !in_single_quotes with true
        // Ensure single-quoted variables are not diagnosed
        let bash_code = "curl '$URL'"; // Should be safe (single quotes)
        let result = check(bash_code);
        // With correct logic: 0 diagnostics (single quotes protect variable)
        // With mutation (!in_single_quotes → true): might incorrectly diagnose
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Single-quoted variables should be safe"
        );
    }

    #[test]
    fn test_mutation_sec002_quote_detection_double_quotes() {
        // MUTATION: Line 62:20 - Additional test for quote tracking logic
        // Tests quote tracking logic comprehensively
        let bash_code = r#"curl "${URL}""#; // Should be safe (double quotes)
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Double-quoted variables should be safe"
        );
    }

    #[test]
    fn test_mutation_sec002_variable_detection_underscore() {
        // MUTATION: Line 69:56 - replace == with != in *c == '_'
        // Tests that underscore is correctly detected as part of variable names
        let bash_code = "curl $MY_VAR"; // Variable with underscore
        let result = check(bash_code);
        // With ==: detects $MY_VAR (correct)
        // With !=: might fail to detect underscore as valid variable char
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect variable with underscore"
        );
    }

    // ================================================================
    // GH-228: a command substitution is a FRESH quoting context
    // ================================================================

    #[test]
    fn test_GH228_cmdsub_quoted_variable_not_flagged() {
        // The regression: the assignment's opening `"` set in_double_quotes, then the
        // `"` before $url toggled it OFF, so a correctly quoted var looked bare.
        let s = r#"out="$(curl -sSfL "$url" | cut -d' ' -f1)""#;
        assert_eq!(check(s).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH228_cmdsub_unquoted_variable_flagged_once() {
        let s = r#"out="$(curl -sSfL $url | cut -d' ' -f1)""#;
        let r = check(s);
        assert_eq!(r.diagnostics.len(), 1);
        let d = &r.diagnostics[0];
        assert_eq!(d.span.start_col, 19);
        assert_eq!(d.span.end_col, 23); // `$url` is 4 bytes
        assert!(d.message.contains("curl"));
        assert!(d.message.contains("$url"));
        assert_eq!(d.fix.as_ref().unwrap().replacement, "\"$url\"");
    }

    #[test]
    fn test_GH228_cmdsub_simple_assignment() {
        let r = check(r#"x="$(curl $u)""#);
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_col, 11);
        assert_eq!(r.diagnostics[0].span.end_col, 13);
    }

    #[test]
    fn test_GH228_backtick_substitution_recursed() {
        let r = check("x=`curl $u`");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_col, 9);
    }

    #[test]
    fn test_GH228_second_pipeline_stage_is_fresh_command() {
        // `cut` is not dangerous; the `-d' '` single quotes must not leak.
        assert_eq!(
            check(r#"x="$(echo hi | cut -d' ' -f1)""#).diagnostics.len(),
            0
        );
    }

    #[test]
    fn test_GH228_braced_expansion_unquoted_is_flagged() {
        // Regression: `${URL}` was invisible because the old predicate demanded
        // an alphanumeric byte immediately after `$`.
        let r = check("curl ${URL}");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_col, 6);
        assert_eq!(r.diagnostics[0].span.end_col, 12);
        assert_eq!(
            r.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"${URL}\""
        );
    }

    #[test]
    fn test_GH228_braced_expansion_quoted_is_not_flagged() {
        assert_eq!(check(r#"curl "${URL:-$FALLBACK}""#).diagnostics.len(), 0);
    }

    // ================================================================
    // GH-229: command position is exempt
    // ================================================================

    #[test]
    fn test_GH229_dispatcher_variable_command_not_flagged() {
        let s = "sh_c='sh -c'\n\
                 if [ \"$(id -u)\" -ne 0 ]; then sh_c='sudo -E sh -c'; fi\n\
                 $sh_c 'docker version'";
        assert_eq!(check(s).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_command_name_inside_single_quotes_not_matched() {
        // `docker` here is literal text of an argument, not a command. The unquoted
        // $IMAGE belongs to `echo`, which is not dangerous.
        assert_eq!(check("echo 'docker run' $IMAGE").diagnostics.len(), 0);
        assert_eq!(check("echo 'docker run $IMAGE'").diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_command_name_inside_double_quotes_not_matched() {
        assert_eq!(check(r#"echo "docker run" $IMAGE"#).diagnostics.len(), 0);
        assert_eq!(check(r#"echo "docker run $IMAGE""#).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_braced_dispatcher_not_flagged() {
        assert_eq!(check("${SH_C} 'docker version'").diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_quoted_command_name_still_resolves() {
        // `"docker"` really does invoke docker, so this stays a true positive.
        assert_eq!(check(r#""docker" run $IMG"#).diagnostics.len(), 1);
    }

    // ================================================================
    // span / fix correctness
    // ================================================================

    #[test]
    fn test_GH228_span_covers_whole_expansion_for_autofix() {
        let r = check("curl $URL");
        let d = &r.diagnostics[0];
        assert_eq!((d.span.start_col, d.span.end_col), (6, 10));
        // Splicing the fix over the span must reconstruct valid shell.
        let line = "curl $URL";
        let fixed = format!(
            "{}{}{}",
            &line[..d.span.start_col - 1],
            d.fix.as_ref().unwrap().replacement,
            &line[d.span.end_col - 1..]
        );
        assert_eq!(fixed, r#"curl "$URL""#);
    }

    #[test]
    fn test_GH228_columns_are_byte_offsets_not_char_offsets() {
        // Char-based columns made `bashrs lint --fix` panic in apply_single_fix.
        let src = "A=1; curl é$U";
        let r = check(src);
        assert_eq!(r.diagnostics.len(), 1);
        let d = &r.diagnostics[0];
        // 'é' occupies bytes 11-12 (1-indexed), so `$` is at byte column 13.
        // A char-indexed column would be 12 and would split the 'é'.
        assert_eq!(d.span.start_col, 13);
        assert!(src.is_char_boundary(d.span.start_col - 1));
        assert!(src.is_char_boundary(d.span.end_col - 1));
    }

    #[test]
    fn test_GH228_braced_modifier_fix_preserves_source_text() {
        // The fix must wrap the expansion exactly as written; reconstructing
        // `${NAME}` from the parsed name would silently drop the `:-default`.
        let r = check("curl ${URL:-https://d}");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(
            r.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"${URL:-https://d}\""
        );
    }

    // ================================================================
    // command resolution: wrappers, paths, prefixes, keywords
    // ================================================================

    #[test]
    fn test_GH229_sudo_wrapper_resolves_to_real_command() {
        assert_eq!(
            check("sudo -E docker run $IMG").diagnostics[0].span.start_col,
            20
        );
    }

    #[test]
    fn test_GH229_timeout_numeric_operand_skipped() {
        assert_eq!(check("timeout 5 curl $U").diagnostics[0].span.start_col, 16);
    }

    #[test]
    fn test_GH229_wrapper_operand_rule_does_not_swallow_ssh() {
        // `ssh` must not be mistaken for a numeric operand (`ssh` minus 'h' == "ss").
        assert_eq!(check("sudo ssh $HOST").diagnostics.len(), 1);
    }

    #[test]
    fn test_GH229_wrapper_argument_named_docker_is_not_a_command() {
        // Measured false positive today: `docker` is a *group name* for usermod.
        assert_eq!(check("sudo usermod -aG docker $USER").diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_numeric_first_word_is_not_a_wrapper_operand() {
        // Doc transcript: `  18 kubectl set image deployment/$app_name ...`
        assert_eq!(
            check("  18 kubectl set image deployment/$app_name")
                .diagnostics
                .len(),
            0
        );
    }

    #[test]
    fn test_GH229_absolute_path_command_matches_by_basename() {
        assert_eq!(check("/usr/bin/curl $U").diagnostics[0].span.start_col, 15);
    }

    #[test]
    fn test_GH229_assignment_prefix_before_command_is_not_an_argument() {
        // Assignment RHS does not word-split, and `curl` is still the command.
        assert_eq!(check(r#"VAR=$X curl "$URL""#).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_assignment_shaped_argument_after_command_is_flagged() {
        // `myapp=myapp:$VERSION` is an ARGUMENT here; it does word-split.
        let r = check("kubectl set image deployment/myapp myapp=myapp:$VERSION");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_col, 48);
    }

    #[test]
    fn test_GH229_reserved_words_do_not_become_command_names() {
        assert_eq!(
            check("if ! curl $URL; then :; fi").diagnostics[0].span.start_col,
            11
        );
    }

    #[test]
    fn test_GH229_for_loop_word_is_command_name_of_its_own() {
        assert_eq!(
            check(r#"for u in $URLS; do curl "$u"; done"#)
                .diagnostics
                .len(),
            0
        );
    }

    #[test]
    fn test_GH228_redirect_target_is_not_an_argument() {
        // Only $URL is reported; $OUT is a redirect target (SC2086's business).
        let r = check("curl $URL > $OUT");
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_col, 6);
    }

    #[test]
    fn test_GH228_quoted_header_then_unquoted_url() {
        let r = check(r#"curl -H "Authorization: Bearer $TOKEN" $URL"#);
        assert_eq!(r.diagnostics.len(), 1);
        assert_eq!(r.diagnostics[0].span.start_col, 40);
    }

    #[test]
    fn test_GH228_trailing_comment_is_not_scanned() {
        assert_eq!(check(r#"curl "$a" # curl $b"#).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH228_escaped_dollar_is_not_an_expansion() {
        assert_eq!(check(r"curl \$URL").diagnostics.len(), 0);
    }

    #[test]
    fn test_GH228_arithmetic_expansion_is_not_word_split() {
        assert_eq!(check("curl $(( x + $y ))").diagnostics.len(), 0);
    }

    #[test]
    fn test_GH228_command_substitution_argument_is_not_a_variable() {
        assert_eq!(check("curl $(get_url)").diagnostics.len(), 0);
    }

    #[test]
    fn test_GH228_inner_command_not_dangerous_no_finding() {
        assert_eq!(check(r#"curl "$(echo $url)""#).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH229_command_substring_is_not_a_command() {
        assert_eq!(check("curl_handler $URL").diagnostics.len(), 0);
    }

    // ================================================================
    // malformed input: must not panic, must not hang
    // ================================================================

    #[test]
    fn test_GH228_unterminated_double_quote_does_not_panic() {
        assert_eq!(check(r#"curl "$url"#).diagnostics.len(), 0);
    }

    #[test]
    fn test_GH228_unterminated_single_quote_does_not_panic() {
        let _ = check("curl '");
    }

    #[test]
    fn test_GH228_unterminated_command_substitution_does_not_panic() {
        let r = check("x=$(curl $u");
        assert_eq!(r.diagnostics.len(), 1);
    }

    #[test]
    fn test_GH228_unterminated_brace_expansion_does_not_panic() {
        let _ = check("curl ${URL");
    }

    #[test]
    fn test_GH228_deeply_nested_substitution_terminates() {
        let s = format!("curl {}$u{}", "$(".repeat(64), ")".repeat(64));
        let _ = check(&s);
    }

    // ================================================================
    // Property tests (GH-228 / GH-229)
    // ================================================================

    use proptest::prelude::*;

    proptest! {
        /// Quoting an expansion can only ever remove findings.
        #[test]
        fn prop_GH228_quoting_never_adds_a_finding(
            name in "[A-Za-z_][A-Za-z0-9_]{0,8}",
            cmd  in prop::sample::select(DANGEROUS_COMMANDS),
        ) {
            let bare   = check(&format!("{} ${}", cmd, name)).diagnostics.len();
            let quoted = check(&format!("{} \"${}\"", cmd, name)).diagnostics.len();
            prop_assert!(quoted <= bare);
            prop_assert_eq!(quoted, 0);
        }

        /// A dangerous command name in *command* position after a variable is never
        /// reachable - the GH-229 invariant.
        #[test]
        fn prop_GH229_variable_command_never_flagged(
            var in "[a-z_][a-z0-9_]{0,8}",
            arg in "[a-z ]{1,20}",
        ) {
            prop_assert_eq!(check(&format!("${} '{}'", var, arg)).diagnostics.len(), 0);
        }

        /// Every emitted span is a valid byte range on its own line - the guard
        /// against the apply_single_fix panic.
        #[test]
        fn prop_GH228_spans_are_char_boundaries(src in ".{0,200}") {
            for d in check(&src).diagnostics {
                let line = src.lines().nth(d.span.start_line - 1).unwrap_or("");
                prop_assert!(d.span.start_col >= 1 && d.span.end_col > d.span.start_col);
                prop_assert!(d.span.end_col <= line.len() + 1);
                prop_assert!(line.is_char_boundary(d.span.start_col - 1));
                prop_assert!(line.is_char_boundary(d.span.end_col - 1));
            }
        }

        /// Never panics, never exceeds one finding per line.
        #[test]
        fn prop_GH228_total_and_bounded(src in ".{0,400}") {
            let r = check(&src);
            prop_assert!(r.diagnostics.len() <= src.lines().count());
        }
    }
}
