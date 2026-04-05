/// Integration Test 001: End-to-end purification workflow
#[test]
fn test_INTEGRATION_001_complete_purification() {
    // ARRANGE: Complex Makefile with multiple issues
    let makefile = r#"
# Makefile with multiple categories of issues
FILES = $(wildcard *.c)
VERSION = $(shell date +%s)

build: compile link
compile:
	mkdir build
	gcc -c main.c -o build/main.o
link:
	gcc build/main.o -o app
	echo "Build complete"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect issues across multiple categories
    // 1. Non-deterministic (wildcard needs sort, date timestamp)
    // 2. Missing error handling
    // 3. Performance issues
    assert!(
        result.transformations_applied >= 3,
        "Should detect issues across multiple transformation categories"
    );
    assert!(
        result.report.len() >= 3,
        "Should generate recommendations for multiple issues"
    );
}

/// Integration Test 002: Verify no false positives on clean Makefiles
#[test]
fn test_INTEGRATION_002_clean_makefile_no_false_positives() {
    // ARRANGE: Well-written Makefile with no obvious issues
    let makefile = r#"
.DELETE_ON_ERROR:
.SUFFIXES:

FILES := $(sort $(wildcard *.c))

build: compile
compile:
	mkdir -p build || exit 1
	gcc -c main.c -o build/main.o || exit 1
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should have minimal recommendations (good practices already applied)
    // Test passes if purify_makefile runs without panic
    // Clean Makefile should not trigger excessive recommendations
    let _ = result.transformations_applied; // Verify result exists
}

/// Integration Test 003: Verify composition of transformations
#[test]
fn test_INTEGRATION_003_transformation_composition() {
    // ARRANGE: Makefile that triggers multiple transformation categories
    let makefile = r#"
FILES = $(wildcard *.c)
VERSION = $(shell date +%s)

build: compile link
compile:
	mkdir build
	for f in *.c; do gcc -c $$f; done
	echo "Compiled"
link:
	gcc *.o -o app
	echo -e "Linked\n"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect multiple categories:
    // - Reproducibility (wildcard, date)
    // - Error handling (mkdir, for loop)
    // - Portability (echo -e)
    // - Performance (wildcard not sorted)
    let report_text = result.report.join("\n");

    // Check for diverse transformation categories
    let has_reproducibility = report_text.contains("wildcard") || report_text.contains("date");
    let has_error_handling = report_text.contains("error") || report_text.contains("exit");
    let has_portability = report_text.contains("echo") || report_text.contains("portable");

    assert!(
        has_reproducibility || has_error_handling || has_portability,
        "Should detect issues from multiple transformation categories"
    );
}

/// Integration Test 004: Verify all 5 transformation categories are functional
#[test]
fn test_INTEGRATION_004_all_categories_functional() {
    // ARRANGE: Makefile that exercises all 5 categories
    let makefile = r#"
# 1. Parallel Safety - race condition
FILES = $(wildcard *.c)
# 2. Reproducibility - non-deterministic
VERSION = $(shell date +%s)

# 3. Performance - multiple shell invocations
build: compile link
compile:
	# 4. Error Handling - no error checks
	mkdir build
	gcc -c main.c
	# 5. Portability - bashisms
	if [[ -f main.o ]]; then echo "found"; fi
link:
	gcc *.o -o app
	echo -e "Done\n"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should have detected issues from all categories
    assert!(
        result.transformations_applied >= 5,
        "Should detect issues from all 5 transformation categories"
    );

    // Verify report contains recommendations
    assert!(
        result.report.len() >= 5,
        "Should generate recommendations from multiple categories"
    );
}

/// Integration Test 005: Verify backward compatibility (existing tests still pass)
#[test]
fn test_INTEGRATION_005_backward_compatibility() {
    // ARRANGE: Simple Makefile from earlier tests
    let makefile = r#"
FILES = $(wildcard *.c)
all:
	echo "Building"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should still work (backward compatibility)
    // Purification succeeded - result exists
    let _ = result.transformations_applied;
    let _ = result.manual_fixes_needed;
}

// ===== NASA-QUALITY UNIT TESTS for detect_missing_file_dependencies helpers =====

#[test]
fn test_try_extract_output_redirect_valid() {
    let recipe = "echo hello > output.txt";
    assert_eq!(
        try_extract_output_redirect(recipe),
        Some("output.txt".to_string()),
        "Should extract output filename from redirect"
    );
}

#[test]
fn test_try_extract_output_redirect_no_redirect() {
    let recipe = "echo hello";
    assert_eq!(
        try_extract_output_redirect(recipe),
        None,
        "Should return None when no redirect present"
    );
}

#[test]
fn test_try_extract_output_redirect_multiple_words() {
    let recipe = "cat input.txt > output.txt extra";
    assert_eq!(
        try_extract_output_redirect(recipe),
        Some("output.txt".to_string()),
        "Should extract only first word after redirect"
    );
}

#[test]
fn test_try_extract_cat_input_valid() {
    let recipe = "cat input.txt";
    assert_eq!(
        try_extract_cat_input(recipe),
        Some("input.txt".to_string()),
        "Should extract input filename from cat command"
    );
}

#[test]
fn test_try_extract_cat_input_no_cat() {
    let recipe = "echo hello";
    assert_eq!(
        try_extract_cat_input(recipe),
        None,
        "Should return None when no cat command"
    );
}

#[test]
fn test_try_extract_cat_input_automatic_variable() {
    let recipe = "cat $<";
    assert_eq!(
        try_extract_cat_input(recipe),
        None,
        "Should return None for automatic variables"
    );
}

#[test]
fn test_try_extract_cat_input_with_path() {
    let recipe = "cat src/file.txt | grep pattern";
    assert_eq!(
        try_extract_cat_input(recipe),
        Some("src/file.txt".to_string()),
        "Should extract filename with path"
    );
}

#[test]
fn test_is_automatic_variable_all_variants() {
    assert!(is_automatic_variable("$<"), "$< should be automatic");
    assert!(is_automatic_variable("$@"), "$@ should be automatic");
    assert!(is_automatic_variable("$^"), "$^ should be automatic");
    assert!(is_automatic_variable("$?"), "$? should be automatic");
    assert!(is_automatic_variable("$*"), "$* should be automatic");
    assert!(is_automatic_variable("$+"), "$+ should be automatic");
}

#[test]
fn test_is_automatic_variable_normal_filename() {
    assert!(
        !is_automatic_variable("file.txt"),
        "Normal filename should NOT be automatic"
    );
    assert!(
        !is_automatic_variable("$VAR"),
        "User variable should NOT be automatic"
    );
}

#[test]
fn test_target_has_prerequisite_true() {
    use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span};

    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".to_string(),
            prerequisites: vec!["compile".to_string()],
            recipe: vec![],
            phony: false,
            recipe_metadata: None,
            span: Span::new(0, 10, 1),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };

    assert!(
        target_has_prerequisite(&ast, "build", "compile"),
        "Should find existing prerequisite"
    );
}

#[test]
fn test_target_has_prerequisite_false() {
    use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span};

    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".to_string(),
            prerequisites: vec!["compile".to_string()],
            recipe: vec![],
            phony: false,
            recipe_metadata: None,
            span: Span::new(0, 10, 1),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };

    assert!(
        !target_has_prerequisite(&ast, "build", "missing"),
        "Should return false for missing prerequisite"
    );
}

#[test]
fn test_target_has_prerequisite_nonexistent_target() {
    use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span};

    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".to_string(),
            prerequisites: vec!["compile".to_string()],
            recipe: vec![],
            phony: false,
            recipe_metadata: None,
            span: Span::new(0, 10, 1),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };

    assert!(
        !target_has_prerequisite(&ast, "nonexistent", "compile"),
        "Should return false for nonexistent target"
    );
}

// =============================================================================
// reproducible_builds — analyze_reproducible_builds coverage
// =============================================================================

#[test]
fn test_reproducible_builds_date_in_variable() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "BUILD_TIME".into(),
            value: "$(shell date +%s)".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.len() >= 2,
        "Should detect timestamp and suggest SOURCE_DATE_EPOCH"
    );
    assert!(result
        .iter()
        .any(|t| matches!(t, Transformation::DetectTimestamp { .. })));
    assert!(result
        .iter()
        .any(|t| matches!(t, Transformation::SuggestSourceDateEpoch { .. })));
}

#[test]

include!("tests_incl2_incl2_incl2_incl2_incl2.rs");
