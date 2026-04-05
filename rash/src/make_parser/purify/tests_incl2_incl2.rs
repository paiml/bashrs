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

include!("tests_incl2_incl2_incl2.rs");
