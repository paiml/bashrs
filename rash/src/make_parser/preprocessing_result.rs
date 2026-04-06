//! Makefile parser
//!
//! Parses GNU Makefiles into AST representation.
//!
//! ## Design Principles
//!
//! - Keep complexity <10 per function
//! - Clear error messages
//! - Preserve source location information
//! - Support incremental parsing

use super::ast::*;
use super::error::{MakeParseError, SourceLocation};
use std::collections::HashMap;

/// Result of preprocessing with metadata about line continuations
struct PreprocessingResult {
    /// The preprocessed text with continuations resolved
    text: String,
    /// Metadata for recipe lines with continuations
    /// Maps from preprocessed line number to continuation metadata
    /// breaks: Vec<(position_in_line, original_indentation)>
    recipe_metadata_map: HashMap<usize, Vec<(usize, String)>>,
}

/// Preprocess Makefile input to handle line continuations
///
/// Line continuations in Makefiles use backslash (\) at the end of a line
/// to concatenate with the next line. This function processes the input
/// and joins continued lines with a single space.
///
/// # Arguments
///
/// * `input` - Raw Makefile source code
///
/// # Returns
///
/// * `String` - Preprocessed Makefile with continuations resolved
///
/// # Examples
///
/// ```ignore
/// let input = "VAR = a \\\n    b";
/// let output = preprocess_line_continuations(input);
/// assert_eq!(output, "VAR = a b");
/// ```
/// Preprocess with metadata tracking for line continuations
fn preprocess_line_continuations_with_metadata(input: &str) -> PreprocessingResult {
    let mut result = String::new();
    let mut recipe_metadata_map = HashMap::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut preprocessed_line_num = 0;

    while i < lines.len() {
        let mut line = lines[i].to_string();
        let mut breaks: Vec<(usize, String)> = Vec::new();

        // Check if this line ends with backslash (continuation)
        while line.trim_end().ends_with('\\') && i + 1 < lines.len() {
            // Record the position where we're about to insert the continuation
            let break_position = line.trim_end().strip_suffix('\\').unwrap().trim_end().len();

            // Remove the trailing backslash and any trailing whitespace
            line = line
                .trim_end()
                .strip_suffix('\\')
                .unwrap()
                .trim_end()
                .to_string();

            // Get the next line and capture its original indentation
            i += 1;
            let next_line_full = lines[i];
            let original_indent = next_line_full
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            let next_line = next_line_full.trim_start();

            // Record the break position and original indentation
            breaks.push((break_position, original_indent));

            // Concatenate with a single space
            line.push(' ');
            line.push_str(next_line);
        }

        // If this line had continuations, store the metadata mapped to preprocessed line number
        if !breaks.is_empty() {
            recipe_metadata_map.insert(preprocessed_line_num, breaks);
        }

        result.push_str(&line);
        result.push('\n');
        i += 1;
        preprocessed_line_num += 1;
    }

    PreprocessingResult {
        text: result,
        recipe_metadata_map,
    }
}

fn preprocess_line_continuations(input: &str) -> String {
    // Simple version for backward compatibility
    preprocess_line_continuations_with_metadata(input).text
}

/// Parse a Makefile string into an AST
///
/// # Arguments
///
/// * `input` - Makefile source code as a string
///
/// # Returns
///
/// * `Ok(MakeAst)` - Successfully parsed AST
/// * `Err(String)` - Parse error with description
///
/// # Examples
///
/// ```rust
/// use bashrs::make_parser::parse_makefile;
///
/// let makefile = "test:\n\tcargo test";
/// let ast = parse_makefile(makefile).unwrap();
/// assert_eq!(ast.items.len(), 1);
/// ```
/// Check if line is empty or whitespace-only
fn is_empty_line(line: &str) -> bool {
    line.trim().is_empty()
}

/// Check if line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line starts an include directive
fn is_include_directive(line: &str) -> bool {
    line.trim_start().starts_with("include ")
        || line.trim_start().starts_with("-include ")
        || line.trim_start().starts_with("sinclude ")
}

/// Check if line starts a conditional block
fn is_conditional_directive(line: &str) -> bool {
    line.trim_start().starts_with("ifeq ")
        || line.trim_start().starts_with("ifdef ")
        || line.trim_start().starts_with("ifndef ")
        || line.trim_start().starts_with("ifneq ")
}

/// Check if line starts a define block
fn is_define_directive(line: &str) -> bool {
    line.trim_start().starts_with("define ")
}

/// Check if line is a target rule
fn is_target_rule(line: &str) -> bool {
    line.contains(':') && !line.trim_start().starts_with('\t')
}

/// Parse a comment line and create MakeItem::Comment
fn parse_comment_line(line: &str, line_num: usize) -> MakeItem {
    let text = line
        .trim_start()
        .strip_prefix('#')
        .unwrap_or("")
        .trim()
        .to_string();

    MakeItem::Comment {
        text,
        span: Span::new(0, line.len(), line_num),
    }
}

/// Parse all Makefile items (first pass)
/// Try to parse and add item to list, handling errors
fn try_add_item(
    items: &mut Vec<MakeItem>,
    result: Result<MakeItem, MakeParseError>,
) -> Result<(), String> {
    match result {
        Ok(item) => {
            items.push(item);
            Ok(())
        }
        Err(e) => Err(e.to_detailed_string()),
    }
}

/// Try to parse include directive
fn try_parse_include(line: &str, line_num: usize) -> Option<Result<MakeItem, MakeParseError>> {
    if is_include_directive(line) {
        Some(parse_include(line, line_num))
    } else {
        None
    }
}

/// Try to parse variable assignment
fn try_parse_variable(line: &str, line_num: usize) -> Option<Result<MakeItem, MakeParseError>> {
    if is_variable_assignment(line) {
        Some(parse_variable(line, line_num))
    } else {
        None
    }
}

/// Try to parse comment line
fn try_parse_comment(line: &str, line_num: usize) -> Option<MakeItem> {
    if is_comment_line(line) {
        Some(parse_comment_line(line, line_num))
    } else {
        None
    }
}

/// Should skip this line (empty)
fn should_skip_line(line: &str) -> bool {
    is_empty_line(line)
}

fn parse_makefile_items(
    lines: &[&str],
    metadata_map: &HashMap<usize, Vec<(usize, String)>>,
) -> Result<Vec<MakeItem>, String> {
    let mut items = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let line_num = i + 1;

        // Skip empty lines
        if should_skip_line(line) {
            i += 1;
            continue;
        }

        // Try comment
        if let Some(comment) = try_parse_comment(line, line_num) {
            items.push(comment);
            i += 1;
            continue;
        }

        // Try include directive
        if let Some(result) = try_parse_include(line, line_num) {
            try_add_item(&mut items, result)?;
            i += 1;
            continue;
        }

        // Try conditional directive
        if is_conditional_directive(line) {
            try_add_item(&mut items, parse_conditional(lines, &mut i, metadata_map))?;
            continue;
        }

        // Try define directive
        if is_define_directive(line) {
            try_add_item(&mut items, parse_define_block(lines, &mut i))?;
            continue;
        }

        // Try variable assignment
        if let Some(result) = try_parse_variable(line, line_num) {
            try_add_item(&mut items, result)?;
            i += 1;
            continue;
        }

        // Try target rule
        if is_target_rule(line) {
            try_add_item(&mut items, parse_target_rule(lines, &mut i, metadata_map))?;
            continue;
        }

        i += 1;
    }

    Ok(items)
}

/// Collect all .PHONY target declarations
fn collect_phony_targets(items: &[MakeItem]) -> std::collections::HashSet<String> {
    let mut phony_targets = std::collections::HashSet::new();

    for item in items {
        if let MakeItem::Target {
            name,
            prerequisites,
            ..
        } = item
        {
            if name == ".PHONY" {
                for prereq in prerequisites {
                    phony_targets.insert(prereq.clone());
                }
            }
        }
    }

    phony_targets
}

/// Mark targets as .PHONY if declared (second pass)
fn mark_phony_targets(
    items: Vec<MakeItem>,
    phony_targets: &std::collections::HashSet<String>,
) -> Vec<MakeItem> {
    items
        .into_iter()
        .map(|item| {
            if let MakeItem::Target {
                name,
                prerequisites,
                recipe,
                phony: _,
                recipe_metadata,
                span,
            } = item
            {
                MakeItem::Target {
                    phony: phony_targets.contains(&name),
                    name,
                    prerequisites,
                    recipe,
                    recipe_metadata,
                    span,
                }
            } else {
                item
            }
        })
        .collect()
}

pub fn parse_makefile(input: &str) -> Result<MakeAst, String> {
    let preprocessing = preprocess_line_continuations_with_metadata(input);
    let lines: Vec<&str> = preprocessing.text.lines().collect();
    let line_count = lines.len();

    // First pass: Parse all items
    let mut items = parse_makefile_items(&lines, &preprocessing.recipe_metadata_map)?;

    // Second pass: Mark .PHONY targets
    let phony_targets = collect_phony_targets(&items);
    items = mark_phony_targets(items, &phony_targets);

    Ok(MakeAst {
        items,
        metadata: MakeMetadata::with_line_count(line_count),
    })
}

/// Check if a line is a variable assignment
///
/// Variable assignments contain '=' but are not target rules (which contain ':').
/// We need to check for assignment operators (:=, ?=, +=, !=, =) to distinguish.
///
/// Examples:
/// - "CC = gcc" -> true (variable)
/// - "CC := gcc" -> true (variable, := is assignment operator)
/// - "target: dep" -> false (target rule)
/// - "target: VAR=value" -> false (target rule with variable in prerequisites)
fn is_variable_assignment(line: &str) -> bool {
    let trimmed = line.trim();

    // Check for assignment operators (in order of specificity)
    if trimmed.contains(":=")
        || trimmed.contains("?=")
        || trimmed.contains("+=")
        || trimmed.contains("!=")
    {
        return true;
    }

    // Must contain '='
    if !trimmed.contains('=') {
        return false;
    }

    // If it contains ':', check if ':' comes before '='
    // This distinguishes "CC = gcc" from "target: VAR=value"
    if let Some(colon_pos) = trimmed.find(':') {
        if let Some(equals_pos) = trimmed.find('=') {
            // If ':' comes before '=', it's a target rule
            if colon_pos < equals_pos {
                return false;
            }
        }
    }

    true
}

/// Parse a variable assignment
///
/// Variable assignment syntax:
/// ```makefile
/// VAR = value      # Recursive (expanded when used)
/// VAR := value     # Simple (expanded immediately)
/// VAR ?= value     # Conditional (only if not defined)
/// VAR += value     # Append
/// VAR != command   # Shell (execute command)
/// ```
fn parse_variable(line: &str, line_num: usize) -> Result<MakeItem, MakeParseError> {
    let trimmed = line.trim();

    // Determine the flavor by finding the assignment operator
    let (name_part, value_part, flavor) = if let Some(pos) = trimmed.find(":=") {
        (&trimmed[..pos], &trimmed[pos + 2..], VarFlavor::Simple)
    } else if let Some(pos) = trimmed.find("?=") {
        (&trimmed[..pos], &trimmed[pos + 2..], VarFlavor::Conditional)
    } else if let Some(pos) = trimmed.find("+=") {
        (&trimmed[..pos], &trimmed[pos + 2..], VarFlavor::Append)
    } else if let Some(pos) = trimmed.find("!=") {
        (&trimmed[..pos], &trimmed[pos + 2..], VarFlavor::Shell)
    } else if let Some(pos) = trimmed.find('=') {
        (&trimmed[..pos], &trimmed[pos + 1..], VarFlavor::Recursive)
    } else {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::NoAssignmentOperator {
            location,
            found: trimmed.to_string(),
        });
    };

    let name = name_part.trim();
    if name.is_empty() {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::EmptyVariableName { location });
    }

    let value = value_part.trim();

    Ok(MakeItem::Variable {
        name: name.to_string(),
        value: value.to_string(),
        flavor,
        span: Span::new(0, line.len(), line_num),
    })
}

/// Detect if a string contains a GNU Make function call
///
/// Function call syntax: $(function_name arg1,arg2,...)
/// Examples: $(wildcard *.c), $(patsubst %.c,%.o,$(SOURCES))
fn contains_function_call(text: &str) -> bool {
    // Check for $( pattern which indicates potential function call
    text.contains("$(") && !text.starts_with('$')
}

/// Extract function calls from a string
///
/// Returns a vector of (function_name, args_string) tuples
/// Handles nested function calls by extracting the outermost one first
///
/// # Examples
///
/// ```ignore
/// let calls = extract_function_calls("$(wildcard *.c)");
/// assert_eq!(calls[0].0, "wildcard");
/// ```
pub fn extract_function_calls(text: &str) -> Vec<(String, String)> {
    let mut functions = Vec::new();
    let chars = text.chars().peekable();
    let mut pos = 0;


}

    include!("parser_part2_incl2.rs");
