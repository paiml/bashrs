#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_col, 10);
    }

    #[test]
    fn test_span_point() {
        let span = Span::point(5, 10);
        assert_eq!(span.start_line, 5);
        assert_eq!(span.start_col, 10);
        assert_eq!(span.end_line, 5);
        assert_eq!(span.end_col, 10);
    }

    #[test]
    fn test_span_display_single_line() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.to_string(), "1:5-10");
    }

    #[test]
    fn test_span_display_multi_line() {
        let span = Span::new(1, 5, 3, 10);
        assert_eq!(span.to_string(), "1:5-3:10");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Note);
        assert!(Severity::Note < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Error.to_string(), "error");
    }

    #[test]
    fn test_fix_creation() {
        let fix = Fix::new("\"$var\"");
        assert_eq!(fix.replacement, "\"$var\"");
    }

    #[test]
    fn test_diagnostic_creation() {
        let span = Span::new(1, 5, 1, 10);
        let diag = Diagnostic::new(
            "SC2086",
            Severity::Warning,
            "Double quote to prevent globbing",
            span,
        );

        assert_eq!(diag.code, "SC2086");
        assert_eq!(diag.severity, Severity::Warning);
        assert_eq!(diag.message, "Double quote to prevent globbing");
        assert_eq!(diag.span, span);
        assert!(diag.fix.is_none());
    }

    #[test]
    fn test_diagnostic_with_fix() {
        let span = Span::new(1, 5, 1, 10);
        let fix = Fix::new("\"$var\"");
        let diag = Diagnostic::new("SC2086", Severity::Warning, "Double quote", span)
            .with_fix(fix.clone());

        assert!(diag.fix.is_some());
        assert_eq!(diag.fix.unwrap().replacement, "\"$var\"");
    }

    #[test]
    fn test_diagnostic_display() {
        let span = Span::new(1, 5, 1, 10);
        let diag = Diagnostic::new("SC2086", Severity::Warning, "Double quote", span);

        let display = diag.to_string();
        assert!(display.contains("1:5-10"));
        assert!(display.contains("warning"));
        assert!(display.contains("SC2086"));
        assert!(display.contains("Double quote"));
    }

    #[test]
    fn test_lint_result_new() {
        let result = LintResult::new();
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_lint_result_add() {
        let mut result = LintResult::new();
        let span = Span::new(1, 1, 1, 5);
        let diag = Diagnostic::new("SC2086", Severity::Warning, "Test", span);

        result.add(diag);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_lint_result_merge() {
        let mut result1 = LintResult::new();
        let mut result2 = LintResult::new();

        let span = Span::new(1, 1, 1, 5);
        result1.add(Diagnostic::new("SC2086", Severity::Warning, "Test 1", span));
        result2.add(Diagnostic::new("SC2046", Severity::Warning, "Test 2", span));

        result1.merge(result2);
        assert_eq!(result1.diagnostics.len(), 2);
    }

    #[test]
    fn test_lint_result_has_errors() {
        let mut result = LintResult::new();
        assert!(!result.has_errors());

        let span = Span::new(1, 1, 1, 5);
        result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));
        assert!(!result.has_errors());

        result.add(Diagnostic::new("SC2046", Severity::Error, "Test", span));
        assert!(result.has_errors());
    }

    #[test]
    fn test_lint_result_has_warnings() {
        let mut result = LintResult::new();
        assert!(!result.has_warnings());

        let span = Span::new(1, 1, 1, 5);
        result.add(Diagnostic::new("SC2086", Severity::Info, "Test", span));
        assert!(!result.has_warnings());

        result.add(Diagnostic::new("SC2046", Severity::Warning, "Test", span));
        assert!(result.has_warnings());
    }

    #[test]
    fn test_lint_result_count_by_severity() {
        let mut result = LintResult::new();
        let span = Span::new(1, 1, 1, 5);

        result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));
        result.add(Diagnostic::new("SC2046", Severity::Warning, "Test", span));
        result.add(Diagnostic::new("SC2116", Severity::Error, "Test", span));

        assert_eq!(result.count_by_severity(Severity::Warning), 2);
        assert_eq!(result.count_by_severity(Severity::Error), 1);
        assert_eq!(result.count_by_severity(Severity::Info), 0);
    }

    #[test]
    fn test_lint_result_max_severity() {
        let mut result = LintResult::new();
        assert_eq!(result.max_severity(), None);

        let span = Span::new(1, 1, 1, 5);
        result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));
        assert_eq!(result.max_severity(), Some(Severity::Warning));

        result.add(Diagnostic::new("SC2046", Severity::Error, "Test", span));
        assert_eq!(result.max_severity(), Some(Severity::Error));
    }
}
