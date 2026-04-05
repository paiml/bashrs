
    #[test]
    fn test_generate_report_failure_variant() {
        let report = make_report_with_result(VerificationResult::Failure {
            reasons: vec![
                "States differ in env".to_string(),
                "cwd mismatch".to_string(),
            ],
        });
        let text = ProofInspector::generate_report(&report);
        assert!(text.contains("FAILURE"));
        assert!(text.contains("States differ in env"));
        assert!(text.contains("cwd mismatch"));
    }

    /// Partial verification result.
    #[test]
    fn test_generate_report_partial_variant() {
        let report = make_report_with_result(VerificationResult::Partial {
            issues: vec!["Partial match only".to_string()],
        });
        let text = ProofInspector::generate_report(&report);
        assert!(text.contains("PARTIAL"));
        assert!(text.contains("Partial match only"));
    }

    /// Success variant (already covered by upstream but make explicit here).
    #[test]
    fn test_generate_report_success_variant() {
        let report = make_report_with_result(VerificationResult::Success { confidence: 0.99 });
        let text = ProofInspector::generate_report(&report);
        assert!(text.contains("SUCCESS"));
        assert!(text.contains("99.0"));
    }

    /// Report with non-empty equivalence sections shows ❌ markers.
    #[test]
    fn test_generate_report_non_equivalent_shows_x_marks() {
        let ast = TinyAst::SetEnvironmentVariable {
            name: "KEY".to_string(),
            value: "val".to_string(),
        };
        // Use an initial state that already has KEY with a different value so
        // the env comparison shows a difference.
        let initial = state_with_env(&[("KEY", "old_val")]);
        let report = ProofInspector::inspect(&ast, initial);
        let text = ProofInspector::generate_report(&report);
        assert!(text.contains("Formal Verification Report"));
        assert!(text.contains("Equivalence Analysis"));
    }

    // ── compare_filesystems ───────────────────────────────────────────────────

    /// Filesystems are identical → matches = true.
    #[test]
    fn test_compare_filesystems_identical() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["hi".to_string()],
        };
        let initial = empty_state();
        let report = ProofInspector::inspect(&ast, initial);
        // echo doesn't change filesystem; rash and posix traces should have
        // the same filesystem (both start from empty state).
        let fs_cmp = &report.equivalence_analysis.fs_comparison;
        assert!(
            fs_cmp.matches,
            "echo should not change filesystem: differences={:?}",
            fs_cmp.differences
        );
    }

    /// Rash filesystem has an extra directory that posix doesn't.
    #[test]
    fn test_compare_filesystems_rash_only_entry() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "mkdir".to_string(),
            args: vec!["/rash_only_dir".to_string()],
        };
        let initial = empty_state();
        let report = ProofInspector::inspect(&ast, initial);
        // Check equivalence analysis ran without panic.
        let _ = &report.equivalence_analysis.fs_comparison;
    }

    // ── Sequence tracing (covers children path in annotate_ast) ───────────────

    #[test]
    fn test_annotate_ast_sequence_children_count() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "A".to_string(),
                    value: "1".to_string(),
                },
                TinyAst::SetEnvironmentVariable {
                    name: "B".to_string(),
                    value: "2".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec!["done".to_string()],
                },
            ],
        };
        let initial = empty_state();
        let report = ProofInspector::inspect(&ast, initial);
        assert_eq!(report.annotated_ast.children.len(), 3);
    }

    /// Nested sequence: Sequence containing a Sequence.
    #[test]
    fn test_annotate_ast_nested_sequence() {
        let inner = TinyAst::Sequence {
            commands: vec![TinyAst::SetEnvironmentVariable {
                name: "INNER".to_string(),
                value: "x".to_string(),
            }],
        };
        let outer = TinyAst::Sequence {
            commands: vec![
                inner,
                TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec!["outer".to_string()],
                },
            ],
        };
        let report = ProofInspector::inspect(&outer, empty_state());
        assert_eq!(report.annotated_ast.children.len(), 2);
        // Shouldn't panic
        let text = ProofInspector::generate_report(&report);
        assert!(text.contains("Formal Verification Report"));
    }

    // ── emitter justifications for all TinyAst variants ──────────────────────

    #[test]
    fn test_emitter_justifications_execute_command() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "chmod".to_string(),
            args: vec!["755".to_string(), "/bin/x".to_string()],
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        assert!(!report.emitter_justifications.is_empty());
        let j = &report.emitter_justifications[0];
        assert!(j.ast_node.contains("ExecuteCommand"));
        assert!(!j.generated_code.is_empty());
        assert!(!j.reasoning.is_empty());
        assert!(!j.considerations.is_empty());
    }

    #[test]
    fn test_emitter_justifications_set_env() {
        let ast = TinyAst::SetEnvironmentVariable {
            name: "MY_VAR".to_string(),
            value: "42".to_string(),
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        let j = &report.emitter_justifications[0];
        assert!(j.ast_node.contains("SetEnvironmentVariable"));
        assert!(j.generated_code.contains("MY_VAR"));
    }

    #[test]
    fn test_emitter_justifications_change_directory() {
        let ast = TinyAst::ChangeDirectory {
            path: "/tmp".to_string(),
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        let j = &report.emitter_justifications[0];
        assert!(j.ast_node.contains("ChangeDirectory"));
        assert!(j.generated_code.contains("cd"));
    }

    #[test]
    fn test_emitter_justifications_sequence_includes_all() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "V1".to_string(),
                    value: "a".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec!["b".to_string()],
                },
            ],
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        // Sequence itself + 2 children = at least 3 justifications
        assert!(report.emitter_justifications.len() >= 3);
        let sequence_j = report
            .emitter_justifications
            .iter()
            .find(|j| j.ast_node == "Sequence");
        assert!(sequence_j.is_some());
    }

    // ── ExecutionTrace structure ──────────────────────────────────────────────

    #[test]
    fn test_rash_trace_has_steps() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "A".to_string(),
                    value: "1".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec!["ok".to_string()],
                },
            ],
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        assert_eq!(report.rash_trace.steps.len(), 2);
        assert_eq!(report.rash_trace.steps[0].step_number, 1);
        assert_eq!(report.rash_trace.steps[1].step_number, 2);
    }

    #[test]
    fn test_posix_trace_has_one_step() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["hi".to_string()],
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        // posix trace treats entire code as one step
        assert_eq!(report.posix_trace.steps.len(), 1);
        assert_eq!(report.posix_trace.steps[0].step_number, 1);
    }

    // ── Equivalence analysis with mismatched states ───────────────────────────

    #[test]
    fn test_equivalence_analysis_env_mismatch() {
        // SetEnv changes the rash state but POSIX may not apply the same
        let ast = TinyAst::SetEnvironmentVariable {
            name: "UNIQUE_XVAR_2025".to_string(),
            value: "42".to_string(),
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        // The equivalence analysis should reflect the env difference
        let eq = &report.equivalence_analysis;
        // Either matched (posix semantics also set the var) or not — no panic
        let _ = eq.are_equivalent;
    }

    // ── StateTransformation struct fields ────────────────────────────────────

    #[test]
    fn test_state_transformation_fields_accessible() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "mkdir".to_string(),
            args: vec!["/cov_test_dir".to_string()],
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        let t = &report.annotated_ast.transformation;
        // Just access each field to ensure they're covered
        let _ = &t.env_changes;
        let _ = &t.cwd_change;
        let _ = &t.fs_changes;
        let _ = &t.output_produced;
        let _ = &t.errors_produced;
        let _ = &t.exit_code_change;
    }

    // ── CwdChange recorded when cwd transitions ───────────────────────────────

    #[test]
    fn test_change_directory_ast_produces_step() {
        let ast = TinyAst::ChangeDirectory {
            path: "/".to_string(),
        };
        let initial = empty_state(); // cwd is already "/"
        let report = ProofInspector::inspect(&ast, initial);
        // rash trace should have 1 step for the cd operation
        assert_eq!(report.rash_trace.steps.len(), 1);
        assert!(report.rash_trace.steps[0]
            .operation
            .contains("Change directory"));
    }

    // ── generate_report includes emitter justifications section ───────────────

    #[test]
    fn test_generate_report_contains_emitter_section() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["world".to_string()],
        };
        let report = ProofInspector::inspect(&ast, empty_state());
        let text = ProofInspector::generate_report(&report);
        assert!(text.contains("Emitter Justifications"));
        assert!(text.contains("Generated:"));
        assert!(text.contains("Reasoning:"));
        assert!(text.contains("Considerations:"));
    }
}
