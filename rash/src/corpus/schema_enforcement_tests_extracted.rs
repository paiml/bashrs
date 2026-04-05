#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat, output: &str) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            name: format!("test-{id}"),
            description: "Test entry".to_string(),
            format,
            tier: CorpusTier::Trivial,
            input: String::new(),
            expected_output: output.to_string(),
            shellcheck: true,
            deterministic: true,
            idempotent: true,
        }
    }

    #[test]
    fn test_grammar_category_code() {
        assert_eq!(GrammarCategory::MissingQuoting.code(), "GRAM-001");
        assert_eq!(GrammarCategory::Bashism.code(), "GRAM-002");
        assert_eq!(GrammarCategory::TabSpaceConfusion.code(), "GRAM-003");
        assert_eq!(GrammarCategory::ShellFormCmd.code(), "GRAM-004");
        assert_eq!(GrammarCategory::UndefinedVariable.code(), "GRAM-005");
        assert_eq!(GrammarCategory::InvalidArithmetic.code(), "GRAM-006");
        assert_eq!(GrammarCategory::MissingFrom.code(), "GRAM-007");
        assert_eq!(GrammarCategory::CircularDependency.code(), "GRAM-008");
    }

    #[test]
    fn test_grammar_category_display() {
        assert_eq!(format!("{}", GrammarCategory::Bashism), "GRAM-002");
    }

    #[test]
    fn test_grammar_category_all() {
        assert_eq!(GrammarCategory::all().len(), 8);
    }

    #[test]
    fn test_grammar_category_applicable_format() {
        assert_eq!(
            GrammarCategory::MissingQuoting.applicable_format(),
            CorpusFormat::Bash
        );
        assert_eq!(
            GrammarCategory::TabSpaceConfusion.applicable_format(),
            CorpusFormat::Makefile
        );
        assert_eq!(
            GrammarCategory::MissingFrom.applicable_format(),
            CorpusFormat::Dockerfile
        );
    }

    #[test]
    fn test_grammar_category_description() {
        assert!(!GrammarCategory::MissingQuoting.description().is_empty());
        assert!(!GrammarCategory::Bashism.fix_pattern().is_empty());
    }

    #[test]
    fn test_validation_layer_display() {
        assert_eq!(format!("{}", ValidationLayer::Lexical), "L1:Lexical");
        assert_eq!(format!("{}", ValidationLayer::Syntactic), "L2:Syntactic");
        assert_eq!(format!("{}", ValidationLayer::Semantic), "L3:Semantic");
        assert_eq!(format!("{}", ValidationLayer::Behavioral), "L4:Behavioral");
    }

    #[test]
    fn test_validate_bash_clean() {
        let entry = make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nset -eu\necho \"hello\"\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
        assert!(result.violations.is_empty());
        assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    }

    #[test]
    fn test_validate_bash_bashism() {
        let entry = make_entry(
            "B-002",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo ok; fi\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].category, GrammarCategory::Bashism);
    }

    #[test]
    fn test_validate_bash_unquoted_expansion() {
        let entry = make_entry("B-003", CorpusFormat::Bash, "#!/bin/sh\necho $HOME\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::MissingQuoting
        );
    }

    #[test]
    fn test_validate_bash_quoted_expansion_ok() {
        let entry = make_entry("B-004", CorpusFormat::Bash, "#!/bin/sh\necho \"$HOME\"\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_bash_assignment_not_flagged() {
        let entry = make_entry("B-005", CorpusFormat::Bash, "#!/bin/sh\nFOO=$HOME\n");
        let result = validate_entry(&entry);
        // Assignments are not flagged for unquoted expansions
        assert!(result.valid);
    }

    #[test]
    fn test_validate_bash_invalid_arithmetic() {
        let entry = make_entry("B-006", CorpusFormat::Bash, "#!/bin/sh\n(( x = x + 1 ))\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::InvalidArithmetic
        );
    }

    #[test]
    fn test_validate_bash_posix_arithmetic_ok() {
        let entry = make_entry("B-007", CorpusFormat::Bash, "#!/bin/sh\nx=$((x + 1))\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_makefile_clean() {
        let entry = make_entry(
            "M-001",
            CorpusFormat::Makefile,
            "CC := gcc\n\nall:\n\t$(CC) -o main main.c\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_makefile_space_recipe() {
        let entry = make_entry("M-002", CorpusFormat::Makefile, "all:\n    echo hello\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::TabSpaceConfusion
        );
    }

    #[test]
    fn test_validate_dockerfile_clean() {
        let entry = make_entry(
            "D-001",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nRUN apk add --no-cache curl\nCMD [\"curl\", \"https://example.com\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_dockerfile_missing_from() {
        let entry = make_entry("D-002", CorpusFormat::Dockerfile, "RUN apt-get update\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::MissingFrom);
    }

    #[test]
    fn test_validate_dockerfile_shell_form_cmd() {
        let entry = make_entry(
            "D-003",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD echo hello\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::ShellFormCmd);
    }

    #[test]
    fn test_validate_dockerfile_exec_form_ok() {
        let entry = make_entry(
            "D-004",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD [\"echo\", \"hello\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_dockerfile_arg_before_from() {
        let entry = make_entry(
            "D-005",
            CorpusFormat::Dockerfile,
            "ARG VERSION=3.18\nFROM alpine:${VERSION}\nRUN echo ok\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_corpus_report() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash, "#!/bin/sh\necho \"ok\"\n"),
            make_entry(
                "B-002",
                CorpusFormat::Bash,
                "#!/bin/sh\nif [[ 1 ]]; then echo ok; fi\n",
            ),
            make_entry("M-001", CorpusFormat::Makefile, "all:\n\techo ok\n"),
            make_entry(
                "D-001",
                CorpusFormat::Dockerfile,
                "FROM alpine:3.18\nRUN echo ok\n",
            ),
        ];
        let registry = CorpusRegistry { entries };

        let report = validate_corpus(&registry);
        assert_eq!(report.total_entries, 4);
        assert_eq!(report.valid_entries, 3);
        assert_eq!(report.total_violations, 1);
    }

    #[test]
    fn test_schema_report_pass_rate() {
        let report = SchemaReport {
            results: vec![],
            total_entries: 10,
            valid_entries: 9,
            total_violations: 1,
            violations_by_category: vec![],
        };
        let rate = report.pass_rate();
        assert!((rate - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_schema_report_pass_rate_empty() {
        let report = SchemaReport {
            results: vec![],
            total_entries: 0,
            valid_entries: 0,
            total_violations: 0,
            violations_by_category: vec![],
        };
        assert!((report.pass_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_format_schema_report() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash, "#!/bin/sh\necho \"ok\"\n"),
            make_entry("M-001", CorpusFormat::Makefile, "all:\n\techo ok\n"),
        ];
        let registry = CorpusRegistry { entries };
        let report = validate_corpus(&registry);
        let table = format_schema_report(&report);
        assert!(table.contains("Bash"));
        assert!(table.contains("Makefile"));
        assert!(table.contains("Total"));
    }

    #[test]
    fn test_format_grammar_errors() {
        let entries = vec![make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ 1 ]]; then echo ok; fi\n",
        )];
        let registry = CorpusRegistry { entries };
        let report = validate_corpus(&registry);
        let table = format_grammar_errors(&report);
        assert!(table.contains("GRAM-001"));
        assert!(table.contains("GRAM-002"));
        assert!(table.contains("B-001"));
    }

    #[test]
    fn test_format_grammar_errors_clean() {
        let entries = vec![make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\necho \"ok\"\n",
        )];
        let registry = CorpusRegistry { entries };
        let report = validate_corpus(&registry);
        let table = format_grammar_errors(&report);
        assert!(table.contains("No grammar violations"));
    }

    #[test]
    fn test_format_grammar_spec_bash() {
        let spec = format_grammar_spec(CorpusFormat::Bash);
        assert!(spec.contains("POSIX Shell Grammar"));
        assert!(spec.contains("complete_command"));
        assert!(spec.contains("L1: Lexical"));
    }

    #[test]
    fn test_format_grammar_spec_makefile() {
        let spec = format_grammar_spec(CorpusFormat::Makefile);
        assert!(spec.contains("GNU Make Grammar"));
        assert!(spec.contains("makefile"));
        assert!(spec.contains("recipe"));
    }

    #[test]
    fn test_format_grammar_spec_dockerfile() {
        let spec = format_grammar_spec(CorpusFormat::Dockerfile);
        assert!(spec.contains("Dockerfile Grammar"));
        assert!(spec.contains("FROM"));
        assert!(spec.contains("exec_form"));
    }

    #[test]
    fn test_check_unquoted_expansion_simple() {
        assert!(check_unquoted_expansion("echo $HOME"));
        assert!(!check_unquoted_expansion("echo \"$HOME\""));
        assert!(!check_unquoted_expansion("FOO=$BAR"));
    }

    #[test]
    fn test_check_unquoted_expansion_single_quote() {
        assert!(!check_unquoted_expansion("echo '$HOME'"));
    }

    #[test]
    fn test_check_unquoted_expansion_arithmetic() {
        assert!(!check_unquoted_expansion("x=$((x + 1))"));
    }

    #[test]
    fn test_check_unquoted_expansion_escaped() {
        assert!(!check_unquoted_expansion("echo \\$HOME"));
    }

    // BH-MUT-0013: is_unquoted_var_at mutation targets
    // Kills mutations of the $( exclusion and ${}/$ _ detection at lines 277-287

    #[test]
    fn test_SCHEMA_MUT_013a_subshell_not_flagged() {
        // $(...) subshell is NOT a bare variable expansion
        assert!(!check_unquoted_expansion("echo $(date)"));
    }

    #[test]
    fn test_SCHEMA_MUT_013b_brace_expansion_flagged() {
        // ${VAR} outside quotes IS an unquoted expansion
        assert!(check_unquoted_expansion("echo ${HOME}"));
    }

    #[test]
    fn test_SCHEMA_MUT_013c_underscore_var_flagged() {
        // $_ outside quotes IS an unquoted expansion
        assert!(check_unquoted_expansion("echo $_"));
    }

    #[test]
    fn test_SCHEMA_MUT_013d_mixed_quotes_var_flagged() {
        // Var between quoted segments is still unquoted
        assert!(check_unquoted_expansion("echo \"hello\" $var 'world'"));
    }

    // BH-MUT-0014: extract_make_var mutation targets
    // Kills mutations of tab/comment filtering at lines 321-322

    #[test]
    fn test_SCHEMA_MUT_014a_extract_make_var_comment() {
        assert!(extract_make_var("# CC := gcc").is_none());
    }

    #[test]
    fn test_SCHEMA_MUT_014b_extract_make_var_tab() {
        assert!(extract_make_var("\t$(CC) -o main main.c").is_none());
    }

    #[test]
    fn test_SCHEMA_MUT_014c_extract_make_var_valid() {
        assert_eq!(extract_make_var("CC := gcc"), Some("CC".to_string()));
    }

    #[test]
    fn test_SCHEMA_MUT_014d_extract_make_var_invalid_name() {
        // Variable name with spaces should not match
        assert!(extract_make_var("bad name := value").is_none());
    }

    #[test]
    fn test_multiple_violations_same_entry() {
        let entry = make_entry(
            "B-010",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo $var; fi\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        // Should have both bashism and unquoted expansion
        assert!(result.violations.len() >= 2);
        let categories: Vec<GrammarCategory> =
            result.violations.iter().map(|v| v.category).collect();
        assert!(categories.contains(&GrammarCategory::Bashism));
        assert!(categories.contains(&GrammarCategory::MissingQuoting));
    }

    #[test]
    fn test_empty_output_fails_lexical() {
        let entry = make_entry("B-099", CorpusFormat::Bash, "");
        let result = validate_entry(&entry);
        assert!(!result.layers_passed.contains(&ValidationLayer::Lexical));
    }

    #[test]
    fn test_entrypoint_shell_form_violation() {
        let entry = make_entry(
            "D-010",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nENTRYPOINT /bin/sh\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::ShellFormCmd);
    }

    // BH-MUT-0009: is_space_indented_recipe mutation targets
    // Kills mutations of the 4-part AND + OR in lines 337-340

    #[test]
    fn test_SCHEMA_MUT_009a_space_recipe_requires_in_recipe() {
        // Space-indented line NOT inside a recipe context → should NOT flag
        let entry = make_entry(
            "M-MUT-009a",
            CorpusFormat::Makefile,
            "CC := gcc\n    echo hello\n",
        );
        let result = validate_entry(&entry);
        // No GRAM-003 because there's no preceding target rule
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    #[test]
    fn test_SCHEMA_MUT_009b_tab_recipe_not_flagged() {
        // Tab-indented recipe line → should NOT flag (correct indentation)
        let entry = make_entry("M-MUT-009b", CorpusFormat::Makefile, "all:\n\techo hello\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_SCHEMA_MUT_009c_two_space_recipe_flagged() {
        // Two-space indented recipe → should flag GRAM-003
        let entry = make_entry("M-MUT-009c", CorpusFormat::Makefile, "all:\n  echo hello\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    #[test]
    fn test_SCHEMA_MUT_009d_empty_line_resets_recipe() {
        // Empty line between target and space-indented line → NOT in recipe context
        let entry = make_entry(
            "M-MUT-009d",
            CorpusFormat::Makefile,
            "all:\n\n    echo hello\n",
        );
        let result = validate_entry(&entry);
        // Empty line resets in_recipe, so the space line is not flagged
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    // BH-MUT-0010: Dockerfile ENTRYPOINT exec form
    // Kills mutation of || to && and negation of contains('[')

    #[test]
    fn test_SCHEMA_MUT_010a_entrypoint_exec_form_ok() {
        // ENTRYPOINT with exec form → should NOT flag
        let entry = make_entry(
            "D-MUT-010a",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nENTRYPOINT [\"sh\", \"-c\", \"echo hello\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_SCHEMA_MUT_010b_cmd_and_entrypoint_shell_form() {
        // Both CMD and ENTRYPOINT in shell form → should flag both
        let entry = make_entry(
            "D-MUT-010b",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD echo hello\nENTRYPOINT /bin/sh\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        let shell_form_count = result
            .violations
            .iter()
            .filter(|v| v.category == GrammarCategory::ShellFormCmd)
            .count();
        assert_eq!(shell_form_count, 2);
    }

    // BH-MUT-0011: Makefile := assignment vs target rule distinction
    // Kills mutation of !line.contains(":=") at line 367

    #[test]
    fn test_SCHEMA_MUT_011a_assignment_not_target() {
        // := assignment should NOT set in_recipe, so next space line is not flagged
        let entry = make_entry(
            "M-MUT-011a",
            CorpusFormat::Makefile,
            "CC := gcc\n    echo hello\n",
        );
        let result = validate_entry(&entry);
        // No tab/space confusion because CC := gcc is assignment, not target
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    #[test]
    fn test_SCHEMA_MUT_011b_target_then_space_recipe() {
        // Real target rule followed by space-indented recipe → SHOULD flag
        let entry = make_entry(
            "M-MUT-011b",
            CorpusFormat::Makefile,
            "build:\n    gcc -o main main.c\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    // BH-MUT-0012: Bash arithmetic (( vs $(( coexistence
    // Kills mutation of && to || and negation removal at line 228

    #[test]
    fn test_SCHEMA_MUT_012a_posix_arithmetic_not_flagged() {
        // $(( )) is POSIX arithmetic → should NOT flag
        let entry = make_entry(
            "B-MUT-012a",
            CorpusFormat::Bash,
            "#!/bin/sh\nx=$(( x + 1 ))\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_SCHEMA_MUT_012b_bash_arithmetic_flagged() {
        // (( )) without $( prefix → SHOULD flag
        let entry = make_entry(
            "B-MUT-012b",
            CorpusFormat::Bash,
            "#!/bin/sh\n(( x = x + 1 ))\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::InvalidArithmetic));
    }

    #[test]
    fn test_SCHEMA_MUT_012c_mixed_arithmetic_not_flagged() {
        // Line has both (( and $(( — the $(( takes precedence, NOT a bashism
        let entry = make_entry(
            "B-MUT-012c",
            CorpusFormat::Bash,
            "#!/bin/sh\necho \"result: $(( 1 + 2 ))\"\n",
        );
        let result = validate_entry(&entry);
        // $(( is valid POSIX arithmetic expansion, should not flag
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::InvalidArithmetic));
    }
}
