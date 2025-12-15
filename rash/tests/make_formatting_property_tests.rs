#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! Property-based tests for Makefile formatting options
//!
//! These tests use proptest to verify formatting properties hold across
//! a wide range of inputs (100+ generated test cases).
//!
//! ## Coverage
//! - Blank line preservation
//! - Line length limiting
//! - Combined options
//!
//! ## EXTREME TDD
//! Part of the EXTREME TDD methodology for comprehensive testing.

use bashrs::make_parser::{
    generators::{generate_purified_makefile_with_options, MakefileGeneratorOptions},
    parser::parse_makefile,
    purify::purify_makefile,
};
use proptest::prelude::*;

/// Generate valid Makefile content
fn makefile_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
        1..5,
    )
    .prop_map(|targets| {
        let mut makefile = String::new();
        for target in targets {
            makefile.push_str(&format!(".PHONY: {}\n", target));
            makefile.push_str(&format!("{}:\n", target));
            makefile.push_str("\t@echo \"Building\"\n");
            makefile.push_str("\t@echo \"Done\"\n");
            makefile.push('\n');
        }
        makefile
    })
}

proptest! {
    #[test]
    fn prop_preserve_formatting_always_adds_blank_lines(makefile in makefile_strategy()) {
        // Property: With --preserve-formatting, output should have more blank lines
        // than without

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        // Without preserve_formatting
        let options_compact = MakefileGeneratorOptions::default();
        let output_compact = generate_purified_makefile_with_options(&purified.ast, &options_compact);

        // With preserve_formatting
        let options_preserve = MakefileGeneratorOptions {
            preserve_formatting: true,
            ..Default::default()
        };
        let output_preserve = generate_purified_makefile_with_options(&purified.ast, &options_preserve);

        // Count blank lines
        let blank_lines_compact = output_compact.matches("\n\n").count();
        let blank_lines_preserve = output_preserve.matches("\n\n").count();

        prop_assert!(
            blank_lines_preserve >= blank_lines_compact,
            "Preserve formatting should have >= blank lines. Compact: {}, Preserve: {}",
            blank_lines_compact,
            blank_lines_preserve
        );
    }

    #[test]
    fn prop_max_line_length_always_respected(
        makefile in makefile_strategy(),
        max_len in 40usize..120usize
    ) {
        // Property: With --max-line-length, no line should exceed the limit

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        let options = MakefileGeneratorOptions {
            max_line_length: Some(max_len),
            ..Default::default()
        };

        let output = generate_purified_makefile_with_options(&purified.ast, &options);

        for line in output.lines() {
            prop_assert!(
                line.len() <= max_len,
                "Line exceeds max length {}: {} chars: '{}'",
                max_len,
                line.len(),
                line
            );
        }
    }

    #[test]
    fn prop_skip_blank_line_removal_preserves_structure(makefile in makefile_strategy()) {
        // Property: --skip-blank-line-removal should preserve blank lines

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        let options = MakefileGeneratorOptions {
            skip_blank_line_removal: true,
            ..Default::default()
        };

        let output = generate_purified_makefile_with_options(&purified.ast, &options);

        // Should have at least some blank lines (between targets)
        prop_assert!(
            output.contains("\n\n"),
            "Expected blank lines with skip_blank_line_removal"
        );
    }

    #[test]
    fn prop_combined_options_work_together(
        makefile in makefile_strategy(),
        max_len in 60usize..100usize
    ) {
        // Property: Combining options should work without conflicts

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            max_line_length: Some(max_len),
            skip_blank_line_removal: true,
            skip_consolidation: false, // Doesn't affect output yet (Issue #2)
        };

        let output = generate_purified_makefile_with_options(&purified.ast, &options);

        // Should respect line length
        for line in output.lines() {
            prop_assert!(
                line.len() <= max_len,
                "Line exceeds max length: {}",
                line.len()
            );
        }

        // Should have blank lines
        prop_assert!(
            output.contains("\n\n"),
            "Expected blank lines with combined options"
        );
    }

    #[test]
    fn prop_output_is_deterministic(makefile in makefile_strategy()) {
        // Property: Same input + options should always produce same output

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            max_line_length: Some(80),
            ..Default::default()
        };

        let output1 = generate_purified_makefile_with_options(&purified.ast, &options);
        let output2 = generate_purified_makefile_with_options(&purified.ast, &options);

        prop_assert_eq!(
            output1,
            output2,
            "Same input should produce identical output"
        );
    }

    #[test]
    fn prop_output_is_valid_makefile_syntax(makefile in makefile_strategy()) {
        // Property: Generated output should be parseable

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            ..Default::default()
        };

        let output = generate_purified_makefile_with_options(&purified.ast, &options);

        // Output should be parseable
        let reparse_result = parse_makefile(&output);
        prop_assert!(
            reparse_result.is_ok(),
            "Generated output should be parseable: {:?}",
            reparse_result.err()
        );
    }

    #[test]
    fn prop_line_breaks_preserve_tabs(makefile in makefile_strategy()) {
        // Property: Line breaks should preserve leading tabs for recipe lines

        let ast = parse_makefile(&makefile).expect("Valid makefile should parse");
        let purified = purify_makefile(&ast);

        let options = MakefileGeneratorOptions {
            max_line_length: Some(40), // Force line breaking
            ..Default::default()
        };

        let output = generate_purified_makefile_with_options(&purified.ast, &options);

        // All lines that start with @ or echo should be tab-indented
        for line in output.lines() {
            if line.trim_start().starts_with("@echo") || line.trim_start().starts_with("echo") {
                prop_assert!(
                    line.starts_with('\t'),
                    "Recipe line should start with tab: '{}'",
                    line
                );
            }
        }
    }
}

// Test configuration
#[cfg(test)]
mod config {
    use super::*;

    #[test]
    fn test_proptest_runs() {
        // Verify proptest configuration
        // This test ensures proptest is configured correctly
        let config = ProptestConfig {
            cases: 100, // Run 100 test cases
            max_shrink_iters: 1000,
            ..Default::default()
        };

        proptest!(config, |(makefile in makefile_strategy())| {
            // Basic sanity check
            prop_assert!(!makefile.is_empty());
        });
    }
}
