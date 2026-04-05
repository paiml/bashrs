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


include!("parser_parse.rs");
