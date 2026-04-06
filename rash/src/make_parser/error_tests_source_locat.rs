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
}

#[cfg(test)]
mod error_tests_extracted_detailed {
    use super::*;
    include!("error_tests_extracted_detailed.rs");
}
