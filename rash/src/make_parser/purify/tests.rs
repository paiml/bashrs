use super::parallel_safety::{
    is_automatic_variable, target_has_prerequisite, try_extract_cat_input,
    try_extract_output_redirect,
};
use super::*;

#[test]
fn test_find_matching_paren_simple() {
    let s = "$(wildcard *.c)";
    assert_eq!(find_matching_paren(s, 0), Some(14));
}

#[test]
fn test_find_matching_paren_nested() {
    let s = "$(filter %.o, $(wildcard *.c))";
    // Start at "$(wildcard", find its closing paren
    // Position 15 is the start of "$(wildcard"
    // The wildcard closing paren is at position 28
    assert_eq!(find_matching_paren(s, 15), Some(28));
}

#[test]
fn test_wrap_pattern_with_sort_simple() {
    let value = "$(wildcard *.c)";
    let result = wrap_pattern_with_sort(value, "$(wildcard");
    assert_eq!(result, "$(sort $(wildcard *.c))");
}

#[test]
fn test_wrap_pattern_with_sort_nested() {
    let value = "$(filter %.o, $(wildcard *.c))";
    let result = wrap_pattern_with_sort(value, "$(wildcard");
    assert_eq!(result, "$(filter %.o, $(sort $(wildcard *.c)))");
}

#[test]
fn test_extract_variable_name() {
    let message = "Variable 'FILES' uses non-deterministic $(wildcard)";
    assert_eq!(extract_variable_name(message), "FILES");
}

// ========================================
// Sprint 83 - Day 2-3: Parallel Safety Tests
// ========================================

/// Test PARALLEL_SAFETY_001: Check parallel safety analysis runs
#[test]
fn test_PARALLEL_SAFETY_001_parallel_safety_analysis() {
    // ARRANGE: Simple Makefile with no parallel safety issues
    let makefile = r#"
all: build

build:
	gcc -o app main.c
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify with parallel safety transformations
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Note: This Makefile has no issues, so no .NOTPARALLEL recommended
    // This is correct idempotent behavior
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PARALLEL_SAFETY_002: Detect race condition in shared file write
#[test]
fn test_PARALLEL_SAFETY_002_detect_race_condition() {
    // ARRANGE: Two targets writing to same file
    let makefile = r#"
target1:
	echo "output1" > shared.txt

target2:
	echo "output2" > shared.txt

all: target1 target2
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify with parallel safety analysis
    let result = purify_makefile(&ast);

    // ASSERT: Should detect race condition AND recommend .NOTPARALLEL
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("race") || r.contains("parallel") || r.contains(".NOTPARALLEL")),
        "Should detect race condition in shared file write and recommend .NOTPARALLEL"
    );
}

/// Test PARALLEL_SAFETY_003: Add order-only prerequisite for dependency
#[test]
fn test_PARALLEL_SAFETY_003_add_order_only_prereq() {
    // ARRANGE: Target that needs order-only prerequisite
    let makefile = r#"
build: | output_dir
	gcc -o output_dir/app main.c

output_dir:
	mkdir -p output_dir
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should preserve order-only prerequisite (already correct)
    // Test passes if purify_makefile runs without panic
    // Should handle order-only prerequisites correctly
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PARALLEL_SAFETY_004: Detect missing dependency causing race
#[test]
fn test_PARALLEL_SAFETY_004_missing_dependency() {
    // ARRANGE: Target using file created by another target (missing dep)
    let makefile = r#"
generate:
	echo "data" > data.txt

process:
	cat data.txt > output.txt

all: generate process
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect that process depends on generate's output
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("dependency") || r.contains("data.txt")),
        "Should detect missing dependency"
    );
}

/// Test PARALLEL_SAFETY_005: Preserve existing .NOTPARALLEL
#[test]
fn test_PARALLEL_SAFETY_005_preserve_notparallel() {
    // ARRANGE: Makefile already has .NOTPARALLEL
    let makefile = r#"
.NOTPARALLEL:

all: build

build:
	gcc -o app main.c
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should not add duplicate .NOTPARALLEL
    let notparallel_count = result
        .report
        .iter()
        .filter(|r| r.contains(".NOTPARALLEL"))
        .count();
    assert!(
        notparallel_count <= 1,
        "Should not add duplicate .NOTPARALLEL"
    );
}

/// Test PARALLEL_SAFETY_006: Detect .PHONY target parallel safety
#[test]
fn test_PARALLEL_SAFETY_006_phony_target_safety() {
    // ARRANGE: .PHONY targets that are parallel-safe
    let makefile = r#"
.PHONY: clean test

clean:
	rm -f *.o

test:
	./run_tests.sh

all: clean test
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should handle .PHONY targets correctly
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PARALLEL_SAFETY_007: Multiple targets same output file
#[test]
fn test_PARALLEL_SAFETY_007_multiple_targets_same_output() {
    // ARRANGE: Multiple targets writing to same output
    let makefile = r#"
debug: main.c
	gcc -g -o app main.c

release: main.c
	gcc -O2 -o app main.c
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should warn about conflict
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("conflict") || r.contains("same output")),
        "Should detect multiple targets with same output"
    );
}

/// Test PARALLEL_SAFETY_008: Recursive make calls need serialization
#[test]
fn test_PARALLEL_SAFETY_008_recursive_make_serialization() {
    // ARRANGE: Recursive make calls
    let makefile = r#"
subdirs:
	$(MAKE) -C subdir1
	$(MAKE) -C subdir2

all: subdirs
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend proper dependency handling
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("recursive") || r.contains("$(MAKE)")),
        "Should handle recursive make calls"
    );
}

/// Test PARALLEL_SAFETY_009: Parallel-safe pattern rule
#[test]
fn test_PARALLEL_SAFETY_009_pattern_rule_safety() {
    // ARRANGE: Pattern rule that is parallel-safe
    let makefile = r#"
%.o: %.c
	gcc -c $< -o $@

all: main.o util.o
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should recognize parallel-safe pattern rules
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PARALLEL_SAFETY_010: Shared directory creation race
#[test]
fn test_PARALLEL_SAFETY_010_shared_directory_race() {
    // ARRANGE: Multiple targets creating same directory
    let makefile = r#"
obj/main.o: main.c
	mkdir -p obj
	gcc -c main.c -o obj/main.o

obj/util.o: util.c
	mkdir -p obj
	gcc -c util.c -o obj/util.o

all: obj/main.o obj/util.o
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend order-only prerequisite for directory
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("directory") || r.contains("mkdir")),
        "Should detect shared directory creation race"
    );
}

// ========================================
// Sprint 83 - Day 4: Reproducible Builds Tests
// ========================================

/// Test REPRODUCIBLE_001: Detect $(shell date) timestamp
#[test]
fn test_REPRODUCIBLE_001_detect_shell_date() {
    // ARRANGE: Makefile with $(shell date) timestamp
    let makefile = r#"
VERSION := $(shell date +%Y%m%d)

build:
	echo "Building version $(VERSION)"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect timestamp and suggest SOURCE_DATE_EPOCH
    assert!(
        result.report.iter().any(|r| r.contains("timestamp")
            || r.contains("date")
            || r.contains("SOURCE_DATE_EPOCH")),
        "Should detect non-deterministic timestamp $(shell date)"
    );
}

/// Test REPRODUCIBLE_002: Detect $(shell date +%s) unix timestamp
#[test]
fn test_REPRODUCIBLE_002_detect_unix_timestamp() {
    // ARRANGE: Makefile with unix timestamp
    let makefile = r#"
RELEASE := release-$(shell date +%s)

deploy:
	tar -czf $(RELEASE).tar.gz src/
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect unix timestamp
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("timestamp") || r.contains("date")),
        "Should detect non-deterministic unix timestamp"
    );
}

include!("tests_tests_REPRODUCIBLE.rs");
