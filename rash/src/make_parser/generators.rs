//! Code generators for Makefile AST
//!
//! Generates purified Makefiles from AST.
//!
//! This module implements the code generation phase of the purification pipeline:
//! Parse → AST → Analyze → Purify → **Generate** → Purified Makefile
//!
//! ## Design Principles
//!
//! 1. **Correctness**: Generated Makefiles must be syntactically valid
//! 2. **Determinism**: Same AST always produces same output
//! 3. **POSIX Compliance**: Generated output passes shellcheck
//! 4. **Readability**: Output is human-readable and maintainable
//!
//! ## Key Features
//!
//! - Variable generation with all 5 flavors (`:=`, `=`, `?=`, `+=`, `!=`)
//! - Target generation with tab-indented recipes (REQUIRED by Make)
//! - Comment preservation
//! - .PHONY target support
//! - Pattern rule support
//! - Conditional block support

use super::ast::*;

/// Options for controlling Makefile generation
///
/// These options allow users to customize the output format while maintaining
/// correctness and determinism.
///
/// ## Added in v6.34.0 (Dogfooding Follow-up)
///
/// See: docs/dogfooding/makefile-purification.md
#[derive(Debug, Clone, Default)]
pub struct MakefileGeneratorOptions {
    /// Preserve formatting (keep blank lines, multi-line format)
    pub preserve_formatting: bool,

    /// Maximum line length (None = unlimited)
    pub max_line_length: Option<usize>,

    /// Skip blank line removal transformation
    pub skip_blank_line_removal: bool,

    /// Skip multi-line consolidation transformation
    pub skip_consolidation: bool,
}

/// Generate a purified Makefile from an AST
///
/// This function emits a complete Makefile from a parsed and purified AST.
///
/// # Examples
///
/// ```ignore
/// use bashrs::make_parser::{MakeAst, MakeItem, MakeMetadata, VarFlavor, Span, generate_purified_makefile};
///
/// let ast = MakeAst {
///     items: vec![
///         MakeItem::Variable {
///             name: "CC".to_string(),
///             value: "gcc".to_string(),
///             flavor: VarFlavor::Simple,
///             span: Span::dummy(),
///         }
///     ],
///     metadata: MakeMetadata::new(),
/// };
///
/// let output = generate_purified_makefile(&ast);
/// assert_eq!(output.trim(), "CC := gcc");
/// ```
pub fn generate_purified_makefile(ast: &MakeAst) -> String {
    generate_purified_makefile_with_options(ast, &MakefileGeneratorOptions::default())
}

/// Generate a purified Makefile from an AST with custom formatting options
///
/// This function provides fine-grained control over the output format while
/// maintaining correctness and determinism.
///
/// ## Options
///
/// - `preserve_formatting`: Keep blank lines and multi-line format (combines skip flags)
/// - `max_line_length`: Break lines longer than this (None = unlimited)
/// - `skip_blank_line_removal`: Keep blank lines between sections
/// - `skip_consolidation`: Keep multi-line format (no single-line if/then/else)
///
/// ## Added in v6.34.0 (Dogfooding Follow-up)
///
/// See: docs/dogfooding/makefile-purification.md
///
/// # Examples
///
/// ```ignore
/// use bashrs::make_parser::{MakeAst, MakefileGeneratorOptions, generate_purified_makefile_with_options};
///
/// let ast = parse_makefile("...")?;
/// let options = MakefileGeneratorOptions {
///     preserve_formatting: true,
///     max_line_length: Some(120),
///     ..Default::default()
/// };
///
/// let output = generate_purified_makefile_with_options(&ast, &options);
/// ```
pub fn generate_purified_makefile_with_options(
    ast: &MakeAst,
    options: &MakefileGeneratorOptions,
) -> String {
    let mut output = String::new();
    let mut prev_was_comment = false;

    for (idx, item) in ast.items.iter().enumerate() {
        let item_output = generate_item(item, options);

        // Handle blank line preservation
        let should_add_blank_line =
            should_preserve_blank_line(item, idx > 0, prev_was_comment, options);

        if should_add_blank_line && idx > 0 {
            output.push('\n'); // Add blank line before item
        }

        // Apply line length limits
        let formatted_output = if let Some(max_len) = options.max_line_length {
            apply_line_length_limit(&item_output, max_len)
        } else {
            item_output
        };

        output.push_str(&formatted_output);
        output.push('\n');

        prev_was_comment = matches!(item, MakeItem::Comment { .. });
    }

    output
}

/// Determine if a blank line should be preserved before this item
fn should_preserve_blank_line(
    item: &MakeItem,
    has_prev: bool,
    prev_was_comment: bool,
    options: &MakefileGeneratorOptions,
) -> bool {
    if !has_prev {
        return false;
    }

    // If preserve_formatting is on, preserve blank lines before major sections
    if options.preserve_formatting || options.skip_blank_line_removal {
        match item {
            MakeItem::Comment { .. } if !prev_was_comment => true,
            MakeItem::Target { .. } => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Apply line length limit to output
///
/// Breaks long lines at reasonable boundaries (spaces, semicolons).
fn apply_line_length_limit(text: &str, max_length: usize) -> String {
    let mut result = String::new();

    for line in text.lines() {
        if line.len() <= max_length {
            result.push_str(line);
            result.push('\n');
        } else {
            // Break long line at spaces or semicolons
            let mut current_line = String::new();

            // Preserve leading tabs for recipe lines
            let leading_tabs = line.chars().take_while(|c| *c == '\t').count();
            let indent = "\t".repeat(leading_tabs);
            current_line.push_str(&indent);
            let mut current_len = indent.len();

            let content = &line[leading_tabs..];

            for word in content.split_whitespace() {
                let word_len = word.len() + 1; // +1 for space

                if current_len + word_len > max_length && current_len > indent.len() {
                    // Line would be too long, break here
                    result.push_str(&current_line);
                    if !current_line.ends_with('\\') {
                        result.push_str(" \\");
                    }
                    result.push('\n');

                    // Start new line with indent
                    current_line.clear();
                    current_line.push_str(&indent);
                    current_line.push(' '); // Continuation indent
                    current_len = indent.len() + 1;
                }

                if !current_line.ends_with(&indent) && !current_line.ends_with(' ') {
                    current_line.push(' ');
                    current_len += 1;
                }

                current_line.push_str(word);
                current_len += word.len();
            }

            if !current_line.trim().is_empty() {
                result.push_str(&current_line);
                result.push('\n');
            }
        }
    }

    result.trim_end_matches('\n').to_string()
}

/// Generate text for a single MakeItem
fn generate_item(item: &MakeItem, options: &MakefileGeneratorOptions) -> String {
    match item {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => generate_variable(name, value, flavor),

        MakeItem::Target {
            name,
            prerequisites,
            recipe,
            phony,
            recipe_metadata,
            ..
        } => generate_target(
            name,
            prerequisites,
            recipe,
            *phony,
            recipe_metadata.as_ref(),
            options,
        ),

        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            recipe_metadata,
            ..
        } => generate_pattern_rule(
            target_pattern,
            prereq_patterns,
            recipe,
            recipe_metadata.as_ref(),
            options,
        ),

        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => generate_conditional(condition, then_items, else_items.as_deref(), options),

        MakeItem::Include { path, optional, .. } => generate_include(path, *optional),

        MakeItem::Comment { text, .. } => generate_comment(text),

        MakeItem::FunctionCall { name, args, .. } => {
            // Function calls are typically embedded in variables, not standalone
            // But if we encounter one, generate it
            format!("$({} {})", name, args.join(","))
        }
    }
}

/// Generate a variable assignment
///
/// # Examples
///
/// ```ignore
/// # use bashrs::make_parser::ast::VarFlavor;
/// # use bashrs::make_parser::generators::generate_variable;
/// assert_eq!(generate_variable("CC", "gcc", &VarFlavor::Simple), "CC := gcc");
/// assert_eq!(generate_variable("VAR", "val", &VarFlavor::Recursive), "VAR = val");
/// ```
fn generate_variable(name: &str, value: &str, flavor: &VarFlavor) -> String {
    format!("{} {} {}", name, flavor, value)
}

/// Generate a target with prerequisites and recipe
///
/// # Arguments
///
/// * `name` - Target name
/// * `prerequisites` - List of prerequisites
/// * `recipe` - List of recipe lines (will be tab-indented)
/// * `phony` - Whether this target should be marked as .PHONY
/// * `recipe_metadata` - Optional metadata about line continuations
/// * `options` - Generator options (for preserve_formatting, skip_consolidation)
///
/// # Examples
///
/// ```ignore
/// # use bashrs::make_parser::generators::generate_target;
/// let options = MakefileGeneratorOptions::default();
/// let output = generate_target("build", &vec!["main.c".to_string()], &vec!["gcc -o build main.c".to_string()], false, None, &options);
/// assert!(output.contains("build: main.c"));
/// assert!(output.contains("\tgcc -o build main.c"));
/// ```
fn generate_target(
    name: &str,
    prerequisites: &[String],
    recipe: &[String],
    phony: bool,
    recipe_metadata: Option<&RecipeMetadata>,
    options: &MakefileGeneratorOptions,
) -> String {
    let mut output = String::new();

    // Add .PHONY declaration if needed
    if phony {
        output.push_str(&format!(".PHONY: {}\n", name));
    }

    // Generate target line
    output.push_str(name);
    output.push(':');

    if !prerequisites.is_empty() {
        output.push(' ');
        output.push_str(&prerequisites.join(" "));
    }

    output.push('\n');

    // Generate recipe lines
    // If preserve_formatting or skip_consolidation is set AND we have metadata,
    // reconstruct the original line breaks with backslash continuations
    if let Some(metadata) = recipe_metadata {
        if (options.preserve_formatting || options.skip_consolidation) && !recipe.is_empty() {
            // Reconstruct with line breaks
            for line in recipe {
                output.push_str(&reconstruct_recipe_line_with_breaks(line, metadata));
            }
        } else {
            // Default: Generate recipe lines as single lines (MUST use tabs)
            for line in recipe {
                output.push('\t');
                output.push_str(line);
                output.push('\n');
            }
        }
    } else {
        // No metadata: Generate recipe lines as single lines (MUST use tabs)
        for line in recipe {
            output.push('\t');
            output.push_str(line);
            output.push('\n');
        }
    }

    // Remove trailing newline (will be added by generate_item)
    output.pop();

    output
}

/// Reconstruct a recipe line with backslash continuations based on metadata
///
/// Takes a consolidated single-line recipe and metadata about where line breaks
/// occurred, and reconstructs the original multi-line format with backslashes.
fn reconstruct_recipe_line_with_breaks(line: &str, metadata: &RecipeMetadata) -> String {
    if metadata.line_breaks.is_empty() {
        // No breaks, return as single line with tab
        return format!("\t{}\n", line);
    }

    let mut output = String::new();
    output.push('\t'); // Start with tab

    let line_bytes = line.as_bytes();
    let mut last_pos = 0;

    for (break_pos, original_indent) in &metadata.line_breaks {
        // Add text up to break position
        if *break_pos <= line.len() {
            // Add text up to break, trimming any trailing space
            let text_segment = &line[last_pos..*break_pos];
            let trimmed_segment = text_segment.trim_end();
            output.push_str(trimmed_segment);
            output.push_str(" \\");
            output.push('\n');
            output.push_str(original_indent);

            // Move past the break position and skip the consolidation space
            last_pos = *break_pos;
            if last_pos < line_bytes.len() && line_bytes[last_pos] == b' ' {
                last_pos += 1; // Skip the space we added during consolidation
            }
        }
    }

    // Add remaining text
    if last_pos < line.len() {
        output.push_str(&line[last_pos..]);
    }
    output.push('\n');

    output
}

/// Generate a pattern rule
fn generate_pattern_rule(
    target_pattern: &str,
    prereq_patterns: &[String],
    recipe: &[String],
    recipe_metadata: Option<&RecipeMetadata>,
    options: &MakefileGeneratorOptions,
) -> String {
    generate_target(
        target_pattern,
        prereq_patterns,
        recipe,
        false,
        recipe_metadata,
        options,
    )
}

/// Generate a conditional block
fn generate_conditional(
    condition: &MakeCondition,
    then_items: &[MakeItem],
    else_items: Option<&[MakeItem]>,
    options: &MakefileGeneratorOptions,
) -> String {
    let mut output = String::new();

    // Generate condition
    match condition {
        MakeCondition::IfEq(left, right) => {
            output.push_str(&format!("ifeq ({},{})\n", left, right));
        }
        MakeCondition::IfNeq(left, right) => {
            output.push_str(&format!("ifneq ({},{})\n", left, right));
        }
        MakeCondition::IfDef(var) => {
            output.push_str(&format!("ifdef {}\n", var));
        }
        MakeCondition::IfNdef(var) => {
            output.push_str(&format!("ifndef {}\n", var));
        }
    }

    // Generate then branch
    for item in then_items {
        output.push_str(&generate_item(item, options));
        output.push('\n');
    }

    // Generate else branch if present
    if let Some(else_items) = else_items {
        output.push_str("else\n");
        for item in else_items {
            output.push_str(&generate_item(item, options));
            output.push('\n');
        }
    }

    // Close conditional
    output.push_str("endif");

    output
}

/// Generate an include directive
fn generate_include(path: &str, optional: bool) -> String {
    if optional {
        format!("-include {}", path)
    } else {
        format!("include {}", path)
    }
}

/// Generate a comment
fn generate_comment(text: &str) -> String {
    format!("# {}", text)
}

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
