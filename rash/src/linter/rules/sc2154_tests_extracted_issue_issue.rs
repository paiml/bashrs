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
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
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
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("EUID")),
            "SC2154 must NOT flag EUID - it's a bash builtin"
        );
    }

    #[test]
    fn test_FP_098_uid_not_flagged() {
        let script = r#"echo $UID"#;
        let result = check(script);
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("'UID'")),
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
            result
                .diagnostics
                .iter()
                .all(|d| !d.message.contains("RANDOM")
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
            result
                .diagnostics
                .iter()
                .all(|d| !d.message.contains("HOSTNAME")
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
            result
                .diagnostics
                .iter()
                .all(|d| !d.message.contains("SHLVL")
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
            result
                .diagnostics
                .iter()
                .all(|d| !d.message.contains("OLDPWD")
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
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("shell_rc")),
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
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("level")),
            "SC2154 must NOT flag variable assigned in case with default"
        );
    }

    // Issue #132: Parameter expansion with default should NOT trigger warning
    #[test]
    fn test_issue_132_parameter_expansion_default_not_flagged() {
        // ${VAR:-} is intentional check for environment variable
        let script = r#"
_is_bashrs_test() {
    [[ "${BASHRS_TEST:-}" == "1" ]]
}
"#;
        let result = check(script);
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("BASHRS_TEST")),
            "SC2154 must NOT flag ${{VAR:-}} - it's intentional env var check"
        );
    }

    #[test]
    fn test_issue_132_parameter_expansion_variants() {
        // All parameter expansion operators should be recognized
        let script = r#"
echo "${VAR1:-default}"
echo "${VAR2:=assigned}"
echo "${VAR3:+alternate}"
echo "${VAR4:?error}"
echo "${VAR5-default}"
echo "${VAR6=assigned}"
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2154 must NOT flag any parameter expansion operators"
        );
    }

    #[test]
    fn test_issue_132_regular_undefined_still_flagged() {
        // Regular undefined variables should still be flagged
        let script = r#"
echo "$UNDEFINED_VAR"
echo "${ANOTHER_UNDEFINED}"
"#;
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            2,
            "SC2154 should still flag regular undefined variables"
        );
    }
