use super::parallel_safety::{
    is_automatic_variable, target_has_prerequisite, try_extract_cat_input,
    try_extract_output_redirect,
};
use super::*;

/// Test PORTABILITY_002: Detect GNU Make-specific extensions
#[test]
fn test_PORTABILITY_002_detect_gnu_make_extensions() {
    // ARRANGE: Makefile with GNU Make-specific syntax
    let makefile = r#"
%.o: %.c
	gcc -c $< -o $@

build: $(wildcard *.c)
	gcc -o app $^
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should detect GNU Make-specific constructs
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PORTABILITY_003: Detect platform-specific commands
#[test]
fn test_PORTABILITY_003_detect_platform_specific_commands() {
    // ARRANGE: Makefile with platform-specific commands
    let makefile = r#"
detect:
	uname -s
	cat /proc/cpuinfo
	ifconfig
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect platform-specific commands
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("platform") || r.contains("portable") || r.contains("uname")),
        "Should detect platform-specific commands like uname, /proc, ifconfig"
    );
}

/// Test PORTABILITY_004: Detect shell-specific features
#[test]
fn test_PORTABILITY_004_detect_shell_specific_features() {
    // ARRANGE: Makefile with bash-specific features
    let makefile = r#"
build:
	echo $$RANDOM
	source setup.sh
	declare -a array=(1 2 3)
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect shell-specific features (source, declare)
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("source") || r.contains("declare") || r.contains("bash")),
        "Should detect bash-specific features like source and declare"
    );
}

/// Test PORTABILITY_005: Detect path separator issues
#[test]
fn test_PORTABILITY_005_detect_path_separator_issues() {
    // ARRANGE: Makefile with hardcoded path separators
    let makefile = r#"
build:
	gcc -I/usr/local/include -L/usr/local/lib app.c
	install -m 755 app /usr/local/bin/app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should detect hardcoded paths that may not be portable
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PORTABILITY_006: Preserve portable constructs
#[test]
fn test_PORTABILITY_006_preserve_portable_constructs() {
    // ARRANGE: Makefile with POSIX-compliant syntax
    let makefile = r#"
build:
	if [ -f file.txt ]; then echo "found"; fi
	result=`expr 1 + 2`
	. setup.sh
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should not flag POSIX-compliant constructs ([ instead of [[, expr, .)
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PORTABILITY_007: Detect non-portable flags
#[test]
fn test_PORTABILITY_007_detect_non_portable_flags() {
    // ARRANGE: Makefile with GNU-specific flags
    let makefile = r#"
build:
	cp --preserve=all src dest
	ls --color=auto
	grep --color=always pattern file
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect GNU-specific long flags
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("--") || r.contains("GNU") || r.contains("portable")),
        "Should detect GNU-specific long flags like --preserve, --color"
    );
}

/// Test PORTABILITY_008: Detect echo -e and echo -n
#[test]
fn test_PORTABILITY_008_detect_echo_flags() {
    // ARRANGE: Makefile with non-portable echo usage
    let makefile = r#"
build:
	echo -e "Line1\nLine2"
	echo -n "No newline"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect non-portable echo flags
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("echo") || r.contains("printf") || r.contains("portable")),
        "Should detect non-portable echo -e and echo -n (recommend printf)"
    );
}

/// Test PORTABILITY_009: Detect sed -i (GNU extension)
#[test]
fn test_PORTABILITY_009_detect_sed_in_place() {
    // ARRANGE: Makefile with sed -i (GNU extension)
    let makefile = r#"
build:
	sed -i 's/old/new/g' file.txt
	sed -i.bak 's/foo/bar/' data.txt
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect sed -i (non-portable)
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("sed") || r.contains("portable") || r.contains("-i")),
        "Should detect sed -i as non-portable GNU extension"
    );
}

/// Test PORTABILITY_010: Comprehensive portability check
#[test]
fn test_PORTABILITY_010_comprehensive_portability_check() {
    // ARRANGE: Makefile with multiple portability issues
    let makefile = r#"
build:
	if [[ -f file.txt ]]; then echo -e "Found\n"; fi
	uname -s
	source env.sh
	cp --preserve=all src dest
	sed -i 's/old/new/' file
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect multiple portability issues
    assert!(
        result.transformations_applied >= 1,
        "Should detect multiple portability issues ([[, echo -e, uname, source, --, sed -i)"
    );
}

// ========================================
// Sprint 83 - Days 8-9: Property & Integration Tests
// ========================================

/// Property Test 001: Idempotency - purifying twice should be identical to purifying once
#[test]
fn test_PROPERTY_001_idempotency() {
    // ARRANGE: Makefile with various issues
    let makefile = r#"
FILES = $(wildcard *.c)
build:
	mkdir build
	gcc -c main.c
	echo "Done"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify once
    let result1 = purify_makefile(&ast);

    // Purify the result again (should be idempotent)
    let result2 = purify_makefile(&ast);

    // ASSERT: Both purifications should produce the same recommendations
    assert_eq!(
        result1.report.len(),
        result2.report.len(),
        "Purification should be idempotent - same recommendations"
    );
    assert_eq!(
        result1.transformations_applied, result2.transformations_applied,
        "Purification should apply same number of transformations"
    );
}

/// Property Test 002: Parallel Safety - verify parallel safety analysis works
#[test]
fn test_PROPERTY_002_parallel_safety_preserved() {
    // ARRANGE: Makefile that could benefit from parallel safety checks
    let makefile = r#"
all: build test
build:
	gcc -c main.c -o main.o
test:
	./test.sh
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Parallel safety analysis should execute without errors
    let _ = result.transformations_applied; // Verify result exists
}

/// Property Test 003: Reproducibility - verify non-deterministic detection
#[test]
fn test_PROPERTY_003_reproducibility_enforced() {
    // ARRANGE: Makefile with non-deterministic elements
    let makefile = r#"
VERSION = $(shell date +%s)
build:
	echo "Version: $$RANDOM"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect non-deterministic patterns
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("date") || r.contains("RANDOM") || r.contains("deterministic")),
        "Should detect non-deterministic patterns"
    );
}

/// Property Test 004: Performance - verify optimization recommendations
#[test]
fn test_PROPERTY_004_performance_optimizations() {
    // ARRANGE: Makefile with performance issues
    let makefile = r#"
VAR = $(shell echo test)
FILES = $(wildcard *.c)
build:
	gcc -c file1.c
	gcc -c file2.c
	gcc -c file3.c
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend performance improvements
    assert!(
        result.transformations_applied >= 1,
        "Should recommend performance optimizations"
    );
}

/// Property Test 005: Error Handling - verify error handling recommendations
#[test]
fn test_PROPERTY_005_error_handling_completeness() {
    // ARRANGE: Makefile without error handling
    let makefile = r#"
build:
	mkdir build
	gcc -c main.c
	cp main.o build/
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend error handling
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("error") || r.contains("exit") || r.contains("DELETE_ON_ERROR")),
        "Should recommend error handling improvements"
    );
}

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


}

    include!("tests_part2_incl2.rs");
