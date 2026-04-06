
    #[test]
    fn test_mark_top_level_called_functions_exact_word_match() {
        let source = r#"
#!/bin/bash
test() {
    echo "Test"
}

# Word boundary test: "test" as standalone word
echo before test after
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("test".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.contains("test"),
            "Exact word match should mark function as covered"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_nested_braces() {
        let source = r#"
#!/bin/bash
outer() {
    inner() {
        echo "Inner"
    }
}

deploy
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("deploy".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.contains("deploy"),
            "Top-level call after nested function definitions should be detected"
        );
    }

    // ============================================================================
    // CoverageReport Method Tests
    // ============================================================================

    #[test]
    fn test_coverage_report_new() {
        let report = CoverageReport::new();
        assert_eq!(report.total_lines, 0);
        assert!(report.covered_lines.is_empty());
        assert!(report.all_functions.is_empty());
        assert!(report.covered_functions.is_empty());
        assert!(report.line_coverage.is_empty());
    }

    #[test]
    fn test_coverage_report_default() {
        let report = CoverageReport::default();
        assert_eq!(report.total_lines, 0);
        assert!(report.covered_lines.is_empty());
    }

    #[test]
    fn test_line_coverage_percent_empty() {
        let report = CoverageReport::new();
        assert_eq!(report.line_coverage_percent(), 0.0);
    }

    #[test]
    fn test_line_coverage_percent_full() {
        let mut report = CoverageReport::new();
        report.total_lines = 10;
        for i in 1..=10 {
            report.covered_lines.insert(i);
        }
        assert_eq!(report.line_coverage_percent(), 100.0);
    }

    #[test]
    fn test_line_coverage_percent_partial() {
        let mut report = CoverageReport::new();
        report.total_lines = 10;
        for i in 1..=5 {
            report.covered_lines.insert(i);
        }
        assert_eq!(report.line_coverage_percent(), 50.0);
    }

    #[test]
    fn test_function_coverage_percent_empty() {
        let report = CoverageReport::new();
        assert_eq!(report.function_coverage_percent(), 0.0);
    }

    #[test]
    fn test_function_coverage_percent_full() {
        let mut report = CoverageReport::new();
        report.all_functions = vec!["foo".to_string(), "bar".to_string()];
        report.covered_functions.insert("foo".to_string());
        report.covered_functions.insert("bar".to_string());
        assert_eq!(report.function_coverage_percent(), 100.0);
    }

    #[test]
    fn test_function_coverage_percent_partial() {
        let mut report = CoverageReport::new();
        report.all_functions = vec!["foo".to_string(), "bar".to_string()];
        report.covered_functions.insert("foo".to_string());
        assert_eq!(report.function_coverage_percent(), 50.0);
    }

    #[test]
    fn test_uncovered_lines_empty() {
        let report = CoverageReport::new();
        assert!(report.uncovered_lines().is_empty());
    }

    #[test]
    fn test_uncovered_lines_sorted() {
        let mut report = CoverageReport::new();
        report.line_coverage.insert(5, false);
        report.line_coverage.insert(2, true);
        report.line_coverage.insert(8, false);
        report.line_coverage.insert(1, false);

        let uncovered = report.uncovered_lines();
        assert_eq!(uncovered, vec![1, 5, 8]);
    }

    #[test]
    fn test_uncovered_functions_empty() {
        let report = CoverageReport::new();
        assert!(report.uncovered_functions().is_empty());
    }

    #[test]
    fn test_uncovered_functions() {
        let mut report = CoverageReport::new();
        report.all_functions = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        report.covered_functions.insert("bar".to_string());

        let uncovered = report.uncovered_functions();
        assert_eq!(uncovered.len(), 2);
        assert!(uncovered.contains(&"foo".to_string()));
        assert!(uncovered.contains(&"baz".to_string()));
    }

    #[test]
    fn test_generate_coverage_no_tests() {
        let source = r#"#!/bin/bash
echo "No tests here"
"#;
        let result = generate_coverage(source);
        assert!(result.is_ok());
        let report = result.unwrap();
        // No tests = zero covered lines
        assert_eq!(report.covered_lines.len(), 0);
    }

    // ============================================================================
    // Helper Function Tests
    // ============================================================================

    #[test]
    fn test_is_function_start_parens_style() {
        assert!(is_function_start("foo() {"));
        assert!(is_function_start("my_func() {"));
        assert!(is_function_start("  bar() {"));
    }

    #[test]
    fn test_is_function_start_keyword_style() {
        assert!(is_function_start("function foo"));
        assert!(is_function_start("function my_func {"));
    }

    #[test]
    fn test_is_function_start_not_function() {
        assert!(!is_function_start("echo hello"));
        assert!(!is_function_start("# function comment"));
        assert!(!is_function_start("x=5"));
    }

    #[test]
    fn test_extract_function_name_parens_style() {
        assert_eq!(extract_function_name("foo() {"), "foo");
        assert_eq!(extract_function_name("my_func() {"), "my_func");
        assert_eq!(extract_function_name("bar() {"), "bar");
    }

    #[test]
    fn test_extract_function_name_keyword_style() {
        assert_eq!(extract_function_name("function foo {"), "foo");
        assert_eq!(extract_function_name("function my_func"), "my_func");
    }

    #[test]
    fn test_extract_function_name_unknown() {
        assert_eq!(extract_function_name("echo hello"), "unknown");
        assert_eq!(extract_function_name("x=5"), "unknown");
    }

    #[test]
    fn test_is_function_end_true() {
        assert!(is_function_end("}"));
    }

    #[test]
    fn test_is_function_end_false() {
        assert!(!is_function_end("} else {"));
        assert!(!is_function_end("echo }"));
        assert!(!is_function_end(""));
    }

    #[test]
    fn test_is_top_level_code_true() {
        assert!(is_top_level_code("echo hello"));
        assert!(is_top_level_code("foo"));
        assert!(is_top_level_code("x=5"));
    }

    #[test]
    fn test_is_top_level_code_false() {
        assert!(!is_top_level_code(""));
        assert!(!is_top_level_code("#comment"));
        assert!(!is_top_level_code("# another comment"));
    }

    #[test]
    fn test_mark_line_covered() {
        let mut report = CoverageReport::new();
        report.line_coverage.insert(5, false);
        mark_line_covered(5, &mut report);
        assert!(report.covered_lines.contains(&5));
        assert_eq!(report.line_coverage.get(&5), Some(&true));
    }

    #[test]
    fn test_should_skip_line_empty() {
        assert!(should_skip_line(""));
        // Note: whitespace-only strings return false (not trimmed)
        assert!(!should_skip_line("   "));
    }

    #[test]
    fn test_should_skip_line_comment() {
        assert!(should_skip_line("#comment"));
        assert!(should_skip_line("# another comment"));
        assert!(should_skip_line("#!"));
    }

    #[test]
    fn test_should_skip_line_code() {
        assert!(!should_skip_line("echo hello"));
        assert!(!should_skip_line("x=5"));
    }

    #[test]
    fn test_is_function_start_line_parens() {
        assert!(is_function_start_line("foo() {"));
        assert!(is_function_start_line("bar() {"));
    }

    #[test]
    fn test_is_function_start_line_keyword() {
        assert!(is_function_start_line("function foo"));
        assert!(is_function_start_line("function bar {"));
    }

    #[test]
    fn test_is_function_start_line_not_function() {
        assert!(!is_function_start_line("echo hello"));
        assert!(!is_function_start_line("x=5"));
    }

    #[test]
    fn test_should_exit_function_true() {
        assert!(should_exit_function("}", true));
    }

    #[test]
    fn test_should_exit_function_false_not_in_function() {
        assert!(!should_exit_function("}", false));
    }

    #[test]
    fn test_should_exit_function_false_not_brace() {
        assert!(!should_exit_function("echo hello", true));
        assert!(!should_exit_function("} else {", true));
    }

    #[test]
    fn test_is_function_call_exact_match() {
        assert!(is_function_call("foo", "foo"));
        assert!(is_function_call("my_func", "my_func"));
    }

    #[test]
    fn test_is_function_call_with_parens() {
        assert!(is_function_call("foo()", "foo"));
        assert!(is_function_call("foo(arg)", "foo"));
    }

    #[test]
    fn test_is_function_call_not_match() {
        assert!(!is_function_call("foobar", "foo"));
        assert!(!is_function_call("bar", "foo"));
    }

    #[test]
    fn test_analyze_script_empty() {
        let mut report = CoverageReport::new();
        analyze_script("", &mut report);
        assert_eq!(report.total_lines, 0);
        assert!(report.all_functions.is_empty());
    }

    #[test]
    fn test_analyze_script_simple() {
        let source = "echo hello\necho world";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert_eq!(report.total_lines, 2);
    }

    #[test]
    fn test_analyze_script_with_function() {
        let source = "foo() {\n  echo hello\n}\n";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert!(report.all_functions.contains(&"foo".to_string()));
    }

    #[test]
    fn test_analyze_script_function_keyword_style() {
        let source = "function bar {\n  echo hello\n}\n";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert!(report.all_functions.contains(&"bar".to_string()));
    }

    #[test]
    fn test_analyze_script_skip_test_functions() {
        let source = "test_foo() {\n  echo test\n}\n";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert!(!report.all_functions.contains(&"test_foo".to_string()));
    }

    #[test]
    fn test_analyze_script_skip_comments() {
        let source = "# comment\necho hello\n# another";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert_eq!(report.total_lines, 1);
    }

    #[test]
    fn test_analyze_script_skip_empty_lines() {
        let source = "\n\necho hello\n\n";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert_eq!(report.total_lines, 1);
    }

    #[test]
    fn test_analyze_script_line_coverage_map() {
        let source = "echo hello";
        let mut report = CoverageReport::new();
        analyze_script(source, &mut report);
        assert_eq!(report.line_coverage.get(&1), Some(&false));
    }

    #[test]
    fn test_mark_covered_functions_lines_empty() {
        let covered = HashSet::new();
        let mut report = CoverageReport::new();
        mark_covered_functions_lines("", &covered, &mut report);
        assert!(report.covered_lines.is_empty());
    }

    #[test]
    fn test_mark_covered_functions_lines_marks_correctly() {
        let source = "foo() {\n  echo hello\n}\n";
        let mut covered = HashSet::new();
        covered.insert("foo".to_string());
        let mut report = CoverageReport::new();
        report.line_coverage.insert(1, false);
        report.line_coverage.insert(2, false);
        report.line_coverage.insert(3, false);
        mark_covered_functions_lines(source, &covered, &mut report);
        // Lines inside covered function should be marked
        assert!(report.covered_lines.contains(&1) || report.covered_lines.contains(&2));
    }

    #[test]
    fn test_mark_covered_functions_lines_top_level() {
        let source = "echo hello";
        let covered = HashSet::new();
        let mut report = CoverageReport::new();
        report.line_coverage.insert(1, false);
        mark_covered_functions_lines(source, &covered, &mut report);
        // Top level code is assumed executed
        assert!(report.covered_lines.contains(&1));
    }

    #[test]
    fn test_mark_function_calls_on_line_simple() {
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());
        mark_function_calls_on_line("greet", &mut report);
        assert!(report.covered_functions.contains("greet"));
    }

    #[test]
    fn test_mark_function_calls_on_line_with_args() {
        let mut report = CoverageReport::new();
        report.all_functions.push("deploy".to_string());
        mark_function_calls_on_line("deploy()", &mut report);
        assert!(report.covered_functions.contains("deploy"));
    }

    #[test]
    fn test_mark_function_calls_on_line_not_found() {
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());
        mark_function_calls_on_line("echo hello", &mut report);
        assert!(report.covered_functions.is_empty());
    }

    #[test]
    fn test_generate_coverage_with_tests() {
        let source = r#"#!/bin/bash
greet() {
    echo "Hello"
}

test_greet() {
    greet
    assert "Hello"
}
"#;
        let result = generate_coverage(source);
        assert!(result.is_ok());
    }
