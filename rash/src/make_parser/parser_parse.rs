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
#[path = "parser_tests_parse_empty.rs"]
mod tests_extracted;
