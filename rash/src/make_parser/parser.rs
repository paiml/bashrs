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
/// ```
/// let input = "VAR = a \\\n    b";
/// let output = preprocess_line_continuations(input);
/// assert_eq!(output, "VAR = a b");
/// ```
fn preprocess_line_continuations(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let mut line = lines[i].to_string();

        // Check if this line ends with backslash (continuation)
        while line.trim_end().ends_with('\\') && i + 1 < lines.len() {
            // Remove the trailing backslash and any trailing whitespace
            line = line.trim_end().strip_suffix('\\').unwrap().trim_end().to_string();

            // Get the next line and trim leading whitespace
            i += 1;
            let next_line = lines[i].trim_start();

            // Concatenate with a single space
            line.push(' ');
            line.push_str(next_line);
        }

        result.push_str(&line);
        result.push('\n');
        i += 1;
    }

    result
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
pub fn parse_makefile(input: &str) -> Result<MakeAst, String> {
    // Preprocess: handle line continuations (backslash at end of line)
    let preprocessed = preprocess_line_continuations(input);
    let lines: Vec<&str> = preprocessed.lines().collect();
    let line_count = lines.len();

    let mut items = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Skip empty lines
        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        // Parse comment lines
        if line.trim_start().starts_with('#') {
            let text = line.trim_start()
                .strip_prefix('#')
                .unwrap_or("")
                .trim()
                .to_string();

            items.push(MakeItem::Comment {
                text,
                span: Span::new(0, line.len(), i + 1),
            });

            i += 1;
            continue;
        }

        // Parse include directives
        if line.trim_start().starts_with("include ") || line.trim_start().starts_with("-include ") || line.trim_start().starts_with("sinclude ") {
            match parse_include(line, i + 1) {
                Ok(include) => items.push(include),
                Err(e) => return Err(format!("Line {}: {}", i + 1, e)),
            }
            i += 1;
            continue;
        }

        // Parse conditional blocks (ifeq, ifdef, ifndef, ifneq)
        if line.trim_start().starts_with("ifeq ") ||
           line.trim_start().starts_with("ifdef ") ||
           line.trim_start().starts_with("ifndef ") ||
           line.trim_start().starts_with("ifneq ") {
            match parse_conditional(&lines, &mut i) {
                Ok(conditional) => items.push(conditional),
                Err(e) => return Err(format!("Line {}: {}", i + 1, e)),
            }
            continue;
        }

        // Parse variable assignments (contains '=' but is not a target rule)
        if is_variable_assignment(line) {
            match parse_variable(line, i + 1) {
                Ok(var) => items.push(var),
                Err(e) => return Err(format!("Line {}: {}", i + 1, e)),
            }
            i += 1;
            continue;
        }

        // Parse target rules (contains ':')
        if line.contains(':') && !line.trim_start().starts_with('\t') {
            match parse_target_rule(&lines, &mut i) {
                Ok(target) => items.push(target),
                Err(e) => return Err(format!("Line {}: {}", i + 1, e)),
            }
            continue;
        }

        // Unknown line - skip for now
        i += 1;
    }

    // Second pass: Mark targets as .PHONY
    // Collect all .PHONY declarations
    let mut phony_targets: std::collections::HashSet<String> = std::collections::HashSet::new();
    for item in &items {
        if let MakeItem::Target { name, prerequisites, .. } = item {
            if name == ".PHONY" {
                for prereq in prerequisites {
                    phony_targets.insert(prereq.clone());
                }
            }
        }
    }

    // Update targets to mark them as phony if declared
    items = items.into_iter().map(|item| {
        if let MakeItem::Target { name, prerequisites, recipe, phony: _, span } = item {
            MakeItem::Target {
                phony: phony_targets.contains(&name),
                name,
                prerequisites,
                recipe,
                span,
            }
        } else {
            item
        }
    }).collect();

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
    if trimmed.contains(":=") || trimmed.contains("?=") ||
       trimmed.contains("+=") || trimmed.contains("!=") {
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
fn parse_variable(line: &str, line_num: usize) -> Result<MakeItem, String> {
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
        return Err("No assignment operator found".to_string());
    };

    let name = name_part.trim();
    if name.is_empty() {
        return Err("Empty variable name".to_string());
    }

    let value = value_part.trim();

    Ok(MakeItem::Variable {
        name: name.to_string(),
        value: value.to_string(),
        flavor,
        span: Span::new(0, line.len(), line_num),
    })
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
fn parse_include(line: &str, line_num: usize) -> Result<MakeItem, String> {
    let trimmed = line.trim();

    // Check if this is optional include (-include or sinclude)
    let optional = trimmed.starts_with("-include ") || trimmed.starts_with("sinclude ");

    // Extract the keyword and path
    let path = if trimmed.starts_with("-include ") {
        trimmed.strip_prefix("-include ")
            .unwrap_or("")
            .trim()
            .to_string()
    } else if trimmed.starts_with("sinclude ") {
        trimmed.strip_prefix("sinclude ")
            .unwrap_or("")
            .trim()
            .to_string()
    } else if trimmed.starts_with("include ") {
        trimmed.strip_prefix("include ")
            .unwrap_or("")
            .trim()
            .to_string()
    } else {
        return Err("Invalid include syntax".to_string());
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
fn parse_conditional(lines: &[&str], index: &mut usize) -> Result<MakeItem, String> {
    let start_line = lines[*index];
    let start_line_num = *index + 1;
    let trimmed = start_line.trim();

    // Parse the condition type and expression
    let condition = if trimmed.starts_with("ifeq ") {
        // ifeq (arg1,arg2)
        let rest = trimmed.strip_prefix("ifeq ").unwrap().trim();
        if !rest.starts_with('(') || !rest.ends_with(')') {
            return Err(format!("Invalid ifeq syntax at line {}", start_line_num));
        }
        let inner = &rest[1..rest.len()-1];
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(format!("ifeq requires two arguments at line {}", start_line_num));
        }
        MakeCondition::IfEq(parts[0].to_string(), parts[1].to_string())
    } else if trimmed.starts_with("ifneq ") {
        // ifneq (arg1,arg2)
        let rest = trimmed.strip_prefix("ifneq ").unwrap().trim();
        if !rest.starts_with('(') || !rest.ends_with(')') {
            return Err(format!("Invalid ifneq syntax at line {}", start_line_num));
        }
        let inner = &rest[1..rest.len()-1];
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(format!("ifneq requires two arguments at line {}", start_line_num));
        }
        MakeCondition::IfNeq(parts[0].to_string(), parts[1].to_string())
    } else if trimmed.starts_with("ifdef ") {
        // ifdef VAR
        let var_name = trimmed.strip_prefix("ifdef ").unwrap().trim().to_string();
        if var_name.is_empty() {
            return Err(format!("ifdef requires variable name at line {}", start_line_num));
        }
        MakeCondition::IfDef(var_name)
    } else if trimmed.starts_with("ifndef ") {
        // ifndef VAR
        let var_name = trimmed.strip_prefix("ifndef ").unwrap().trim().to_string();
        if var_name.is_empty() {
            return Err(format!("ifndef requires variable name at line {}", start_line_num));
        }
        MakeCondition::IfNdef(var_name)
    } else {
        return Err(format!("Unknown conditional directive at line {}", start_line_num));
    };

    // Move past the ifeq/ifdef/ifndef/ifneq line
    *index += 1;

    // Parse items in the 'then' branch until we hit 'else' or 'endif'
    let mut then_items = Vec::new();
    let mut else_items = None;
    let mut depth = 1; // Track nested conditionals

    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim();

        // Check for nested conditionals
        if trimmed.starts_with("ifeq ") || trimmed.starts_with("ifneq ") ||
           trimmed.starts_with("ifdef ") || trimmed.starts_with("ifndef ") {
            depth += 1;
        }

        // Check for endif
        if trimmed == "endif" {
            depth -= 1;
            if depth == 0 {
                // This endif closes our conditional
                *index += 1;
                break;
            }
        }

        // Check for else at our level
        if trimmed == "else" && depth == 1 {
            *index += 1;
            // Parse items in the 'else' branch
            let mut else_vec = Vec::new();
            while *index < lines.len() {
                let else_line = lines[*index];
                let else_trimmed = else_line.trim();

                // Check for nested conditionals in else branch
                if else_trimmed.starts_with("ifeq ") || else_trimmed.starts_with("ifneq ") ||
                   else_trimmed.starts_with("ifdef ") || else_trimmed.starts_with("ifndef ") {
                    depth += 1;
                }

                if else_trimmed == "endif" {
                    depth -= 1;
                    if depth == 0 {
                        *index += 1;
                        break;
                    }
                }

                // Parse item in else branch
                if let Some(item) = parse_conditional_item(lines, index)? {
                    else_vec.push(item);
                    // Note: index was already incremented by parse_conditional_item
                } else {
                    // Empty line or unrecognized - skip it
                    *index += 1;
                }
            }
            else_items = Some(else_vec);
            break;
        }

        // Parse item in then branch
        if let Some(item) = parse_conditional_item(lines, index)? {
            then_items.push(item);
            // Note: index was already incremented by parse_conditional_item
        } else {
            // Empty line or unrecognized - skip it
            *index += 1;
        }
    }

    Ok(MakeItem::Conditional {
        condition,
        then_items,
        else_items,
        span: Span::new(0, start_line.len(), start_line_num),
    })
}

/// Parse a single item within a conditional block
///
/// Returns None for empty lines or lines that should be skipped
///
/// IMPORTANT: This function does NOT increment index for simple items like variables or comments,
/// but parse_target_rule DOES increment index (it advances past recipes).
/// The caller must increment index when Ok(None) is returned.
fn parse_conditional_item(lines: &[&str], index: &mut usize) -> Result<Option<MakeItem>, String> {
    let line = lines[*index];
    let line_num = *index + 1;

    // Skip empty lines
    if line.trim().is_empty() {
        return Ok(None);
    }

    // Don't parse conditional keywords here (handled by parent)
    let trimmed = line.trim();
    if trimmed == "else" || trimmed == "endif" ||
       trimmed.starts_with("ifeq ") || trimmed.starts_with("ifneq ") ||
       trimmed.starts_with("ifdef ") || trimmed.starts_with("ifndef ") {
        return Ok(None);
    }

    // Parse variable assignment
    if is_variable_assignment(line) {
        let var = parse_variable(line, line_num)?;
        *index += 1; // Move past this variable line
        return Ok(Some(var));
    }

    // Parse target rule (THIS INCREMENTS INDEX - it advances past recipes)
    if line.contains(':') && !line.trim_start().starts_with('\t') {
        let target = parse_target_rule(lines, index)?;
        // parse_target_rule already incremented index past the target and its recipe
        // So we DON'T increment it again
        return Ok(Some(target));
    }

    // Parse comment
    if line.trim_start().starts_with('#') {
        let text = line.trim_start()
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
fn parse_target_rule(lines: &[&str], index: &mut usize) -> Result<MakeItem, String> {
    let line = lines[*index];
    let line_num = *index + 1;

    // Split on ':' to get target and prerequisites
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid target rule syntax at line {}", line_num));
    }

    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return Err(format!("Empty target name at line {}", line_num));
    }

    // Parse prerequisites (space-separated)
    let prerequisites: Vec<String> = parts[1]
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // Parse recipe lines (tab-indented lines following the target)
    *index += 1;
    let mut recipe = Vec::new();

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

    // Check if this is a pattern rule (target contains %)
    if name.contains('%') {
        Ok(MakeItem::PatternRule {
            target_pattern: name,
            prereq_patterns: prerequisites,
            recipe,
            span: Span::new(0, line.len(), line_num),
        })
    } else {
        Ok(MakeItem::Target {
            name,
            prerequisites,
            recipe,
            phony: false, // Will be detected in semantic analysis
            span: Span::new(0, line.len(), line_num),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
