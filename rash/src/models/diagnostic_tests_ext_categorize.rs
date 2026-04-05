
    #[test]
    fn test_categorize_parse_error_expected_found() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected `;`, found `let`",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(
            diag.help.as_ref().unwrap().contains("expected"),
            "help: {:?}",
            diag.help
        );
    }

    #[test]
    fn test_categorize_parse_error_expected_semicolon() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected `;`",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(
            diag.help.as_ref().unwrap().contains("semicolon"),
            "help: {:?}",
            diag.help
        );
    }

    #[test]
    fn test_categorize_parse_error_unexpected_eof() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "unexpected eof",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(diag.note.as_ref().unwrap().contains("ended unexpectedly"));
    }

    #[test]
    fn test_categorize_parse_error_unexpected_token() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "unexpected token after this expression",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(diag.note.as_ref().unwrap().contains("unexpected token"));
    }

    #[test]
    fn test_categorize_validation_unsupported() {
        let error = Error::Validation("Unsupported expression type".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
    }

    #[test]
    fn test_categorize_validation_unsupported_macro() {
        let error = Error::Validation("Unsupported macro: vec!".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
        assert!(diag.help.as_ref().unwrap().contains("println!"));
    }

    #[test]
    fn test_categorize_validation_no_main() {
        let error = Error::Validation("No main function found".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Validation);
        assert!(diag.help.as_ref().unwrap().contains("fn main()"));
    }

    #[test]
    fn test_categorize_validation_must_have_initializer() {
        let error = Error::Validation("Variables must have initializers".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Validation);
        assert!(diag.help.as_ref().unwrap().contains("initial value"));
    }

    #[test]
    fn test_categorize_validation_generic() {
        let error = Error::Validation("Some validation issue".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_categorize_ir_generation() {
        let error = Error::IrGeneration("Failed to generate IR".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Transpilation);
        assert!(diag
            .note
            .as_ref()
            .unwrap()
            .contains("intermediate representation"));
    }

    #[test]
    fn test_categorize_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::Io(io_err);
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Io);
        assert!(diag.help.as_ref().unwrap().contains("exists"));
    }

    #[test]
    fn test_categorize_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let error = Error::Io(io_err);
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Io);
        assert!(diag.help.as_ref().unwrap().contains("chmod"));
    }

    #[test]
    fn test_categorize_unsupported() {
        let error = Error::Unsupported("async functions".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
        assert!(diag.note.as_ref().unwrap().contains("async functions"));
    }

    #[test]
    fn test_categorize_internal_error() {
        let error = Error::Internal("unexpected state".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Internal);
        assert!(diag.help.as_ref().unwrap().contains("bug"));
    }

    #[test]
    fn test_error_category_equality() {
        assert_eq!(ErrorCategory::Syntax, ErrorCategory::Syntax);
        assert_ne!(ErrorCategory::Syntax, ErrorCategory::Io);
        assert_eq!(
            ErrorCategory::UnsupportedFeature,
            ErrorCategory::UnsupportedFeature
        );
        assert_eq!(ErrorCategory::Validation, ErrorCategory::Validation);
        assert_eq!(ErrorCategory::Transpilation, ErrorCategory::Transpilation);
        assert_eq!(ErrorCategory::Internal, ErrorCategory::Internal);
    }

    #[test]
    fn test_diagnostic_clone() {
        let diag = Diagnostic {
            error: "test".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(1),
            column: Some(1),
            category: ErrorCategory::Syntax,
            note: Some("note".to_string()),
            help: Some("help".to_string()),
            snippet: Some("code".to_string()),
        };

        let cloned = diag.clone();
        assert_eq!(diag.error, cloned.error);
        assert_eq!(diag.file, cloned.file);
        assert_eq!(diag.category, cloned.category);
    }

    #[test]
    fn test_error_category_debug() {
        let cat = ErrorCategory::Syntax;
        let debug_str = format!("{cat:?}");
        assert_eq!(debug_str, "Syntax");
    }

    #[test]
    fn test_diagnostic_debug() {
        let diag = Diagnostic {
            error: "test".to_string(),
            file: None,
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        };

        let debug_str = format!("{diag:?}");
        assert!(debug_str.contains("Diagnostic"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_extract_snippet_middle_of_file() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";
        let snippet = Diagnostic::extract_snippet(source, 3, Some(2));
        assert!(snippet.contains("line 2"), "snippet: {snippet}");
        assert!(snippet.contains("line 3"), "snippet: {snippet}");
        assert!(snippet.contains("line 4"), "snippet: {snippet}");
        assert!(snippet.contains("^"), "snippet: {snippet}");
    }

    #[test]
    fn test_extract_snippet_first_line() {
        let source = "first line\nsecond line\nthird line";
        let snippet = Diagnostic::extract_snippet(source, 1, Some(5));
        assert!(snippet.contains("first line"), "snippet: {snippet}");
        assert!(snippet.contains("second line"), "snippet: {snippet}");
        assert!(snippet.contains("^"), "snippet: {snippet}");
    }

    #[test]
    fn test_extract_snippet_last_line() {
        let source = "line 1\nline 2\nlast line";
        let snippet = Diagnostic::extract_snippet(source, 3, Some(0));
        assert!(snippet.contains("line 2"), "snippet: {snippet}");
        assert!(snippet.contains("last line"), "snippet: {snippet}");
    }

    #[test]
    fn test_extract_snippet_no_column() {
        let source = "line 1\nline 2\nline 3";
        let snippet = Diagnostic::extract_snippet(source, 2, None);
        assert!(snippet.contains("line 2"), "snippet: {snippet}");
        assert!(!snippet.contains("^"), "no caret without column: {snippet}");
    }

    #[test]
    fn test_from_error_with_source() {
        let source = "fn main() {\n    let x = 42\n    let y = 10;\n}";
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected `;`",
        ));
        let diag =
            Diagnostic::from_error_with_source(&error, Some("test.rs".to_string()), Some(source));

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert_eq!(diag.file, Some("test.rs".to_string()));
        assert!(diag.help.as_ref().unwrap().contains("semicolon"));
    }

    #[test]
    fn test_from_error_unwraps_with_context() {
        let inner = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "test error",
        ));
        let error = Error::WithContext {
            inner: Box::new(inner),
            file: Some("ctx.rs".to_string()),
            source_code: Some("fn main() {}".to_string()),
        };

        let diag = Diagnostic::from_error(&error, None);
        assert_eq!(diag.file, Some("ctx.rs".to_string()));
        assert_eq!(diag.category, ErrorCategory::Syntax);
    }

    #[test]
    fn test_from_error_with_context_prefers_context_file() {
        let inner = Error::Validation("test".to_string());
        let error = Error::WithContext {
            inner: Box::new(inner),
            file: Some("from_context.rs".to_string()),
            source_code: None,
        };

        // Even if we pass a file to from_error, WithContext's file takes precedence
        let diag = Diagnostic::from_error(&error, Some("from_caller.rs".to_string()));
        assert_eq!(diag.file, Some("from_context.rs".to_string()));
    }

    #[test]
    fn test_from_error_with_context_falls_back_to_caller_file() {
        let inner = Error::Validation("test".to_string());
        let error = Error::WithContext {
            inner: Box::new(inner),
            file: None,
            source_code: None,
        };

        let diag = Diagnostic::from_error(&error, Some("fallback.rs".to_string()));
        assert_eq!(diag.file, Some("fallback.rs".to_string()));
    }

    #[test]
    fn test_category_tags() {
        assert_eq!(ErrorCategory::Syntax.tag(), "syntax");
        assert_eq!(ErrorCategory::UnsupportedFeature.tag(), "unsupported");
        assert_eq!(ErrorCategory::Validation.tag(), "validation");
        assert_eq!(ErrorCategory::Transpilation.tag(), "transpile");
        assert_eq!(ErrorCategory::Io.tag(), "io");
        assert_eq!(ErrorCategory::Internal.tag(), "internal");
    }

    #[test]
    fn test_extract_message_strips_parse_prefix() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "cannot parse",
        ));
        let msg = Diagnostic::extract_message(&error);
        // Should strip "Parse error: " prefix
        assert_eq!(msg, "cannot parse");
    }

    #[test]
    fn test_extract_message_keeps_validation_prefix() {
        let error = Error::Validation("something wrong".to_string());
        let msg = Diagnostic::extract_message(&error);
        assert_eq!(msg, "AST validation error: something wrong");
    }
}
