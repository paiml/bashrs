#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a basic span for tests
    fn test_span() -> Span {
        Span::dummy()
    }

    // ====== Variable Generation Tests ======

    #[test]
    fn test_generate_variable_simple() {
        let output = generate_variable("CC", "gcc", &VarFlavor::Simple);
        assert_eq!(output, "CC := gcc");
    }

    #[test]
    fn test_generate_variable_recursive() {
        let output = generate_variable("VAR", "$(OTHER)", &VarFlavor::Recursive);
        assert_eq!(output, "VAR = $(OTHER)");
    }

    #[test]
    fn test_generate_variable_conditional() {
        let output = generate_variable("VAR", "default", &VarFlavor::Conditional);
        assert_eq!(output, "VAR ?= default");
    }

    #[test]
    fn test_generate_variable_append() {
        let output = generate_variable("CFLAGS", "-O2", &VarFlavor::Append);
        assert_eq!(output, "CFLAGS += -O2");
    }

    #[test]
    fn test_generate_variable_shell() {
        let output = generate_variable("DATE", "date +%Y", &VarFlavor::Shell);
        assert_eq!(output, "DATE != date +%Y");
    }

    // ====== Comment Generation Tests ======

    #[test]
    fn test_generate_comment_simple() {
        let output = generate_comment("This is a comment");
        assert_eq!(output, "# This is a comment");
    }

    #[test]
    fn test_generate_comment_empty() {
        let output = generate_comment("");
        assert_eq!(output, "# ");
    }

    // ====== Include Generation Tests ======

    #[test]
    fn test_generate_include_required() {
        let output = generate_include("config.mk", false);
        assert_eq!(output, "include config.mk");
    }

    #[test]
    fn test_generate_include_optional() {
        let output = generate_include("config.mk", true);
        assert_eq!(output, "-include config.mk");
    }

    // ====== Target Generation Tests ======

    #[test]
    fn test_generate_target_simple() {
        let options = MakefileGeneratorOptions::default();
        let output = generate_target(
            "build",
            &["main.c".to_string()],
            &["gcc -o build main.c".to_string()],
            false,
            None,
            &options,
        );
        assert!(output.contains("build: main.c"));
        assert!(output.contains("\tgcc -o build main.c"));
    }

    #[test]
    fn test_generate_target_phony() {
        let options = MakefileGeneratorOptions::default();
        let output = generate_target(
            "clean",
            &[],
            &["rm -rf *.o".to_string()],
            true,
            None,
            &options,
        );
        assert!(output.contains(".PHONY: clean"));
        assert!(output.contains("clean:"));
    }

    #[test]
    fn test_generate_target_no_recipe() {
        let options = MakefileGeneratorOptions::default();
        let output = generate_target(
            "all",
            &["build".to_string(), "test".to_string()],
            &[],
            false,
            None,
            &options,
        );
        assert!(output.contains("all: build test"));
    }

    #[test]
    fn test_generate_target_with_metadata_preserve_formatting() {
        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            ..Default::default()
        };
        let metadata = RecipeMetadata {
            line_breaks: vec![(10, "\t".to_string())],
        };
        let output = generate_target(
            "build",
            &[],
            &["gcc -o out file.c".to_string()],
            false,
            Some(&metadata),
            &options,
        );
        assert!(output.contains("build:"));
    }

    // ====== Conditional Generation Tests ======

    #[test]
    fn test_generate_conditional_ifeq() {
        let options = MakefileGeneratorOptions::default();
        let condition = MakeCondition::IfEq("$(DEBUG)".to_string(), "yes".to_string());
        let then_items = vec![MakeItem::Variable {
            name: "CFLAGS".to_string(),
            value: "-g".to_string(),
            flavor: VarFlavor::Append,
            span: test_span(),
        }];

        let output = generate_conditional(&condition, &then_items, None, &options);
        assert!(output.contains("ifeq ($(DEBUG),yes)"));
        assert!(output.contains("CFLAGS += -g"));
        assert!(output.contains("endif"));
    }

    #[test]
    fn test_generate_conditional_ifneq() {
        let options = MakefileGeneratorOptions::default();
        let condition = MakeCondition::IfNeq("$(CC)".to_string(), "clang".to_string());
        let then_items = vec![];

        let output = generate_conditional(&condition, &then_items, None, &options);
        assert!(output.contains("ifneq ($(CC),clang)"));
        assert!(output.contains("endif"));
    }

    #[test]
    fn test_generate_conditional_ifdef() {
        let options = MakefileGeneratorOptions::default();
        let condition = MakeCondition::IfDef("DEBUG".to_string());
        let then_items = vec![];

        let output = generate_conditional(&condition, &then_items, None, &options);
        assert!(output.contains("ifdef DEBUG"));
        assert!(output.contains("endif"));
    }

    #[test]
    fn test_generate_conditional_ifndef() {
        let options = MakefileGeneratorOptions::default();
        let condition = MakeCondition::IfNdef("RELEASE".to_string());
        let then_items = vec![];

        let output = generate_conditional(&condition, &then_items, None, &options);
        assert!(output.contains("ifndef RELEASE"));
        assert!(output.contains("endif"));
    }

    #[test]
    fn test_generate_conditional_with_else() {
        let options = MakefileGeneratorOptions::default();
        let condition = MakeCondition::IfDef("DEBUG".to_string());
        let then_items = vec![MakeItem::Variable {
            name: "OPT".to_string(),
            value: "-O0".to_string(),
            flavor: VarFlavor::Simple,
            span: test_span(),
        }];
        let else_items = vec![MakeItem::Variable {
            name: "OPT".to_string(),
            value: "-O3".to_string(),
            flavor: VarFlavor::Simple,
            span: test_span(),
        }];

        let output = generate_conditional(&condition, &then_items, Some(&else_items), &options);
        assert!(output.contains("ifdef DEBUG"));
        assert!(output.contains("OPT := -O0"));
        assert!(output.contains("else"));
        assert!(output.contains("OPT := -O3"));
        assert!(output.contains("endif"));
    }

    // ====== Line Length Limit Tests ======

    #[test]
    fn test_apply_line_length_limit_short_line() {
        let output = apply_line_length_limit("short line", 80);
        assert_eq!(output, "short line");
    }

    #[test]
    fn test_apply_line_length_limit_long_line() {
        let long_line =
            "gcc -o output file1.c file2.c file3.c file4.c file5.c file6.c file7.c file8.c";
        let output = apply_line_length_limit(long_line, 40);
        assert!(
            output.contains("\\"),
            "Long lines should be broken with backslash"
        );
    }

    #[test]
    fn test_apply_line_length_limit_preserves_tabs() {
        let recipe = "\tgcc -o output file1.c file2.c file3.c file4.c file5.c";
        let output = apply_line_length_limit(recipe, 30);
        assert!(output.starts_with("\t"), "Should preserve leading tab");
    }

    // ====== MakefileGeneratorOptions Tests ======

    #[test]
    fn test_options_default() {
        let options = MakefileGeneratorOptions::default();
        assert!(!options.preserve_formatting);
        assert!(options.max_line_length.is_none());
        assert!(!options.skip_blank_line_removal);
        assert!(!options.skip_consolidation);
    }

    // ====== Full AST Generation Tests ======

    #[test]
    fn test_generate_purified_makefile_empty() {
        let ast = MakeAst {
            items: vec![],
            metadata: MakeMetadata::new(),
        };
        let output = generate_purified_makefile(&ast);
        assert!(output.is_empty());
    }

    #[test]
    fn test_generate_purified_makefile_simple() {
        let ast = MakeAst {
            items: vec![
                MakeItem::Variable {
                    name: "CC".to_string(),
                    value: "gcc".to_string(),
                    flavor: VarFlavor::Simple,
                    span: test_span(),
                },
                MakeItem::Target {
                    name: "all".to_string(),
                    prerequisites: vec!["build".to_string()],
                    recipe: vec![],
                    phony: true,
                    recipe_metadata: None,
                    span: test_span(),
                },
            ],
            metadata: MakeMetadata::new(),
        };
        let output = generate_purified_makefile(&ast);
        assert!(output.contains("CC := gcc"));
        assert!(output.contains(".PHONY: all"));
        assert!(output.contains("all: build"));
    }

    #[test]
    fn test_generate_purified_makefile_with_formatting_options() {
        let ast = MakeAst {
            items: vec![
                MakeItem::Comment {
                    text: "This is a comment".to_string(),
                    span: test_span(),
                },
                MakeItem::Target {
                    name: "build".to_string(),
                    prerequisites: vec![],
                    recipe: vec!["echo building".to_string()],
                    phony: false,
                    recipe_metadata: None,
                    span: test_span(),
                },
            ],
            metadata: MakeMetadata::new(),
        };
        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            max_line_length: Some(100),
            skip_blank_line_removal: true,
            skip_consolidation: true,
        };
        let output = generate_purified_makefile_with_options(&ast, &options);
        assert!(output.contains("# This is a comment"));
        assert!(output.contains("build:"));
    }

    #[test]
    fn test_generate_item_function_call() {
        let options = MakefileGeneratorOptions::default();
        let item = MakeItem::FunctionCall {
            name: "shell".to_string(),
            args: vec!["date".to_string(), "+%Y".to_string()],
            span: test_span(),
        };
        let output = generate_item(&item, &options);
        assert_eq!(output, "$(shell date,+%Y)");
    }

    // ====== Pattern Rule Tests ======

    #[test]
    fn test_generate_pattern_rule() {
        let options = MakefileGeneratorOptions::default();
        let output = generate_pattern_rule(
            "%.o",
            &["%.c".to_string()],
            &["$(CC) -c $< -o $@".to_string()],
            None,
            &options,
        );
        assert!(output.contains("%.o: %.c"));
        assert!(output.contains("\t$(CC) -c $< -o $@"));
    }

    // ====== Recipe Line Reconstruction Tests ======

    #[test]
    fn test_reconstruct_recipe_line_with_breaks_no_breaks() {
        let metadata = RecipeMetadata {
            line_breaks: vec![],
        };
        let output = reconstruct_recipe_line_with_breaks("echo hello", &metadata);
        assert_eq!(output, "\techo hello\n");
    }

    #[test]
    fn test_reconstruct_recipe_line_with_breaks_single_break() {
        let metadata = RecipeMetadata {
            line_breaks: vec![(5, "\t\t".to_string())],
        };
        let output = reconstruct_recipe_line_with_breaks("echo hello world", &metadata);
        // Should insert line break at position 5
        assert!(output.contains("\\"), "Should contain line continuation");
    }

    // ====== Blank Line Preservation Tests ======

    #[test]
    fn test_should_preserve_blank_line_no_prev() {
        let options = MakefileGeneratorOptions::default();
        let item = MakeItem::Comment {
            text: "test".to_string(),
            span: test_span(),
        };
        assert!(!should_preserve_blank_line(&item, false, false, &options));
    }

    #[test]
    fn test_should_preserve_blank_line_with_preserve_formatting() {
        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            ..Default::default()
        };
        let target = MakeItem::Target {
            name: "test".to_string(),
            prerequisites: vec![],
            recipe: vec![],
            phony: false,
            recipe_metadata: None,
            span: test_span(),
        };
        assert!(should_preserve_blank_line(&target, true, false, &options));
    }

    #[test]
    fn test_should_preserve_blank_line_comment_after_comment() {
        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            ..Default::default()
        };
        let comment = MakeItem::Comment {
            text: "test".to_string(),
            span: test_span(),
        };
        // Comment after comment should NOT add blank line
        assert!(!should_preserve_blank_line(&comment, true, true, &options));
    }

    #[test]
    fn test_should_preserve_blank_line_comment_after_non_comment() {
        let options = MakefileGeneratorOptions {
            preserve_formatting: true,
            ..Default::default()
        };
        let comment = MakeItem::Comment {
            text: "test".to_string(),
            span: test_span(),
        };
        // Comment after non-comment should add blank line
        assert!(should_preserve_blank_line(&comment, true, false, &options));
    }
}
