#[cfg(test)]
mod tests {
    use super::*;

    // ===== RED PHASE: Unit Tests for mark_top_level_called_functions =====
    // NASA-level quality: Comprehensive coverage, edge cases, isolation

    #[test]
    fn test_mark_top_level_called_functions_empty_source() {
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions("", &mut report);

        assert!(
            report.covered_functions.is_empty(),
            "Empty source should not mark any functions as covered"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_no_calls() {
        let source = r#"
#!/bin/bash
# Script with no function calls
echo "Hello World"
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.is_empty(),
            "No function calls should result in empty coverage"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_simple_call() {
        let source = r#"
#!/bin/bash
greet() {
    echo "Hello"
}

greet
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.contains("greet"),
            "Top-level function call should be marked as covered"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_call_with_parens() {
        let source = r#"
#!/bin/bash
deploy() {
    echo "Deploying"
}

deploy()
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("deploy".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.contains("deploy"),
            "Function call with parentheses should be marked as covered"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_inside_function_not_covered() {
        let source = r#"
#!/bin/bash
greet() {
    echo "Hello"
}

main() {
    greet
}
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());
        report.all_functions.push("main".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            !report.covered_functions.contains("greet"),
            "Function called INSIDE another function should NOT be marked as top-level covered"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_multiple_calls() {
        let source = r#"
#!/bin/bash
greet() {
    echo "Hello"
}

deploy() {
    echo "Deploy"
}

greet
deploy
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());
        report.all_functions.push("deploy".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert_eq!(
            report.covered_functions.len(),
            2,
            "Both top-level function calls should be marked as covered"
        );
        assert!(report.covered_functions.contains("greet"));
        assert!(report.covered_functions.contains("deploy"));
    }

    #[test]
    fn test_mark_top_level_called_functions_skip_comments() {
        let source = r#"
#!/bin/bash
greet() {
    echo "Hello"
}

# greet - this is a comment, not a call
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.is_empty(),
            "Function name in comment should NOT be marked as covered"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_skip_empty_lines() {
        let source = r#"
#!/bin/bash
greet() {
    echo "Hello"
}


"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.is_empty(),
            "Empty lines should not affect coverage"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_function_keyword_style() {
        let source = r#"
#!/bin/bash
function greet {
    echo "Hello"
}

greet
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.contains("greet"),
            "Function defined with 'function' keyword should work"
        );
    }

    #[test]
    fn test_mark_top_level_called_functions_partial_match_rejected() {
        let source = r#"
#!/bin/bash
greet() {
    echo "Hello"
}

# "greet_user" should NOT match "greet"
greet_user
"#;
        let mut report = CoverageReport::new();
        report.all_functions.push("greet".to_string());

        mark_top_level_called_functions(source, &mut report);

        assert!(
            report.covered_functions.is_empty(),
            "Partial function name match should NOT mark function as covered"
        );
    }
}

#[cfg(test)]
mod mod_tests_ext_mark {
    use super::*;
}

include!("mod_tests_ext_mark.rs");
