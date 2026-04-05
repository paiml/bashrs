#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Helper to format a diagnostic with colors disabled
    fn format_no_color(diag: &Diagnostic) -> String {
        // SAFETY: Only called from serial tests
        unsafe { std::env::set_var("NO_COLOR", "1") };
        let result = format!("{diag}");
        unsafe { std::env::remove_var("NO_COLOR") };
        result
    }

    #[test]
    fn test_diagnostic_quality_score() {
        let mut diag = Diagnostic {
            error: "test error".to_string(),
            file: None,
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        };

        // Baseline: just error prefix
        assert!(diag.quality_score() < 0.7); // Only error prefix, no context

        // Add location
        diag.file = Some("test.rs".to_string());
        diag.line = Some(10);
        diag.column = Some(5);
        assert!(diag.quality_score() < 0.7); // Missing note+help, below threshold

        // Add note and help (target ≥0.7)
        diag.note = Some("Explanation".to_string());
        diag.help = Some("Suggestion".to_string());
        assert!(diag.quality_score() >= 0.7); // Should exceed 0.7 threshold
    }

    #[test]
    fn test_unsupported_feature_diagnostic() {
        let error = Error::Validation("Only functions are allowed in Rash code".to_string());
        let diag = Diagnostic::from_error(&error, Some("example.rs".to_string()));

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
        assert!(diag.note.is_some());
        assert!(diag.help.is_some());

        // Should achieve ≥0.7 quality score
        assert!(
            diag.quality_score() >= 0.7,
            "Quality score {} should be ≥0.7",
            diag.quality_score()
        );
    }

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic {
            error: "unexpected token".to_string(),
            file: Some("main.rs".to_string()),
            line: Some(5),
            column: Some(10),
            category: ErrorCategory::Syntax,
            note: Some("Expected a semicolon here".to_string()),
            help: Some("Add ';' after the statement".to_string()),
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("error[syntax]"), "got: {output}");
        assert!(output.contains("--> main.rs:5:10"), "got: {output}");
        assert!(
            output.contains("note: Expected a semicolon"),
            "got: {output}"
        );
        assert!(output.contains("help: Add ';'"), "got: {output}");
    }

    // ====== Additional Tests for Coverage ======

    #[test]
    fn test_diagnostic_display_no_file() {
        let diag = Diagnostic {
            error: "parse error".to_string(),
            file: None,
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(
            output.contains("error[syntax]: parse error"),
            "got: {output}"
        );
        assert!(!output.contains("-->"), "got: {output}");
    }

    #[test]
    fn test_diagnostic_display_file_only() {
        let diag = Diagnostic {
            error: "file error".to_string(),
            file: Some("test.rs".to_string()),
            line: None,
            column: None,
            category: ErrorCategory::Io,
            note: None,
            help: None,
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("--> test.rs"), "got: {output}");
        assert!(!output.contains(":0"), "No line number: {output}");
    }

    #[test]
    fn test_diagnostic_display_file_and_line() {
        let diag = Diagnostic {
            error: "line error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(42),
            column: None,
            category: ErrorCategory::Validation,
            note: None,
            help: None,
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("--> test.rs:42"), "got: {output}");
    }

    #[test]
    fn test_diagnostic_display_with_snippet() {
        let diag = Diagnostic {
            error: "syntax error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(5),
            column: Some(10),
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: Some(" 5 | let x = foo(\n   |           ^\n".to_string()),
        };

        let output = format_no_color(&diag);
        assert!(output.contains("let x = foo("), "got: {output}");
        assert!(output.contains("^"), "got: {output}");
    }

    #[test]
    fn test_diagnostic_display_snippet_column_0() {
        let diag = Diagnostic {
            error: "syntax error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(5),
            column: Some(0),
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: Some(" 5 | bad code\n   | ^\n".to_string()),
        };

        let output = format_no_color(&diag);
        assert!(output.contains("bad code"), "got: {output}");
        assert!(output.contains("^"), "got: {output}");
    }

    #[test]
    fn test_quality_score_with_snippet() {
        let diag = Diagnostic {
            error: "test error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(10),
            column: Some(5),
            category: ErrorCategory::Syntax,
            note: Some("Explanation".to_string()),
            help: Some("Suggestion".to_string()),
            snippet: Some("let x = bad;".to_string()),
        };

        // With snippet, score should be higher
        let score = diag.quality_score();
        assert!(
            score > 0.9,
            "Score with snippet should be >0.9, got {score}",
        );
    }

    #[test]
    fn test_categorize_parse_error() {
        let error = Error::Parse(syn::Error::new(proc_macro2::Span::call_site(), "test"));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(diag.note.is_some());
        assert!(diag.help.is_some());
    }

include!("diagnostic_tests_ext_categorize.rs");
