/// Get a human-readable type name for a JSON value
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Parse JSONC (JSON with Comments) to serde_json::Value
///
/// Strips single-line (//) and multi-line (/* */) comments before parsing.
pub fn parse_jsonc(content: &str) -> Result<Value, String> {
    let stripped = strip_json_comments(content);
    serde_json::from_str(&stripped).map_err(|e| format!("Invalid JSON: {}", e))
}

/// Strip comments from JSONC content
fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }

        if !in_string && ch == '/' && skip_comment(&mut chars) {
            continue;
        }

        result.push(ch);
    }

    result
}

/// Try to skip a comment starting after '/'. Returns true if a comment was skipped.
fn skip_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    match chars.peek() {
        Some(&'/') => {
            chars.next(); // consume second /
            skip_single_line_comment(chars);
            true
        }
        Some(&'*') => {
            chars.next(); // consume *
            skip_multi_line_comment(chars);
            true
        }
        _ => false,
    }
}

/// Skip to end of single-line comment (until newline)
fn skip_single_line_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = chars.peek() {
        if c == '\n' {
            break;
        }
        chars.next();
    }
}

/// Skip to end of multi-line comment (until */)
fn skip_multi_line_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(c) = chars.next() {
        if c == '*' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                break;
            }
        }
    }
}

/// Run all Dev Container validation rules on parsed JSON
pub fn lint_devcontainer(json: &Value) -> LintResult {
    let mut result = LintResult::new();

    result.merge(check_devcontainer001(json));
    result.merge(check_devcontainer002(json));
    result.merge(check_devcontainer003(json));
    result.merge(check_devcontainer004(json));
    result.merge(check_devcontainer005(json));
    result.merge(check_devcontainer006(json));
    result.merge(check_devcontainer007(json));
    result.merge(check_devcontainer008(json));
    result.merge(check_devcontainer009(json));
    result.merge(check_devcontainer010(json));
    result.merge(check_devcontainer011(json));

    result
}

/// Validate devcontainer.json content (JSONC string)
pub fn validate_devcontainer(content: &str) -> Result<LintResult, String> {
    let json = parse_jsonc(content)?;
    Ok(lint_devcontainer(&json))
}

/// List all DEVCONTAINER rules
pub fn list_devcontainer_rules() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "DEVCONTAINER001",
            "Missing image source (image, build, or dockerComposeFile)",
        ),
        (
            "DEVCONTAINER002",
            "Using :latest tag reduces reproducibility",
        ),
        ("DEVCONTAINER003", "Absolute path in build.dockerfile"),
        (
            "DEVCONTAINER004",
            "Docker Compose requires 'service' property",
        ),
        ("DEVCONTAINER005", "Unknown feature option"),
        ("DEVCONTAINER006", "Duplicate keys in lifecycle command"),
        ("DEVCONTAINER007", "Invalid waitFor value"),
        (
            "DEVCONTAINER008",
            "updateRemoteUserUID=false may cause permission issues",
        ),
        ("DEVCONTAINER009", "workspaceFolder must be absolute path"),
        ("DEVCONTAINER010", "containerEnv values must be strings"),
        ("DEVCONTAINER011", "Invalid VS Code extension ID format"),
    ]
}


include!("devcontainer_tests_cont.rs");
