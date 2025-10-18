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
    let mut output = String::new();

    for item in &ast.items {
        let item_output = generate_item(item);
        output.push_str(&item_output);
        output.push('\n');
    }

    output
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
fn generate_target(
    name: &str,
    prerequisites: &[String],
    recipe: &[String],
    phony: bool,
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
