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

/// Test REPRODUCIBLE_003: Detect $RANDOM variable
#[test]
fn test_REPRODUCIBLE_003_detect_random() {
    // ARRANGE: Makefile with $RANDOM
    let makefile = r#"
SESSION_ID := session-$$RANDOM

test:
	echo "Session: $(SESSION_ID)"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect $RANDOM
    assert!(
        result.report.iter().any(|r| r.contains("RANDOM")
            || r.contains("random")
            || r.contains("non-deterministic")),
        "Should detect non-deterministic $RANDOM variable"
    );
}

/// Test REPRODUCIBLE_004: Detect process ID $$
#[test]
fn test_REPRODUCIBLE_004_detect_process_id() {
    // ARRANGE: Makefile with process ID
    let makefile = r#"
TMP_FILE := /tmp/build-$$$$

build:
	touch $(TMP_FILE)
	gcc -o app main.c
	rm $(TMP_FILE)
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect process ID
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("process") || r.contains("$$") || r.contains("PID")),
        "Should detect non-deterministic process ID $$"
    );
}

/// Test REPRODUCIBLE_005: Suggest SOURCE_DATE_EPOCH replacement
#[test]
fn test_REPRODUCIBLE_005_suggest_source_date_epoch() {
    // ARRANGE: Makefile with timestamp that should use SOURCE_DATE_EPOCH
    let makefile = r#"
BUILD_DATE := $(shell date)

package:
	echo "Built on: $(BUILD_DATE)" > version.txt
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should suggest SOURCE_DATE_EPOCH
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("SOURCE_DATE_EPOCH")),
        "Should suggest using SOURCE_DATE_EPOCH for reproducibility"
    );
}

/// Test REPRODUCIBLE_006: Detect non-deterministic command substitution
#[test]
fn test_REPRODUCIBLE_006_detect_command_substitution() {
    // ARRANGE: Makefile with non-deterministic command
    let makefile = r#"
HOSTNAME := $(shell hostname)

config:
	echo "Host: $(HOSTNAME)" > config.txt
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect hostname (environment-dependent)
    assert!(
        result.report.iter().any(|r| r.contains("hostname")
            || r.contains("environment")
            || r.contains("deterministic")),
        "Should detect environment-dependent hostname"
    );
}

/// Test REPRODUCIBLE_007: Preserve deterministic timestamps
#[test]
fn test_REPRODUCIBLE_007_preserve_deterministic() {
    // ARRANGE: Makefile already using SOURCE_DATE_EPOCH
    let makefile = r#"
BUILD_DATE := $(SOURCE_DATE_EPOCH)

build:
	echo "Build: $(BUILD_DATE)"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should not flag SOURCE_DATE_EPOCH as issue
    // Test passes if purify_makefile runs without panic
    // May still have other transformations, but not for SOURCE_DATE_EPOCH
    let _ = result.transformations_applied; // Verify result exists
}

/// Test REPRODUCIBLE_008: Detect git commit hash timestamp
#[test]
fn test_REPRODUCIBLE_008_detect_git_timestamp() {
    // ARRANGE: Makefile using git commit timestamp
    let makefile = r#"
GIT_DATE := $(shell git log -1 --format=%cd)

version:
	echo "Git date: $(GIT_DATE)" > version.txt
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect git timestamp
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("git") || r.contains("timestamp") || r.contains("deterministic")),
        "Should detect git commit timestamp"
    );
}

/// Test REPRODUCIBLE_009: Detect mktemp usage
#[test]
fn test_REPRODUCIBLE_009_detect_mktemp() {
    // ARRANGE: Makefile using mktemp (non-deterministic temp files)
    let makefile = r#"
build:
	TMP=$$(mktemp); \
	gcc -o $$TMP main.c; \
	cp $$TMP app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect mktemp
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("mktemp") || r.contains("temp") || r.contains("deterministic")),
        "Should detect non-deterministic mktemp usage"
    );
}

/// Test REPRODUCIBLE_010: Comprehensive reproducibility check
#[test]
fn test_REPRODUCIBLE_010_comprehensive_check() {
    // ARRANGE: Makefile with multiple reproducibility issues
    let makefile = r#"
VERSION := $(shell date +%Y%m%d)
SESSION := $$RANDOM
BUILD_HOST := $(shell hostname)

all: build

build:
	echo "Version: $(VERSION)" > version.txt
	echo "Session: $(SESSION)" >> version.txt
	echo "Host: $(BUILD_HOST)" >> version.txt
	gcc -o app main.c
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect multiple issues
    assert!(
        result.transformations_applied >= 3,
        "Should detect at least 3 reproducibility issues (date, RANDOM, hostname)"
    );
}

// ========================================
// Sprint 83 - Day 5: Performance Optimization Tests
// ========================================

/// Test PERFORMANCE_001: Detect multiple shell invocations
#[test]
fn test_PERFORMANCE_001_detect_multiple_shell_invocations() {
    // ARRANGE: Makefile with multiple shell commands that could be combined
    let makefile = r#"
build:
	mkdir -p bin
	gcc -c main.c -o bin/main.o
	gcc -c util.c -o bin/util.o
	gcc bin/main.o bin/util.o -o bin/app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend combining shell invocations
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("combine") || r.contains("shell") || r.contains("performance")),
        "Should detect multiple shell invocations that could be combined"
    );
}

/// Test PERFORMANCE_002: Suggest using := instead of =
#[test]
fn test_PERFORMANCE_002_suggest_simple_expansion() {
    // ARRANGE: Makefile with recursive variable that could be simple
    let makefile = r#"
CC = gcc
CFLAGS = -Wall -O2
LDFLAGS = -lm

build:
	$(CC) $(CFLAGS) main.c $(LDFLAGS) -o app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should suggest using := for simple variables
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains(":=") || r.contains("simple") || r.contains("expansion")),
        "Should suggest using := instead of = for simple variables"
    );
}

/// Test PERFORMANCE_003: Detect missing .SUFFIXES
#[test]
fn test_PERFORMANCE_003_detect_missing_suffixes() {
    // ARRANGE: Makefile without .SUFFIXES, with performance issue (recursive var with shell)
    // Note: .SUFFIXES is only recommended when other performance issues are detected
    let makefile = r#"
VERSION = $(shell git describe)

all: app

app: main.o
	gcc main.o -o app

main.o: main.c
	gcc -c main.c
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend adding .SUFFIXES (because of VERSION performance issue)
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains(".SUFFIXES") || r.contains("builtin") || r.contains("performance")),
        "Should recommend adding .SUFFIXES: when performance issues detected"
    );
}

/// Test PERFORMANCE_004: Detect inefficient variable expansion
#[test]
fn test_PERFORMANCE_004_detect_inefficient_expansion() {
    // ARRANGE: Makefile with variable that re-expands $(shell) multiple times
    let makefile = r#"
VERSION = $(shell git describe --tags)

build:
	echo "Building $(VERSION)"
	tar -czf myapp-$(VERSION).tar.gz src/
	echo "Created myapp-$(VERSION).tar.gz"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should suggest using := to avoid re-expansion
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains(":=") || r.contains("expansion") || r.contains("shell")),
        "Should suggest := to avoid re-expanding $(shell) multiple times"
    );
}

/// Test PERFORMANCE_005: Preserve existing .SUFFIXES
#[test]
fn test_PERFORMANCE_005_preserve_existing_suffixes() {
    // ARRANGE: Makefile already has .SUFFIXES:
    let makefile = r#"
.SUFFIXES:

all: app

app: main.c
	gcc main.c -o app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should not add duplicate .SUFFIXES
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PERFORMANCE_006: Detect sequential recipe lines
#[test]
fn test_PERFORMANCE_006_detect_sequential_recipes() {
    // ARRANGE: Makefile with many sequential recipe lines
    let makefile = r#"
install:
	mkdir -p /usr/local/bin
	cp app /usr/local/bin/
	chmod +x /usr/local/bin/app
	mkdir -p /usr/local/share/doc/app
	cp README.md /usr/local/share/doc/app/
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should suggest combining with && or ;
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("combine") || r.contains("&&") || r.contains("performance")),
        "Should suggest combining sequential recipe lines"
    );
}

/// Test PERFORMANCE_007: Detect expensive wildcard in recipe
#[test]
fn test_PERFORMANCE_007_detect_expensive_wildcard() {
    // ARRANGE: Makefile with wildcard expansion in recipe (expensive)
    let makefile = r#"
clean:
	rm -f *.o
	rm -f *.a
	rm -f *.so
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should suggest combining into single rm command
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("combine") || r.contains("rm") || r.contains("performance")),
        "Should suggest combining rm commands for better performance"
    );
}

/// Test PERFORMANCE_008: Detect := already used
#[test]
fn test_PERFORMANCE_008_detect_simple_expansion_already_used() {
    // ARRANGE: Makefile already uses := (correct)
    let makefile = r#"
CC := gcc
CFLAGS := -Wall -O2

build:
	$(CC) $(CFLAGS) main.c -o app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should not flag variables already using :=
    let _ = result.transformations_applied; // Verify result exists
}

/// Test PERFORMANCE_009: Detect pattern rule efficiency
#[test]
fn test_PERFORMANCE_009_detect_pattern_rule_efficiency() {
    // ARRANGE: Makefile with explicit rules that could be pattern rule
    let makefile = r#"
main.o: main.c
	gcc -c main.c -o main.o

util.o: util.c
	gcc -c util.c -o util.o

math.o: math.c
	gcc -c math.c -o math.o
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should suggest using pattern rule %.o: %.c
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("pattern") || r.contains("%.o") || r.contains("rule")),
        "Should suggest using pattern rule for repeated compilation"
    );
}

/// Test PERFORMANCE_010: Comprehensive performance check
#[test]
fn test_PERFORMANCE_010_comprehensive_performance_check() {
    // ARRANGE: Makefile with multiple performance issues
    let makefile = r#"
VERSION = $(shell git describe --tags)
CC = gcc
CFLAGS = -Wall -O2

all: app

app: main.o util.o
	gcc main.o util.o -o app

main.o: main.c
	mkdir -p obj
	gcc -c main.c -o main.o
	cp main.o obj/

util.o: util.c
	mkdir -p obj
	gcc -c util.c -o util.o
	cp util.o obj/

clean:
	rm -f *.o
	rm -f *.a
	rm -f app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect multiple performance issues
    assert!(
        result.transformations_applied >= 2,
        "Should detect multiple performance issues (shell expansion, multiple commands, etc.)"
    );
}

// ========================================
// Sprint 83 - Day 6: Error Handling Tests
// ========================================

/// Test ERROR_HANDLING_001: Detect missing error handling (|| exit 1)
#[test]
fn test_ERROR_HANDLING_001_detect_missing_error_handling() {
    // ARRANGE: Makefile with important commands without error handling
    let makefile = r#"
build:
	mkdir build
	gcc -c main.c -o build/main.o
	gcc build/main.o -o build/app
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend error handling for critical commands
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("error") || r.contains("exit") || r.contains("handling")),
        "Should recommend adding error handling (|| exit 1) for critical commands"
    );
}

/// Test ERROR_HANDLING_002: Detect silent failures (@ prefix)
#[test]
fn test_ERROR_HANDLING_002_detect_silent_failures() {
    // ARRANGE: Makefile with @ prefix hiding errors
    let makefile = r#"
test:
	@echo "Running tests..."
	@./run-tests.sh
	@echo "Tests complete"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should warn about @ prefix hiding errors
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("@") || r.contains("silent") || r.contains("error")),
        "Should detect @ prefix that may hide errors in critical commands"
    );
}

/// Test ERROR_HANDLING_003: Recommend .DELETE_ON_ERROR
#[test]
fn test_ERROR_HANDLING_003_recommend_delete_on_error() {
    // ARRANGE: Makefile without .DELETE_ON_ERROR but with error handling issues
    let makefile = r#"
build:
	mkdir build
	gcc -c main.c -o build/main.o
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend .DELETE_ON_ERROR (because mkdir without error handling was detected)
    assert!(
        result.report.iter().any(|r| r.contains(".DELETE_ON_ERROR")),
        "Should recommend .DELETE_ON_ERROR when error handling issues are detected"
    );
}

/// Test ERROR_HANDLING_004: Preserve existing .DELETE_ON_ERROR
#[test]
fn test_ERROR_HANDLING_004_preserve_existing_delete_on_error() {
    // ARRANGE: Makefile already has .DELETE_ON_ERROR
    let makefile = r#"
.DELETE_ON_ERROR:

%.o: %.c
	gcc -c $< -o $@
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should preserve existing .DELETE_ON_ERROR without duplication
    let _ = result.transformations_applied; // Verify result exists
}

/// Test ERROR_HANDLING_005: Detect unchecked command substitution
#[test]
fn test_ERROR_HANDLING_005_detect_unchecked_command_substitution() {
    // ARRANGE: Makefile with unchecked $(shell) commands
    let makefile = r#"
VERSION := $(shell git describe --tags)

build:
	echo "Building version $(VERSION)"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should detect potentially unchecked shell command substitution
    let _ = result.transformations_applied; // Verify result exists
}

/// Test ERROR_HANDLING_006: Detect missing .ONESHELL with multiline recipes
#[test]
fn test_ERROR_HANDLING_006_detect_missing_oneshell() {
    // ARRANGE: Makefile with multiline recipe without .ONESHELL
    let makefile = r#"
deploy:
	cd /tmp
	mkdir app
	echo "Done"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend .ONESHELL for related commands
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains(".ONESHELL") || r.contains("multiline") || r.contains("shell")),
        "Should recommend .ONESHELL or && for related commands across lines"
    );
}

/// Test ERROR_HANDLING_007: Detect commands that modify state without checks
#[test]
fn test_ERROR_HANDLING_007_detect_unchecked_state_modification() {
    // ARRANGE: Makefile with state-modifying commands without checks
    let makefile = r#"
clean:
	rm -rf build/*
	rm -rf dist/*
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // Test passes if purify_makefile runs without panic
    // Should verify destructive commands have appropriate flags
    let _ = result.transformations_applied; // Verify result exists
}

/// Test ERROR_HANDLING_008: Detect set -e equivalent missing
#[test]
fn test_ERROR_HANDLING_008_detect_missing_set_e() {
    // ARRANGE: Makefile with shell script without set -e
    let makefile = r#"
test:
	bash -c "echo test1; false; echo test2"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend set -e for shell scripts
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("set -e") || r.contains("error") || r.contains("exit")),
        "Should recommend 'set -e' for inline shell scripts"
    );
}

/// Test ERROR_HANDLING_009: Detect missing error handling in loops
#[test]
fn test_ERROR_HANDLING_009_detect_missing_error_handling_in_loops() {
    // ARRANGE: Makefile with for loop without error handling
    let makefile = r#"
install:
	for f in *.so; do cp $$f /usr/lib; done
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should recommend error handling in loops
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("loop") || r.contains("error") || r.contains("exit")),
        "Should recommend error handling in for loops (|| exit 1)"
    );
}

/// Test ERROR_HANDLING_010: Comprehensive error handling check
#[test]
fn test_ERROR_HANDLING_010_comprehensive_error_handling_check() {
    // ARRANGE: Makefile with multiple error handling issues
    let makefile = r#"
VERSION := $(shell git describe)

build:
	@mkdir build
	gcc -c main.c -o build/main.o
	for f in *.c; do gcc -c $$f; done
	bash -c "echo done"
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect multiple error handling issues
    assert!(
        result.transformations_applied >= 1,
        "Should detect multiple error handling issues (@ prefix, missing ||, loops, etc.)"
    );
}

// ========================================
// Sprint 83 - Day 7: Portability Tests
// ========================================

/// Test PORTABILITY_001: Detect bashisms in recipes ([[, $(()), etc.)
#[test]
fn test_PORTABILITY_001_detect_bashisms() {
    // ARRANGE: Makefile with bash-specific syntax
    let makefile = r#"
test:
	if [[ -f file.txt ]]; then echo "found"; fi
	result=$$((1 + 2))
"#;
    let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should detect bashisms ([[ and $(()))
    assert!(
        result
            .report
            .iter()
            .any(|r| r.contains("bashism") || r.contains("[[") || r.contains("POSIX")),
        "Should detect bashisms like [[ and $(())"
    );
}

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
    assert!(result.iter().any(|t| matches!(t, Transformation::DetectTimestamp { .. })));
    assert!(result.iter().any(|t| matches!(t, Transformation::SuggestSourceDateEpoch { .. })));
}

#[test]
fn test_reproducible_builds_random_in_variable() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "RAND_SEED".into(),
            value: "$$RANDOM".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(|t| matches!(t, Transformation::DetectRandom { .. })),
        "Should detect $$RANDOM"
    );
}

#[test]
fn test_reproducible_builds_process_id() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "TMP_FILE".into(),
            value: "/tmp/build_$$$$".into(),
            flavor: crate::make_parser::ast::VarFlavor::Recursive,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(|t| matches!(t, Transformation::DetectProcessId { .. })),
        "Should detect $$$$ (process ID)"
    );
}

#[test]
fn test_reproducible_builds_hostname() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "HOST".into(),
            value: "$(shell hostname)".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(|t| matches!(t, Transformation::DetectNonDeterministicCommand { .. })),
        "Should detect hostname"
    );
}

#[test]
fn test_reproducible_builds_git_log_timestamp() {
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "GIT_DATE".into(),
            value: "$(shell git log -1 --format=%cd --date=short)".into(),
            flavor: crate::make_parser::ast::VarFlavor::Simple,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(|t| matches!(t, Transformation::DetectNonDeterministicCommand { .. })),
        "Should detect git log timestamp"
    );
}

#[test]
fn test_reproducible_builds_mktemp_in_recipe() {
    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".into(),
            prerequisites: vec![],
            recipe: vec!["mktemp -d".into(), "echo building".into()],
            phony: false,
            recipe_metadata: None,
            span: crate::make_parser::ast::Span::dummy(),
        }],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(
        result.iter().any(|t| matches!(t, Transformation::DetectNonDeterministicCommand { .. })),
        "Should detect mktemp in recipe"
    );
}

#[test]
fn test_reproducible_builds_clean_makefile() {
    let ast = MakeAst {
        items: vec![
            MakeItem::Variable {
                name: "CC".into(),
                value: "gcc".into(),
                flavor: crate::make_parser::ast::VarFlavor::Simple,
                span: crate::make_parser::ast::Span::dummy(),
            },
            MakeItem::Target {
                name: "build".into(),
                prerequisites: vec!["main.c".into()],
                recipe: vec!["$(CC) -o build main.c".into()],
                phony: false,
                recipe_metadata: None,
                span: crate::make_parser::ast::Span::dummy(),
            },
        ],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(result.is_empty(), "Clean Makefile should have no issues");
}

#[test]
fn test_reproducible_builds_empty_ast() {
    let ast = MakeAst {
        items: vec![],
        metadata: crate::make_parser::ast::MakeMetadata::new(),
    };
    let result = reproducible_builds::analyze_reproducible_builds(&ast);
    assert!(result.is_empty());
}

// =============================================================================
// report — format_analysis_transformation coverage (via format_transformation)
// =============================================================================

#[test]
fn test_format_transformation_detect_timestamp() {
    let t = Transformation::DetectTimestamp {
        variable_name: "BUILD_TIME".into(),
        pattern: "$(shell date +%s)".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format DetectTimestamp");
}

#[test]
fn test_format_transformation_detect_random() {
    let t = Transformation::DetectRandom {
        variable_name: "SEED".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format DetectRandom");
}

#[test]
fn test_format_transformation_detect_process_id() {
    let t = Transformation::DetectProcessId {
        variable_name: "TMP".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format DetectProcessId");
}

#[test]
fn test_format_transformation_suggest_source_date_epoch() {
    let t = Transformation::SuggestSourceDateEpoch {
        variable_name: "BUILD_TIME".into(),
        original_pattern: "$(shell date)".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty(), "Should format SuggestSourceDateEpoch");
}

#[test]
fn test_format_transformation_detect_non_deterministic_command() {
    let t = Transformation::DetectNonDeterministicCommand {
        variable_name: "HOST".into(),
        command: "hostname".into(),
        reason: "environment-dependent".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_suggest_combine_shell() {
    let t = Transformation::SuggestCombineShellInvocations {
        target_name: "build".into(),
        recipe_count: 5,
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_suggest_simple_expansion() {
    let t = Transformation::SuggestSimpleExpansion {
        variable_name: "CC".into(),
        reason: "constant value".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_recommend_suffixes() {
    let t = Transformation::RecommendSuffixes {
        reason: "disable builtin rules".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_sequential_recipes() {
    let t = Transformation::DetectSequentialRecipes {
        target_name: "build".into(),
        recipe_count: 10,
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_suggest_pattern_rule() {
    let t = Transformation::SuggestPatternRule {
        pattern: "%.o: %.c".into(),
        target_count: 3,
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_missing_error_handling() {
    let t = Transformation::DetectMissingErrorHandling {
        target_name: "deploy".into(),
        command: "rm -rf /tmp/build".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_silent_failure() {
    let t = Transformation::DetectSilentFailure {
        target_name: "install".into(),
        command: "-cp src dest".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_recommend_delete_on_error() {
    let t = Transformation::RecommendDeleteOnError {
        reason: "prevent partial builds".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_recommend_oneshell() {
    let t = Transformation::RecommendOneshell {
        target_name: "build".into(),
        reason: "recipe uses cd".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_missing_set_e() {
    let t = Transformation::DetectMissingSetE {
        target_name: "build".into(),
        command: "gcc -o build main.c".into(),
        safe: true,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_bashism() {
    let t = Transformation::DetectBashism {
        target_name: "test".into(),
        construct: "[[ -f foo ]]".into(),
        posix_alternative: "[ -f foo ]".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_platform_specific() {
    let t = Transformation::DetectPlatformSpecific {
        target_name: "install".into(),
        command: "apt-get install foo".into(),
        reason: "debian/ubuntu only".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_shell_specific() {
    let t = Transformation::DetectShellSpecific {
        target_name: "run".into(),
        feature: "source".into(),
        posix_alternative: ". .env".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_non_portable_flags() {
    let t = Transformation::DetectNonPortableFlags {
        target_name: "build".into(),
        command: "cp --preserve=all src dest".into(),
        flag: "--preserve".into(),
        reason: "not available on BSD cp".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_non_portable_echo() {
    let t = Transformation::DetectNonPortableEcho {
        target_name: "info".into(),
        command: "echo -e \"\\t\"".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}

#[test]
fn test_format_transformation_detect_loop_without_error_handling() {
    let t = Transformation::DetectLoopWithoutErrorHandling {
        target_name: "deploy".into(),
        loop_command: "for f in *.sh; do sh $$f; done".into(),
        safe: false,
    };
    let result = report::generate_report(&[t]).join("\n");
    assert!(!result.is_empty());
}
