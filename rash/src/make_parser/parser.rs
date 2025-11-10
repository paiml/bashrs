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

    while pos < text.len() {
        // Look for $( pattern
        if text[pos..].starts_with("$(") {
            // Find the matching closing parenthesis
            let start = pos + 2; // Skip "$("
            let mut depth = 1;
            let mut end = start;

            for (i, ch) in text[start..].chars().enumerate() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i;
                        break;
                    }
                }
            }

            if depth == 0 {
                // Extract function content
                let content = &text[start..end];

                // Split by first space or comma to get function name
                let (func_name, args) = if let Some(space_pos) = content.find([' ', ',']) {
                    let name = &content[..space_pos];
                    let args = content[space_pos..].trim_start_matches([' ', ',']);
                    (name.to_string(), args.to_string())
                } else {
                    // No args (e.g., $(CURDIR))
                    (content.to_string(), String::new())
                };

                functions.push((func_name, args));
                pos = end + 1; // Skip past ')'
                continue;
            }
        }

        pos += 1;
    }

    functions
}

/// Split function arguments by commas, respecting nested parentheses
///
/// Example: "%.c,%.o,$(SOURCES)" -> ["%.c", "%.o", "$(SOURCES)"]
fn split_function_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in args.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                // Top-level comma - split here
                if !current.trim().is_empty() {
                    result.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    // Add last argument
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    // If no commas found, return the whole string as single arg
    if result.is_empty() && !args.trim().is_empty() {
        result.push(args.trim().to_string());
    }

    result
}

/// Parse an include directive
///
/// Include directive syntax:
/// ```makefile
/// include common.mk
/// -include optional.mk
/// sinclude optional.mk
/// ```
///
/// The `-include` and `sinclude` variants silently ignore missing files.
fn parse_include(line: &str, line_num: usize) -> Result<MakeItem, MakeParseError> {
    let trimmed = line.trim();

    // Check if this is optional include (-include or sinclude)
    let optional = trimmed.starts_with("-include ") || trimmed.starts_with("sinclude ");

    // Extract the keyword and path
    let path = if trimmed.starts_with("-include ") {
        trimmed
            .strip_prefix("-include ")
            .unwrap_or("")
            .trim()
            .to_string()
    } else if trimmed.starts_with("sinclude ") {
        trimmed
            .strip_prefix("sinclude ")
            .unwrap_or("")
            .trim()
            .to_string()
    } else if trimmed.starts_with("include ") {
        trimmed
            .strip_prefix("include ")
            .unwrap_or("")
            .trim()
            .to_string()
    } else {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::InvalidIncludeSyntax {
            location,
            found: trimmed.to_string(),
        });
    };

    // Path can be empty (edge case handled in tests)
    // Semantic validation should check for non-empty paths

    Ok(MakeItem::Include {
        path,
        optional,
        span: Span::new(0, line.len(), line_num),
    })
}

/// Parse a conditional block starting at the given line index
///
/// Updates the index to point past the parsed conditional (past 'endif').
///
/// Conditional block syntax:
/// ```makefile
/// ifeq ($(VAR),value)
///     CFLAGS = -g
/// else
///     CFLAGS = -O2
/// endif
/// ```
///
/// Supported directives: ifeq, ifneq, ifdef, ifndef
/// Parse two-argument condition (ifeq/ifneq)
fn parse_two_arg_condition(
    rest: &str,
    directive: &str,
    line_num: usize,
    line: &str,
    is_eq: bool,
) -> Result<MakeCondition, MakeParseError> {
    if !rest.starts_with('(') || !rest.ends_with(')') {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::InvalidConditionalSyntax {
            location,
            directive: directive.to_string(),
            found: rest.to_string(),
        });
    }
    let inner = &rest[1..rest.len() - 1];
    let parts: Vec<&str> = inner.splitn(2, ',').collect();
    if parts.len() != 2 {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::MissingConditionalArguments {
            location,
            directive: directive.to_string(),
            expected_args: 2,
            found_args: parts.len(),
        });
    }
    if is_eq {
        Ok(MakeCondition::IfEq(
            parts[0].to_string(),
            parts[1].to_string(),
        ))
    } else {
        Ok(MakeCondition::IfNeq(
            parts[0].to_string(),
            parts[1].to_string(),
        ))
    }
}

/// Parse single-variable condition (ifdef/ifndef)
fn parse_single_var_condition(
    var_name: String,
    directive: &str,
    line_num: usize,
    line: &str,
    is_def: bool,
) -> Result<MakeCondition, MakeParseError> {
    if var_name.is_empty() {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::MissingVariableName {
            location,
            directive: directive.to_string(),
        });
    }
    if is_def {
        Ok(MakeCondition::IfDef(var_name))
    } else {
        Ok(MakeCondition::IfNdef(var_name))
    }
}

/// Check if line starts a conditional directive
fn is_conditional_start(trimmed: &str) -> bool {
    trimmed.starts_with("ifeq ")
        || trimmed.starts_with("ifneq ")
        || trimmed.starts_with("ifdef ")
        || trimmed.starts_with("ifndef ")
}

/// Parse conditional branches (then and optional else)
fn parse_conditional_branches(
    lines: &[&str],
    index: &mut usize,
    metadata_map: &HashMap<usize, Vec<(usize, String)>>,
) -> Result<(Vec<MakeItem>, Option<Vec<MakeItem>>), MakeParseError> {
    let mut then_items = Vec::new();
    let mut else_items = None;
    let mut depth = 1;

    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim();

        if is_conditional_start(trimmed) {
            depth += 1;
        }

        if trimmed == "endif" {
            depth -= 1;
            if depth == 0 {
                *index += 1;
                break;
            }
        }

        if trimmed == "else" && depth == 1 {
            *index += 1;
            else_items = Some(parse_else_branch(lines, index, &mut depth, metadata_map)?);
            break;
        }

        match parse_conditional_item(lines, index, metadata_map) {
            Ok(Some(item)) => then_items.push(item),
            Ok(None) => *index += 1,
            Err(e) => {
                let location = SourceLocation::new(*index + 1);
                return Err(MakeParseError::InvalidTargetRule { location, found: e });
            }
        }
    }

    Ok((then_items, else_items))
}

/// Parse else branch of conditional
fn parse_else_branch(
    lines: &[&str],
    index: &mut usize,
    depth: &mut usize,
    metadata_map: &HashMap<usize, Vec<(usize, String)>>,
) -> Result<Vec<MakeItem>, MakeParseError> {
    let mut else_vec = Vec::new();

    while *index < lines.len() {
        let else_line = lines[*index];
        let else_trimmed = else_line.trim();

        if is_conditional_start(else_trimmed) {
            *depth += 1;
        }

        if else_trimmed == "endif" {
            *depth -= 1;
            if *depth == 0 {
                *index += 1;
                break;
            }
        }

        match parse_conditional_item(lines, index, metadata_map) {
            Ok(Some(item)) => else_vec.push(item),
            Ok(None) => *index += 1,
            Err(e) => {
                let location = SourceLocation::new(*index + 1);
                return Err(MakeParseError::InvalidTargetRule { location, found: e });
            }
        }
    }

    Ok(else_vec)
}

fn parse_conditional(
    lines: &[&str],
    index: &mut usize,
    metadata_map: &HashMap<usize, Vec<(usize, String)>>,
) -> Result<MakeItem, MakeParseError> {
    let start_line = lines[*index];
    let start_line_num = *index + 1;
    let trimmed = start_line.trim();

    // Parse the condition type and expression
    let condition = if trimmed.starts_with("ifeq ") {
        let rest = trimmed.strip_prefix("ifeq ").unwrap().trim();
        parse_two_arg_condition(rest, "ifeq", start_line_num, start_line, true)?
    } else if trimmed.starts_with("ifneq ") {
        let rest = trimmed.strip_prefix("ifneq ").unwrap().trim();
        parse_two_arg_condition(rest, "ifneq", start_line_num, start_line, false)?
    } else if trimmed.starts_with("ifdef ") {
        let var_name = trimmed.strip_prefix("ifdef ").unwrap().trim().to_string();
        parse_single_var_condition(var_name, "ifdef", start_line_num, start_line, true)?
    } else if trimmed.starts_with("ifndef ") {
        let var_name = trimmed.strip_prefix("ifndef ").unwrap().trim().to_string();
        parse_single_var_condition(var_name, "ifndef", start_line_num, start_line, false)?
    } else {
        let location = SourceLocation::new(start_line_num).with_source_line(start_line.to_string());
        return Err(MakeParseError::UnknownConditional {
            location,
            found: trimmed.to_string(),
        });
    };

    *index += 1;

    let (then_items, else_items) = parse_conditional_branches(lines, index, metadata_map)?;

    Ok(MakeItem::Conditional {
        condition,
        then_items,
        else_items,
        span: Span::new(0, start_line.len(), start_line_num),
    })
}

/// Parse a define...endef block for multi-line variable definitions
///
/// Syntax:
/// ```makefile
/// define VAR_NAME [=|:=]
/// multi-line
/// content
/// endef
/// ```
///
/// The index is moved past the endef line.
fn parse_define_block(lines: &[&str], index: &mut usize) -> Result<MakeItem, MakeParseError> {
    let start_line = lines[*index];
    let start_line_num = *index + 1;
    let trimmed = start_line.trim();

    // Parse: define VAR_NAME [=|:=]
    let after_define = trimmed.strip_prefix("define ").unwrap().trim();

    // Check for assignment flavor (=, :=, ?=, +=, !=)
    let (var_name, flavor) = if let Some(name) = after_define.strip_suffix(" =") {
        (name.trim().to_string(), VarFlavor::Recursive)
    } else if let Some(name) = after_define.strip_suffix(" :=") {
        (name.trim().to_string(), VarFlavor::Simple)
    } else if let Some(name) = after_define.strip_suffix(" ?=") {
        (name.trim().to_string(), VarFlavor::Conditional)
    } else if let Some(name) = after_define.strip_suffix(" +=") {
        (name.trim().to_string(), VarFlavor::Append)
    } else if let Some(name) = after_define.strip_suffix(" !=") {
        (name.trim().to_string(), VarFlavor::Shell)
    } else {
        // No explicit flavor - defaults to recursive
        (after_define.to_string(), VarFlavor::Recursive)
    };

    if var_name.is_empty() {
        let location = SourceLocation::new(start_line_num).with_source_line(start_line.to_string());
        return Err(MakeParseError::MissingVariableName {
            location,
            directive: "define".to_string(),
        });
    }

    // Move past the define line
    *index += 1;

    // Collect lines until we find endef
    let mut value_lines = Vec::new();
    while *index < lines.len() {
        let line = lines[*index];

        // Check for endef
        if line.trim() == "endef" {
            // Move past the endef line
            *index += 1;

            // Join the collected lines (preserve newlines and indentation)
            let value = value_lines.join("\n");

            return Ok(MakeItem::Variable {
                name: var_name,
                value,
                flavor,
                span: Span::new(0, start_line.len(), start_line_num),
            });
        }

        // Add this line to the value
        value_lines.push(line.to_string());
        *index += 1;
    }

    // If we got here, we never found endef
    let location = SourceLocation::new(start_line_num).with_source_line(start_line.to_string());
    Err(MakeParseError::UnterminatedDefine { location, var_name })
}

/// Parse a single item within a conditional block
///
/// Returns None for empty lines or lines that should be skipped
///
/// IMPORTANT: This function does NOT increment index for simple items like variables or comments,
/// but parse_target_rule DOES increment index (it advances past recipes).
/// The caller must increment index when Ok(None) is returned.
fn parse_conditional_item(
    lines: &[&str],
    index: &mut usize,
    metadata_map: &HashMap<usize, Vec<(usize, String)>>,
) -> Result<Option<MakeItem>, String> {
    let line = lines[*index];
    let line_num = *index + 1;

    // Skip empty lines
    if line.trim().is_empty() {
        return Ok(None);
    }

    // Don't parse conditional keywords here (handled by parent)
    let trimmed = line.trim();
    if trimmed == "else"
        || trimmed == "endif"
        || trimmed.starts_with("ifeq ")
        || trimmed.starts_with("ifneq ")
        || trimmed.starts_with("ifdef ")
        || trimmed.starts_with("ifndef ")
    {
        return Ok(None);
    }

    // Parse variable assignment
    if is_variable_assignment(line) {
        let var = parse_variable(line, line_num).map_err(|e| e.to_string())?;
        *index += 1; // Move past this variable line
        return Ok(Some(var));
    }

    // Parse target rule (THIS INCREMENTS INDEX - it advances past recipes)
    if line.contains(':') && !line.trim_start().starts_with('\t') {
        let target = parse_target_rule(lines, index, metadata_map).map_err(|e| e.to_string())?;
        // parse_target_rule already incremented index past the target and its recipe
        // So we DON'T increment it again
        return Ok(Some(target));
    }

    // Parse comment
    if line.trim_start().starts_with('#') {
        let text = line
            .trim_start()
            .strip_prefix('#')
            .unwrap_or("")
            .trim()
            .to_string();
        *index += 1; // Move past this comment line
        return Ok(Some(MakeItem::Comment {
            text,
            span: Span::new(0, line.len(), line_num),
        }));
    }

    // Unknown item - skip
    Ok(None)
}

/// Parse a target rule starting at the given line index
///
/// Updates the index to point past the parsed rule.
///
/// Target rule syntax:
/// ```makefile
/// target: prerequisites
///     recipe line 1
///     recipe line 2
/// ```
///
/// Pattern rule syntax:
/// ```makefile
/// %.o: %.c
///     $(CC) -c $< -o $@
/// ```
fn parse_target_rule(
    lines: &[&str],
    index: &mut usize,
    metadata_map: &HashMap<usize, Vec<(usize, String)>>,
) -> Result<MakeItem, MakeParseError> {
    let line = lines[*index];
    let line_num = *index + 1;

    // Split on ':' to get target and prerequisites
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::InvalidTargetRule {
            location,
            found: line.trim().to_string(),
        });
    }

    let name = parts[0].trim().to_string();
    if name.is_empty() {
        let location = SourceLocation::new(line_num).with_source_line(line.to_string());
        return Err(MakeParseError::EmptyTargetName { location });
    }

    // Parse prerequisites (space-separated)
    let prerequisites: Vec<String> = parts[1].split_whitespace().map(|s| s.to_string()).collect();

    // Parse recipe lines (tab-indented lines following the target)
    *index += 1;
    let mut recipe = Vec::new();
    let recipe_start_line = *index; // Track where recipes start for metadata lookup

    while *index < lines.len() {
        let recipe_line = lines[*index];

        // Recipe lines must start with a tab
        if recipe_line.starts_with('\t') {
            recipe.push(recipe_line.trim().to_string());
            *index += 1;
        } else if recipe_line.trim().is_empty() {
            // Empty line - could be end of recipe or just whitespace
            *index += 1;
            // Check if next line is also recipe
            if *index < lines.len() && lines[*index].starts_with('\t') {
                continue;
            } else {
                break;
            }
        } else {
            // Non-tab-indented, non-empty line - end of recipe
            break;
        }
    }

    // Check if any recipe lines have line continuation metadata
    // For simplicity, if the first recipe line has metadata, use it
    let recipe_metadata = if !recipe.is_empty() {
        metadata_map
            .get(&recipe_start_line)
            .map(|breaks| RecipeMetadata {
                line_breaks: breaks.clone(),
            })
    } else {
        None
    };

    // Check if this is a pattern rule (target contains %)
    if name.contains('%') {
        Ok(MakeItem::PatternRule {
            target_pattern: name,
            prereq_patterns: prerequisites,
            recipe,
            recipe_metadata,
            span: Span::new(0, line.len(), line_num),
        })
    } else {
        Ok(MakeItem::Target {
            name,
            prerequisites,
            recipe,
            phony: false, // Will be detected in semantic analysis
            recipe_metadata,
            span: Span::new(0, line.len(), line_num),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_parser::error::SourceLocation;

    #[test]
    fn test_parse_empty_makefile() {
        let result = parse_makefile("");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 0);
    }

    #[test]
    fn test_parse_target_with_recipe() {
        let makefile = "build:\n\tcargo build";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Target { name, recipe, .. } => {
                assert_eq!(name, "build");
                assert_eq!(recipe.len(), 1);
                assert_eq!(recipe[0], "cargo build");
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_parse_target_no_prerequisites() {
        let makefile = "test:\n\tcargo test";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 0);
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_parse_multiple_targets() {
        let makefile = "build:\n\tcargo build\n\ntest:\n\tcargo test";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 2);
    }

    // ===== NASA-QUALITY UNIT TESTS for parse_makefile_items helpers =====

    #[test]
    fn test_should_skip_line_empty() {
        assert!(should_skip_line(""), "Empty line should be skipped");
        assert!(
            should_skip_line("   "),
            "Whitespace-only line should be skipped"
        );
        assert!(should_skip_line("\t"), "Tab-only line should be skipped");
    }

    #[test]
    fn test_should_skip_line_non_empty() {
        assert!(
            !should_skip_line("build:"),
            "Target line should NOT be skipped"
        );
        assert!(
            !should_skip_line("# comment"),
            "Comment should NOT be skipped"
        );
        assert!(
            !should_skip_line("VAR = value"),
            "Variable should NOT be skipped"
        );
    }

    #[test]
    fn test_try_parse_comment_valid() {
        let comment = try_parse_comment("# This is a comment", 1);
        assert!(comment.is_some(), "Should recognize comment line");

        if let Some(MakeItem::Comment { text, .. }) = comment {
            assert_eq!(
                text, "This is a comment",
                "Comment text should have # stripped and be trimmed"
            );
        } else {
            panic!("Expected Comment item");
        }
    }

    #[test]
    fn test_try_parse_comment_not_comment() {
        let comment = try_parse_comment("build:", 1);
        assert!(comment.is_none(), "Non-comment should return None");
    }

    #[test]
    fn test_try_parse_include_valid() {
        let result = try_parse_include("include foo.mk", 1);
        assert!(result.is_some(), "Should recognize include directive");

        if let Some(Ok(MakeItem::Include { path, .. })) = result {
            assert_eq!(path, "foo.mk");
        } else {
            panic!("Expected Include item");
        }
    }

    #[test]
    fn test_try_parse_include_not_include() {
        let result = try_parse_include("build:", 1);
        assert!(result.is_none(), "Non-include should return None");
    }

    #[test]
    fn test_try_parse_variable_valid() {
        let result = try_parse_variable("CC = gcc", 1);
        assert!(result.is_some(), "Should recognize variable assignment");

        if let Some(Ok(MakeItem::Variable { name, value, .. })) = result {
            assert_eq!(name, "CC");
            assert_eq!(value, "gcc");
        } else {
            panic!("Expected Variable item");
        }
    }

    #[test]
    fn test_try_parse_variable_not_variable() {
        let result = try_parse_variable("build:", 1);
        assert!(result.is_none(), "Non-variable should return None");
    }

    #[test]
    fn test_try_add_item_success() {
        let mut items = Vec::new();
        let item = MakeItem::Comment {
            text: "# test".to_string(),
            span: Span::new(0, 6, 1),
        };

        let result = try_add_item(&mut items, Ok(item));

        assert!(result.is_ok(), "Should successfully add item");
        assert_eq!(items.len(), 1, "Should have 1 item");
    }

    #[test]
    fn test_try_add_item_error() {
        let mut items = Vec::new();
        let error = MakeParseError::EmptyVariableName {
            location: SourceLocation::new(1),
        };

        let result = try_add_item(&mut items, Err(error));

        assert!(result.is_err(), "Should propagate error");
        assert_eq!(items.len(), 0, "Should not add any items on error");
        assert!(
            result.unwrap_err().contains("Empty variable name"),
            "Error message should be preserved"
        );
    }

    #[test]
    fn test_try_add_item_multiple_success() {
        let mut items = Vec::new();

        let item1 = MakeItem::Comment {
            text: "# comment 1".to_string(),
            span: Span::new(0, 11, 1),
        };
        let item2 = MakeItem::Comment {
            text: "# comment 2".to_string(),
            span: Span::new(0, 11, 2),
        };

        assert!(try_add_item(&mut items, Ok(item1)).is_ok());
        assert!(try_add_item(&mut items, Ok(item2)).is_ok());

        assert_eq!(items.len(), 2, "Should have 2 items");
    }

    #[test]
    fn test_try_parse_comment_trims_whitespace() {
        let comment = try_parse_comment("#   indented comment  ", 1);
        assert!(comment.is_some());

        if let Some(MakeItem::Comment { text, .. }) = comment {
            assert_eq!(
                text, "indented comment",
                "Whitespace should be trimmed (both leading and trailing)"
            );
        } else {
            panic!("Expected Comment item");
        }
    }
}
