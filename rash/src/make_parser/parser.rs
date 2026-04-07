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
            let break_position = line
                .trim_end()
                .strip_suffix('\\')
                .expect("backslash suffix verified by while condition")
                .trim_end()
                .len();

            // Remove the trailing backslash and any trailing whitespace
            line = line
                .trim_end()
                .strip_suffix('\\')
                .expect("backslash suffix verified by while condition")
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

include!("parser_is_variable.rs");
