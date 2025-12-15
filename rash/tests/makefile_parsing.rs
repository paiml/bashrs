#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! Integration tests for Makefile parsing workflows
//!
//! These tests verify end-to-end parsing scenarios with real-world Makefiles.
//!
//! Following SQLite principles: Test real-world usage patterns

use bashrs::make_parser::{parse_makefile, MakeItem};

// ============================================================================
// Real-World Makefile Tests
// ============================================================================

#[test]
fn integration_simple_rust_project_makefile() {
    // ARRANGE: Typical Rust project Makefile
    let makefile = r#"
# Rust project Makefile
CARGO = cargo

build:
	$(CARGO) build --release

test:
	@$(CARGO) test

clean:
	$(CARGO) clean

.PHONY: build test clean
"#;

    // ACT: Parse the Makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Failed to parse simple Rust Makefile");

    let ast = result.unwrap();

    // Verify structure
    let targets: Vec<_> = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Target { .. }))
        .collect();

    assert_eq!(
        targets.len(),
        4,
        "Expected 4 targets (build, test, clean, .PHONY)"
    );
}

#[test]
fn integration_makefile_with_line_continuations() {
    // ARRANGE: Makefile with line continuations
    let makefile = r#"
FILES = src/main.rs \
        src/lib.rs \
        src/parser.rs

build: $(FILES)
	cargo build
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Line continuations handled
    assert!(result.is_ok());

    let ast = result.unwrap();

    // Verify variable with continuation
    let var = ast
        .items
        .iter()
        .find(|item| matches!(item, MakeItem::Variable { name, .. } if name == "FILES"))
        .expect("FILES variable not found");

    if let MakeItem::Variable { value, .. } = var {
        // Line continuations should be preprocessed
        assert!(value.contains("src/main.rs"));
        assert!(value.contains("src/lib.rs"));
        assert!(value.contains("src/parser.rs"));
    }
}

#[test]
fn integration_makefile_with_all_variable_flavors() {
    // ARRANGE: Makefile using all 5 variable assignment operators
    let makefile = r#"
# All variable flavors
RECURSIVE = $(shell date)
SIMPLE := immediate
CONDITIONAL ?= default
APPEND += more
SHELL != echo "shell"

test:
	@echo "Testing variables"
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: All flavors parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // Count variables
    let var_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Variable { .. }))
        .count();

    assert_eq!(var_count, 5, "Expected 5 variables (one per flavor)");
}

#[test]
fn integration_makefile_with_silent_recipes() {
    // ARRANGE: Makefile with @ prefix for silent execution
    let makefile = r#"
build:
	cargo build --release
	@echo "Build complete"
	@echo "Running tests..."
	cargo test
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: @ prefix preserved
    assert!(result.is_ok());

    let ast = result.unwrap();

    if let MakeItem::Target { recipe, .. } = &ast.items[0] {
        assert_eq!(recipe.len(), 4);

        // Verify @ prefix preserved
        assert!(
            !recipe[0].starts_with('@'),
            "First recipe should not be silent"
        );
        assert!(recipe[1].starts_with('@'), "Second recipe should be silent");
        assert!(recipe[2].starts_with('@'), "Third recipe should be silent");
        assert!(
            !recipe[3].starts_with('@'),
            "Fourth recipe should not be silent"
        );
    } else {
        panic!("Expected Target");
    }
}

#[test]
fn integration_complex_prerequisite_chains() {
    // ARRANGE: Makefile with dependency chains
    let makefile = r#"
all: build test

build: compile link

compile: main.o lib.o

link:
	ld -o program *.o

main.o: main.c
	cc -c main.c

lib.o: lib.c
	cc -c lib.c

test: build
	./program --test
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: All targets with prerequisites parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // Verify 'all' target has correct prerequisites
    let all_target = ast
        .items
        .iter()
        .find(|item| matches!(item, MakeItem::Target { name, .. } if name == "all"))
        .expect("'all' target not found");

    if let MakeItem::Target { prerequisites, .. } = all_target {
        assert_eq!(prerequisites.len(), 2);
        assert_eq!(prerequisites[0], "build");
        assert_eq!(prerequisites[1], "test");
    }
}

// ============================================================================
// GNU Make Manual Examples
// ============================================================================

#[test]
fn integration_gnu_make_manual_example_1() {
    // ARRANGE: From GNU Make manual Section 2.1
    let makefile = r#"
edit : main.o kbd.o command.o display.o
	cc -o edit main.o kbd.o command.o display.o

main.o : main.c defs.h
	cc -c main.c

kbd.o : kbd.c defs.h command.h
	cc -c kbd.c

clean :
	rm edit main.o kbd.o command.o display.o
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok());

    let ast = result.unwrap();

    // Verify target count
    let target_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Target { .. }))
        .count();

    assert_eq!(target_count, 4);
}

// ============================================================================
// Edge Cases and Error Recovery
// ============================================================================

#[test]
fn integration_makefile_with_comments_everywhere() {
    // ARRANGE: Comments in various positions
    let makefile = r#"
# Header comment
# Another header comment

# Variable comment
VAR = value # inline comment not supported by parser yet

# Target comment
target: # prerequisite comment
	# Recipe comment (tab-indented, should be ignored)
	echo "command"
	# Another recipe comment

# Footer comment
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Parses successfully (ignoring inline comments)
    assert!(result.is_ok());
}

#[test]
fn integration_makefile_with_empty_lines() {
    // ARRANGE: Makefile with many empty lines
    let makefile = r#"


VAR = value


target:

	echo "command"


"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Empty lines handled gracefully
    assert!(result.is_ok());

    let ast = result.unwrap();

    assert_eq!(ast.items.len(), 2); // 1 variable + 1 target
}

#[test]
fn integration_makefile_with_special_targets() {
    // ARRANGE: Makefile with special GNU Make targets
    let makefile = r#"
.PHONY: all clean test

.SUFFIXES:

.DEFAULT_GOAL := all

all: build

build:
	cargo build
"#;

    // ACT: Parse
    let result = parse_makefile(makefile);

    // ASSERT: Special targets parsed as regular targets
    assert!(result.is_ok());

    let ast = result.unwrap();

    // Verify .PHONY is parsed
    let phony_found = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".PHONY"));

    assert!(phony_found, ".PHONY target should be parsed");
}

// ============================================================================
// Performance and Scale
// ============================================================================

#[test]
fn integration_large_makefile_performance() {
    // ARRANGE: Generate a large Makefile
    let mut makefile = String::from("# Large Makefile\n\n");

    for i in 0..1000 {
        makefile.push_str(&format!("VAR_{} = value_{}\n", i, i));
    }

    for i in 0..1000 {
        makefile.push_str(&format!("target_{}:\n\techo {}\n\n", i, i));
    }

    // ACT: Parse (should complete quickly)
    let start = std::time::Instant::now();
    let result = parse_makefile(&makefile);
    let duration = start.elapsed();

    // ASSERT: Parses successfully and quickly
    assert!(result.is_ok(), "Failed to parse large Makefile");
    assert!(
        duration.as_millis() < 100,
        "Parsing took too long: {:?}",
        duration
    );

    let ast = result.unwrap();
    assert!(ast.items.len() >= 2000); // 1000 vars + 1000 targets
}
