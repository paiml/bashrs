//! Tests for formatting functions across corpus modules:
//! - `tier_analysis`: `format_tier_weights`, `format_tier_analysis`, `format_tier_targets`
//! - `citl`: `format_convergence_criteria`, `format_lint_pipeline`, `format_regression_report`
//! - `schema_enforcement`: `format_schema_report`, `format_grammar_errors`, `format_grammar_spec`
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::citl::{
#[test]
fn test_format_schema_report_empty_corpus() {
    let registry = crate::corpus::registry::CorpusRegistry { entries: vec![] };
    let report = validate_corpus(&registry);
    let table = format_schema_report(&report);

    assert!(table.contains("Total"));
    assert!(table.contains("0.0%"));
}

// ========================
// schema_enforcement: format_grammar_errors
// ========================

#[test]
fn test_format_grammar_errors_all_categories_shown() {
    let entries = vec![make_corpus_entry(
        "B-001",
        CorpusFormat::Bash,
        "#!/bin/sh\necho \"ok\"\n",
    )];
    let registry = crate::corpus::registry::CorpusRegistry { entries };
    let report = validate_corpus(&registry);
    let table = format_grammar_errors(&report);

    // All 8 GRAM codes should be listed
    for i in 1..=8 {
        assert!(
            table.contains(&format!("GRAM-{i:03}")),
            "Missing GRAM-{i:03} in table"
        );
    }
}

#[test]
fn test_format_grammar_errors_multiple_violations() {
    let entries = vec![
        make_corpus_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo $var; fi\n",
        ),
        make_corpus_entry("D-001", CorpusFormat::Dockerfile, "RUN apt-get update\n"),
    ];
    let registry = crate::corpus::registry::CorpusRegistry { entries };
    let report = validate_corpus(&registry);
    let table = format_grammar_errors(&report);

    assert!(table.contains("Entries with violations"));
    assert!(table.contains("B-001"));
    assert!(table.contains("D-001"));
}

// ========================
// schema_enforcement: format_grammar_spec
// ========================

#[test]
fn test_format_grammar_spec_all_formats() {
    let bash_spec = format_grammar_spec(CorpusFormat::Bash);
    assert!(bash_spec.contains("POSIX Shell Grammar"));
    assert!(bash_spec.contains("complete_command"));
    assert!(bash_spec.contains("L1: Lexical"));
    assert!(bash_spec.contains("L4: Behavioral"));

    let make_spec = format_grammar_spec(CorpusFormat::Makefile);
    assert!(make_spec.contains("GNU Make Grammar"));
    assert!(make_spec.contains("recipe"));
    assert!(make_spec.contains("assignment_op"));

    let docker_spec = format_grammar_spec(CorpusFormat::Dockerfile);
    assert!(docker_spec.contains("Dockerfile Grammar"));
    assert!(docker_spec.contains("FROM"));
    assert!(docker_spec.contains("exec_form"));
    assert!(docker_spec.contains("shell_form"));
}

// ========================
// schema_enforcement: SchemaReport::pass_rate edge cases
// ========================

#[test]
fn test_schema_report_pass_rate_all_valid() {
    let report = SchemaReport {
        results: vec![],
        total_entries: 100,
        valid_entries: 100,
        total_violations: 0,
        violations_by_category: vec![],
    };
    assert!((report.pass_rate() - 100.0).abs() < 0.01);
}

#[test]
fn test_schema_report_pass_rate_none_valid() {
    let report = SchemaReport {
        results: vec![],
        total_entries: 50,
        valid_entries: 0,
        total_violations: 50,
        violations_by_category: vec![],
    };
    assert!((report.pass_rate() - 0.0).abs() < 0.01);
}

// ========================
// schema_enforcement: GrammarCategory exhaustive coverage
// ========================

#[test]
fn test_grammar_category_fix_pattern_all() {
    for cat in GrammarCategory::all() {
        let fix = cat.fix_pattern();
        assert!(
            !fix.is_empty(),
            "fix_pattern for {:?} should not be empty",
            cat
        );
    }
}

#[test]
fn test_grammar_category_description_all() {
    for cat in GrammarCategory::all() {
        let desc = cat.description();
        assert!(
            !desc.is_empty(),
            "description for {:?} should not be empty",
            cat
        );
    }
}

#[test]
fn test_grammar_category_applicable_format_coverage() {
    let mut saw_bash = false;
    let mut saw_makefile = false;
    let mut saw_dockerfile = false;

    for cat in GrammarCategory::all() {
        match cat.applicable_format() {
            CorpusFormat::Bash => saw_bash = true,
            CorpusFormat::Makefile => saw_makefile = true,
            CorpusFormat::Dockerfile => saw_dockerfile = true,
        }
    }

    assert!(saw_bash, "At least one category should apply to Bash");
    assert!(
        saw_makefile,
        "At least one category should apply to Makefile"
    );
    assert!(
        saw_dockerfile,
        "At least one category should apply to Dockerfile"
    );
}

// ========================
// schema_enforcement: ValidationLayer display
// ========================

#[test]
fn test_validation_layer_display_all() {
    assert_eq!(format!("{}", ValidationLayer::Lexical), "L1:Lexical");
    assert_eq!(format!("{}", ValidationLayer::Syntactic), "L2:Syntactic");
    assert_eq!(format!("{}", ValidationLayer::Semantic), "L3:Semantic");
    assert_eq!(format!("{}", ValidationLayer::Behavioral), "L4:Behavioral");
}

// ========================
// schema_enforcement: validate_entry layer tracking
// ========================

#[test]
fn test_validate_entry_bash_all_layers_pass() {
    let entry = make_corpus_entry(
        "B-100",
        CorpusFormat::Bash,
        "#!/bin/sh\nset -eu\necho \"hello world\"\n",
    );
    let result = validate_entry(&entry);
    assert!(result.valid);
    assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    assert!(result.layers_passed.contains(&ValidationLayer::Syntactic));
    assert!(result.layers_passed.contains(&ValidationLayer::Semantic));
}

#[test]
fn test_validate_entry_dockerfile_all_layers_pass() {
    let entry = make_corpus_entry(
        "D-100",
        CorpusFormat::Dockerfile,
        "FROM alpine:3.18\nRUN apk add curl\nCMD [\"curl\", \"https://example.com\"]\n",
    );
    let result = validate_entry(&entry);
    assert!(result.valid);
    assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    assert!(result.layers_passed.contains(&ValidationLayer::Syntactic));
    assert!(result.layers_passed.contains(&ValidationLayer::Semantic));
}

#[test]
fn test_validate_entry_makefile_all_layers_pass() {
    let entry = make_corpus_entry(
        "M-100",
        CorpusFormat::Makefile,
        "CC := gcc\n\nall:\n\t$(CC) -o main main.c\n",
    );
    let result = validate_entry(&entry);
    assert!(result.valid);
    assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    assert!(result.layers_passed.contains(&ValidationLayer::Syntactic));
    assert!(result.layers_passed.contains(&ValidationLayer::Semantic));
}
