fn test_phony_target_marked() {
    let input = ".PHONY: build test\nbuild:\n\tcargo build\ntest:\n\tcargo test";
    let ast = parse_makefile(input).unwrap();
    for item in &ast.items {
        if let MakeItem::Target { name, phony, .. } = item {
            if name == "build" || name == "test" {
                assert!(*phony, "Target {} should be marked phony", name);
            }
        }
    }
}

#[test]
fn test_phony_non_matching_not_marked() {
    let input = ".PHONY: clean\nbuild:\n\tcargo build";
    let ast = parse_makefile(input).unwrap();
    for item in &ast.items {
        if let MakeItem::Target { name, phony, .. } = item {
            if name == "build" {
                assert!(!*phony, "build should NOT be phony");
            }
        }
    }
}

// === Target rule edge cases ===

#[test]
fn test_target_with_prerequisites() {
    let input = "build: src/main.c src/util.c\n\tgcc -o build $^";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target { prerequisites, .. } => {
            assert_eq!(prerequisites.len(), 2);
            assert_eq!(prerequisites[0], "src/main.c");
            assert_eq!(prerequisites[1], "src/util.c");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_target_multiple_recipe_lines() {
    let input = "build:\n\techo step1\n\techo step2\n\techo step3";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3);
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_target_recipe_with_empty_line_between() {
    // Empty line between recipe lines should end the recipe
    // unless the next line is also tab-indented
    let input = "build:\n\techo step1\n\n\techo step2";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            // Both recipe lines should be captured because
            // empty line followed by another tab-indented line continues
            assert_eq!(recipe.len(), 2);
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_target_recipe_ends_at_non_tab() {
    let input = "build:\n\techo step1\nCC = gcc";
    let ast = parse_makefile(input).unwrap();
    assert_eq!(ast.items.len(), 2);
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1);
        }
        other => panic!("Expected Target, got {:?}", other),
    }
    assert!(matches!(ast.items[1], MakeItem::Variable { .. }));
}

#[test]
fn test_empty_target_name_error() {
    let input = ": deps\n\tcommand";
    let result = parse_makefile(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Empty target name"));
}

// === Variable assignment variants ===

#[test]
fn test_variable_simple_assign() {
    let input = "CC := gcc";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Simple);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_conditional_assign() {
    let input = "CC ?= gcc";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Conditional);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_append_assign() {
    let input = "CFLAGS += -Wall";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Append);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_shell_assign() {
    let input = "DATE != date +%Y";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Variable { flavor, .. } => {
            assert_eq!(*flavor, VarFlavor::Shell);
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_variable_empty_name_error() {
    // Direct parse_makefile won't trigger this easily since
    // " = value" would be filtered differently. Use "= value" directly
    // via an indirect approach: the parser treats "= val" as variable
    let input = " = value";
    let result = parse_makefile(input);
    assert!(result.is_err());
}

// === Include directive variants ===

#[test]
fn test_include_mandatory() {
    let input = "include common.mk";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "common.mk");
            assert!(!optional);
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_include_optional_dash() {
    let input = "-include optional.mk";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Include { optional, .. } => {
            assert!(*optional);
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_include_sinclude() {
    let input = "sinclude optional.mk";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk");
            assert!(*optional);
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

// === Function extraction ===

#[test]
fn test_extract_function_calls_wildcard() {
    let calls = extract_function_calls("SOURCES := $(wildcard src/*.c)");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "wildcard");
    assert_eq!(calls[0].1, "src/*.c");
}

#[test]
fn test_extract_function_calls_nested() {
    let calls = extract_function_calls("OBJS := $(patsubst %.c,%.o,$(SOURCES))");
    assert!(!calls.is_empty());
    assert_eq!(calls[0].0, "patsubst");
}

#[test]
fn test_extract_function_calls_no_args() {
    let calls = extract_function_calls("DIR := $(CURDIR)");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "CURDIR");
    assert_eq!(calls[0].1, "");
}

#[test]
fn test_extract_function_calls_multiple() {
    let calls = extract_function_calls("$(wildcard *.c) $(patsubst %.c,%.o,x)");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "wildcard");
    assert_eq!(calls[1].0, "patsubst");
}

#[test]
fn test_extract_function_calls_unmatched_paren() {
    // Unbalanced parens: $( without closing )
    let calls = extract_function_calls("$(unclosed");
    assert!(calls.is_empty());
}

#[test]
fn test_extract_function_calls_no_dollar() {
    let calls = extract_function_calls("no functions here");
    assert!(calls.is_empty());
}

// === Metadata line count ===

#[test]
fn test_metadata_line_count() {
    let input = "# comment\nCC = gcc\nbuild:\n\tcargo build";
    let ast = parse_makefile(input).unwrap();
    assert_eq!(ast.metadata.line_count, 4);
}

#[test]
fn test_metadata_empty_makefile() {
    let ast = parse_makefile("").unwrap();
    assert_eq!(ast.metadata.line_count, 0);
    assert_eq!(ast.items.len(), 0);
}

// === Complex full Makefile ===

#[test]
fn test_full_makefile_mixed() {
    let input = "\
# Build config
CC := gcc
CFLAGS ?= -O2

.PHONY: all clean

all: main.o util.o
\t$(CC) -o app main.o util.o

%.o: %.c
\t$(CC) $(CFLAGS) -c $< -o $@

clean:
\trm -f app *.o

ifdef DEBUG
CFLAGS += -g
endif
";
    let ast = parse_makefile(input).unwrap();
    // Should parse comment, variables, .PHONY, targets, pattern rule, conditional
    assert!(ast.items.len() >= 6);

    // Verify phony targets
    let mut found_all_phony = false;
    let mut found_clean_phony = false;
    for item in &ast.items {
        if let MakeItem::Target { name, phony, .. } = item {
            if name == "all" {
                found_all_phony = *phony;
            }
            if name == "clean" {
                found_clean_phony = *phony;
            }
        }
    }
    assert!(found_all_phony, "all should be phony");
    assert!(found_clean_phony, "clean should be phony");
}

#[test]
fn test_variable_with_colon_after_equals() {
    // "target: VAR=value" should be parsed as target, not variable
    // because colon comes before equals
    let input = "target: dep\n\techo done";
    let ast = parse_makefile(input).unwrap();
    assert!(matches!(ast.items[0], MakeItem::Target { .. }));
}

#[test]
fn test_conditional_else_with_empty_lines() {
    let input = "ifdef FOO\n\nX = 1\nelse\n\nY = 2\nendif";
    let ast = parse_makefile(input).unwrap();
    match &ast.items[0] {
        MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } => {
            assert!(!then_items.is_empty());
            assert!(else_items.is_some());
        }
        other => panic!("Expected Conditional, got {:?}", other),
    }
}
