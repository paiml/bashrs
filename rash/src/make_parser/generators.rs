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
        let item_output = generate_item(item);

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
fn generate_item(item: &MakeItem) -> String {
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
            ..
        } => generate_target(name, prerequisites, recipe, *phony),

        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            recipe,
            ..
        } => generate_pattern_rule(target_pattern, prereq_patterns, recipe),

        MakeItem::Conditional {
            condition,
            then_items,
            else_items,
            ..
        } => generate_conditional(condition, then_items, else_items.as_deref()),

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
///
/// # Examples
///
/// ```ignore
/// # use bashrs::make_parser::generators::generate_target;
/// let output = generate_target("build", &vec!["main.c".to_string()], &vec!["gcc -o build main.c".to_string()], false);
/// assert!(output.contains("build: main.c"));
/// assert!(output.contains("\tgcc -o build main.c"));
/// ```
fn generate_target(name: &str, prerequisites: &[String], recipe: &[String], phony: bool) -> String {
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

    // Generate recipe lines (MUST use tabs)
    for line in recipe {
        output.push('\t');
        output.push_str(line);
        output.push('\n');
    }

    // Remove trailing newline (will be added by generate_item)
    output.pop();

    output
}

/// Generate a pattern rule
fn generate_pattern_rule(
    target_pattern: &str,
    prereq_patterns: &[String],
    recipe: &[String],
) -> String {
    generate_target(target_pattern, prereq_patterns, recipe, false)
}

/// Generate a conditional block
fn generate_conditional(
    condition: &MakeCondition,
    then_items: &[MakeItem],
    else_items: Option<&[MakeItem]>,
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
        output.push_str(&generate_item(item));
        output.push('\n');
    }

    // Generate else branch if present
    if let Some(else_items) = else_items {
        output.push_str("else\n");
        for item in else_items {
            output.push_str(&generate_item(item));
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
