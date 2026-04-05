#[cfg(test)]
mod tests {
    use super::*;

    // ====== SourceLocation Tests ======

    #[test]
    fn test_source_location_new() {
        let loc = SourceLocation::new(42);
        assert_eq!(loc.line, 42);
        assert!(loc.file.is_none());
        assert!(loc.column.is_none());
        assert!(loc.source_line.is_none());
    }

    #[test]
    fn test_source_location_with_file() {
        let loc = SourceLocation::new(10).with_file("Makefile".to_string());
        assert_eq!(loc.file, Some("Makefile".to_string()));
        assert_eq!(loc.line, 10);
    }

    #[test]
    fn test_source_location_with_column() {
        let loc = SourceLocation::new(5).with_column(15);
        assert_eq!(loc.column, Some(15));
    }

    #[test]
    fn test_source_location_with_source_line() {
        let loc = SourceLocation::new(1).with_source_line("CC := gcc".to_string());
        assert_eq!(loc.source_line, Some("CC := gcc".to_string()));
    }

    #[test]
    fn test_source_location_chained_builder() {
        let loc = SourceLocation::new(42)
            .with_file("test.mk".to_string())
            .with_column(8)
            .with_source_line("ifeq ($(X),Y)".to_string());

        assert_eq!(loc.line, 42);
        assert_eq!(loc.file, Some("test.mk".to_string()));
        assert_eq!(loc.column, Some(8));
        assert_eq!(loc.source_line, Some("ifeq ($(X),Y)".to_string()));
    }

    #[test]
    fn test_source_location_display_no_file() {
        let loc = SourceLocation::new(15);
        let display = format!("{}", loc);
        assert_eq!(display, "line 15");
    }

    #[test]
    fn test_source_location_display_with_file() {
        let loc = SourceLocation::new(15).with_file("Makefile".to_string());
        let display = format!("{}", loc);
        assert_eq!(display, "Makefile:15");
    }

    #[test]
    fn test_source_location_display_with_column() {
        let loc = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8);
        let display = format!("{}", loc);
        assert_eq!(display, "Makefile:15:8");
    }

    #[test]
    fn test_source_location_display_no_file_with_column() {
        let loc = SourceLocation::new(15).with_column(8);
        let display = format!("{}", loc);
        assert_eq!(display, "line 15:8");
    }

    #[test]
    fn test_source_location_equality() {
        let loc1 = SourceLocation::new(10).with_file("a.mk".to_string());
        let loc2 = SourceLocation::new(10).with_file("a.mk".to_string());
        let loc3 = SourceLocation::new(20).with_file("a.mk".to_string());

        assert_eq!(loc1, loc2);
        assert_ne!(loc1, loc3);
    }

    // ====== Quality Score Tests ======

    #[test]
    fn test_quality_score_minimum() {
        // UnexpectedEof has no location, so minimal score
        let error = MakeParseError::UnexpectedEof;
        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) = 6.0 / 8.5 = 0.706
        assert!(score >= 0.7, "Score {} should be ≥0.7", score);
        assert!(score < 0.75, "Score {} should be <0.75", score);
    }

    #[test]
    fn test_quality_score_with_location() {
        let location = SourceLocation::new(15);
        let error = MakeParseError::EmptyVariableName { location };
        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) + line(0.25) = 6.25 / 8.5 = 0.735
        assert!(score >= 0.73, "Score {} should be ≥0.73", score);
        assert!(score < 0.75, "Score {} should be <0.75", score);
    }

    #[test]
    fn test_quality_score_with_file_and_column() {
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8);

        let error = MakeParseError::EmptyTargetName { location };
        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) + file(1.0) + line(0.25) + column(0.25) = 7.5 / 8.5 = 0.882
        assert!(score >= 0.88, "Score {} should be ≥0.88", score);
        assert!(score < 0.89, "Score {} should be <0.89", score);
    }

    #[test]
    fn test_quality_score_with_snippet() {
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8)
            .with_source_line("ifeq $(VAR) value".to_string());

        let error = MakeParseError::InvalidConditionalSyntax {
            location,
            directive: "ifeq".to_string(),
            found: "$(VAR) value".to_string(),
        };

        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) + file(1.0) + line(0.25) + column(0.25) + snippet(1.0) = 8.5 / 8.5 = 1.0
        assert_eq!(score, 1.0, "Score should be perfect 1.0");
    }

    #[test]
    fn test_quality_score_target_exceeds_08() {
        // Target: All errors with full context should achieve ≥0.8 quality score
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8)
            .with_source_line("ifeq $(VAR) value".to_string());

        let error = MakeParseError::InvalidConditionalSyntax {
            location,
            directive: "ifeq".to_string(),
            found: "$(VAR) value".to_string(),
        };

        assert!(
            error.quality_score() >= 0.8,
            "Error quality score {} must be ≥0.8",
            error.quality_score()
        );
    }

    #[test]
    fn test_note_present_for_all_errors() {
        // All error types should have explanatory notes
        let errors = vec![
            MakeParseError::EmptyVariableName {
                location: SourceLocation::new(1),
            },
            MakeParseError::InvalidConditionalSyntax {
                location: SourceLocation::new(1),
                directive: "ifeq".to_string(),
                found: "bad".to_string(),
            },
            MakeParseError::UnexpectedEof,
        ];

        for error in errors {
            let note = error.note();
            assert!(!note.is_empty(), "Note should not be empty for {:?}", error);
            assert!(
                note.len() > 10,
                "Note should be descriptive for {:?}",
                error
            );
        }
    }

    #[test]
    fn test_help_present_for_all_errors() {
        // All error types should have recovery hints
        let errors = vec![
            MakeParseError::EmptyTargetName {
                location: SourceLocation::new(1),
            },
            MakeParseError::InvalidIncludeSyntax {
                location: SourceLocation::new(1),
                found: "bad include".to_string(),
            },
            MakeParseError::UnexpectedEof,
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help should not be empty for {:?}", error);
            assert!(help.len() > 10, "Help should be actionable for {:?}", error);
        }
    }

    #[test]
    fn test_detailed_string_format() {
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8)
            .with_source_line("ifeq $(VAR) value".to_string());

        let error = MakeParseError::InvalidConditionalSyntax {
            location,
            directive: "ifeq".to_string(),
            found: "$(VAR) value".to_string(),
        };

        let detailed = error.to_detailed_string();

        // Should contain all components
        assert!(detailed.contains("error:"));
        assert!(detailed.contains("15 | ifeq $(VAR) value"));
        assert!(detailed.contains("note:"));
        assert!(detailed.contains("help:"));
        assert!(detailed.contains("^")); // Caret indicator
    }

    // ====== Error Variant Tests ======

    #[test]
    fn test_invalid_variable_assignment_note_and_help() {
        let error = MakeParseError::InvalidVariableAssignment {
            location: SourceLocation::new(1),
            found: "bad assignment".to_string(),
        };
        let note = error.note();
        let help = error.help();
        assert!(note.contains(":=") || note.contains("="));
        assert!(help.contains("VAR =") || help.contains(":="));
    }

    #[test]
    fn test_empty_variable_name_note_and_help() {
        let error = MakeParseError::EmptyVariableName {
            location: SourceLocation::new(1),
        };
        assert!(error.note().contains("cannot be empty"));
        assert!(error.help().contains("variable name"));
    }

    #[test]
    fn test_no_assignment_operator_note_and_help() {
        let error = MakeParseError::NoAssignmentOperator {
            location: SourceLocation::new(1),
            found: "VAR value".to_string(),
        };
        assert!(error.note().contains("operator"));
        assert!(error.help().contains("="));
    }

    #[test]
    fn test_invalid_include_syntax_note_and_help() {
        let error = MakeParseError::InvalidIncludeSyntax {
            location: SourceLocation::new(1),
            found: "bad include".to_string(),
        };
        assert!(error.note().contains("include"));
        assert!(error.help().contains("include"));
    }

    #[test]
    fn test_invalid_conditional_syntax_note_ifeq() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifeq".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.note().contains("ifeq"));
    }

    #[test]
    fn test_invalid_conditional_syntax_note_ifneq() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifneq".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.note().contains("ifneq"));
    }

    #[test]
    fn test_invalid_conditional_syntax_note_ifdef() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifdef".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.note().contains("ifdef"));
    }

    #[test]
    fn test_invalid_conditional_syntax_note_ifndef() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifndef".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.note().contains("ifndef"));
    }

    #[test]
    fn test_invalid_conditional_syntax_note_unknown() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "unknown".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.note().contains("GNU Make"));
    }

    #[test]
    fn test_invalid_conditional_syntax_help_ifeq() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifeq".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.help().contains("ifeq"));
    }

    #[test]
    fn test_invalid_conditional_syntax_help_ifneq() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifneq".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.help().contains("ifneq"));
    }

    #[test]
    fn test_invalid_conditional_syntax_help_ifdef() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifdef".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.help().contains("ifdef"));
    }

    #[test]
    fn test_invalid_conditional_syntax_help_ifndef() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "ifndef".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.help().contains("ifndef"));
    }

    #[test]
    fn test_invalid_conditional_syntax_help_unknown() {
        let error = MakeParseError::InvalidConditionalSyntax {
            location: SourceLocation::new(1),
            directive: "unknown".to_string(),
            found: "bad".to_string(),
        };
        assert!(error.help().contains("GNU Make"));
    }

    #[test]
    fn test_missing_conditional_arguments_ifeq() {
        let error = MakeParseError::MissingConditionalArguments {
            location: SourceLocation::new(1),
            directive: "ifeq".to_string(),
            expected_args: 2,
            found_args: 0,
        };
        assert!(error.note().contains("2"));
        assert!(error.note().contains("0"));
        assert!(error.help().contains("ifeq"));
    }

    #[test]
    fn test_missing_conditional_arguments_ifdef() {
        let error = MakeParseError::MissingConditionalArguments {
            location: SourceLocation::new(1),
            directive: "ifdef".to_string(),
            expected_args: 1,
            found_args: 0,
        };
        assert!(error.help().contains("ifdef"));
    }

    #[test]
    fn test_missing_conditional_arguments_unknown() {
        let error = MakeParseError::MissingConditionalArguments {
            location: SourceLocation::new(1),
            directive: "unknown".to_string(),
            expected_args: 1,
            found_args: 0,
        };
        assert!(error.help().contains("arguments"));
    }

    #[test]
    fn test_missing_variable_name_note_and_help() {
        let error = MakeParseError::MissingVariableName {
            location: SourceLocation::new(1),
            directive: "ifdef".to_string(),
        };
        assert!(error.note().contains("ifdef"));
        assert!(error.help().contains("ifdef"));
    }

    #[test]
    fn test_unknown_conditional_note_and_help() {
        let error = MakeParseError::UnknownConditional {
            location: SourceLocation::new(1),
            found: "ifequ".to_string(),
        };
        assert!(error.note().contains("Supported"));
        assert!(error.help().contains("ifequ"));
    }

    #[test]
    fn test_invalid_target_rule_note_and_help() {
        let error = MakeParseError::InvalidTargetRule {
            location: SourceLocation::new(1),
            found: "bad rule".to_string(),
        };
        assert!(error.note().contains("target:"));
        assert!(error.help().contains("target:"));
    }

    #[test]
    fn test_empty_target_name_note_and_help() {
        let error = MakeParseError::EmptyTargetName {
            location: SourceLocation::new(1),
        };
        assert!(error.note().contains("cannot be empty"));
        assert!(error.help().contains("target name"));
    }

    #[test]
    fn test_unterminated_define_note_and_help() {
        let error = MakeParseError::UnterminatedDefine {
            location: SourceLocation::new(1),
            var_name: "MY_VAR".to_string(),
        };
        assert!(error.note().contains("endef"));
        assert!(error.help().contains("endef"));
    }

    #[test]
    fn test_unexpected_eof_note_and_help() {
        let error = MakeParseError::UnexpectedEof;
        assert!(error.note().contains("ended unexpectedly"));
        assert!(error.help().contains("endif"));
    }

    #[test]
    fn test_unexpected_eof_has_no_location() {
        let error = MakeParseError::UnexpectedEof;
        assert!(error.location().is_none());
    }

    // ====== Error Location Tests ======

    #[test]
    fn test_all_errors_have_location_except_eof() {
        let errors = vec![
            MakeParseError::InvalidVariableAssignment {
                location: SourceLocation::new(1),
                found: "x".to_string(),
            },
            MakeParseError::EmptyVariableName {
                location: SourceLocation::new(2),
            },
            MakeParseError::NoAssignmentOperator {
                location: SourceLocation::new(3),
                found: "x".to_string(),
            },
            MakeParseError::InvalidIncludeSyntax {
                location: SourceLocation::new(4),
                found: "x".to_string(),
            },
            MakeParseError::InvalidConditionalSyntax {
                location: SourceLocation::new(5),
                directive: "ifeq".to_string(),
                found: "x".to_string(),
            },
            MakeParseError::MissingConditionalArguments {
                location: SourceLocation::new(6),
                directive: "ifeq".to_string(),
                expected_args: 2,
                found_args: 0,
            },
            MakeParseError::MissingVariableName {
                location: SourceLocation::new(7),
                directive: "ifdef".to_string(),
            },
            MakeParseError::UnknownConditional {
                location: SourceLocation::new(8),
                found: "x".to_string(),
            },
            MakeParseError::InvalidTargetRule {
                location: SourceLocation::new(9),
                found: "x".to_string(),
            },
            MakeParseError::EmptyTargetName {
                location: SourceLocation::new(10),
            },
            MakeParseError::UnterminatedDefine {
                location: SourceLocation::new(11),
                var_name: "X".to_string(),
            },
        ];

        for error in errors {
            assert!(
                error.location().is_some(),
                "Error {:?} should have location",
                error
            );
        }
    }

    // ====== Detailed String Tests ======

    #[test]
    fn test_detailed_string_no_source_line() {
        let error = MakeParseError::EmptyVariableName {
            location: SourceLocation::new(5),
        };
        let detailed = error.to_detailed_string();
        assert!(detailed.contains("error:"));
        assert!(detailed.contains("note:"));
        assert!(detailed.contains("help:"));
        assert!(!detailed.contains("|")); // No source snippet
    }

    #[test]
    fn test_detailed_string_with_source_line_no_column() {
        let location = SourceLocation::new(15).with_source_line("MY_VAR value".to_string());
        let error = MakeParseError::NoAssignmentOperator {
            location,
            found: "MY_VAR value".to_string(),
        };
        let detailed = error.to_detailed_string();
        assert!(detailed.contains("15 | MY_VAR value"));
        assert!(!detailed.contains("^")); // No caret without column
    }

    // ====== Error Display Tests ======

    #[test]
    fn test_error_display_messages() {
        let errors_and_expected: Vec<(MakeParseError, &str)> = vec![
            (
                MakeParseError::InvalidVariableAssignment {
                    location: SourceLocation::new(1),
                    found: "bad".to_string(),
                },
                "Invalid variable assignment",
            ),
            (
                MakeParseError::EmptyVariableName {
                    location: SourceLocation::new(1),
                },
                "Empty variable name",
            ),
            (
                MakeParseError::NoAssignmentOperator {
                    location: SourceLocation::new(1),
                    found: "x".to_string(),
                },
                "No assignment operator",
            ),
            (
                MakeParseError::InvalidIncludeSyntax {
                    location: SourceLocation::new(1),
                    found: "x".to_string(),
                },
                "Invalid include syntax",
            ),
            (
                MakeParseError::InvalidConditionalSyntax {
                    location: SourceLocation::new(1),
                    directive: "ifeq".to_string(),
                    found: "x".to_string(),
                },
                "Invalid conditional syntax",
            ),
            (
                MakeParseError::MissingConditionalArguments {
                    location: SourceLocation::new(1),
                    directive: "ifeq".to_string(),
                    expected_args: 2,
                    found_args: 0,
                },
                "Conditional requires arguments",
            ),
            (
                MakeParseError::MissingVariableName {
                    location: SourceLocation::new(1),
                    directive: "ifdef".to_string(),
                },
                "Missing variable name",
            ),
            (
                MakeParseError::UnknownConditional {
                    location: SourceLocation::new(1),
                    found: "x".to_string(),
                },
                "Unknown conditional",
            ),
            (
                MakeParseError::InvalidTargetRule {
                    location: SourceLocation::new(1),
                    found: "x".to_string(),
                },
                "Invalid target rule",
            ),
            (
                MakeParseError::EmptyTargetName {
                    location: SourceLocation::new(1),
                },
                "Empty target name",
            ),
            (
                MakeParseError::UnterminatedDefine {
                    location: SourceLocation::new(1),
                    var_name: "X".to_string(),
                },
                "Unterminated define",
            ),
            (MakeParseError::UnexpectedEof, "Unexpected end of file"),
        ];

        for (error, expected_substring) in errors_and_expected {
            let display = format!("{}", error);
            assert!(
                display.contains(expected_substring),
                "Error display '{}' should contain '{}'",
                display,
                expected_substring
            );
        }
    }

    // ====== Debug Tests ======

    #[test]
    fn test_error_debug() {
        let error = MakeParseError::UnexpectedEof;
        let debug = format!("{:?}", error);
        assert!(debug.contains("UnexpectedEof"));
    }

    #[test]
    fn test_source_location_debug() {
        let loc = SourceLocation::new(42);
        let debug = format!("{:?}", loc);
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_source_location_clone() {
        let loc = SourceLocation::new(42)
            .with_file("test.mk".to_string())
            .with_column(8);
        let cloned = loc.clone();
        assert_eq!(loc, cloned);
    }
}
