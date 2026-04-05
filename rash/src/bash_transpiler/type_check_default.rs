/// Parse a declare flag (-i, -a, -A) into a ShellType
fn parse_declare_flag(s: &str) -> Option<ShellType> {
    match s {
        "-i" => Some(ShellType::Integer),
        "-a" => Some(ShellType::Array(Box::new(ShellType::String))),
        "-A" => Some(ShellType::AssocArray {
            key: Box::new(ShellType::String),
            value: Box::new(ShellType::String),
        }),
        _ => None,
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a type annotation from a comment string
///
/// Supported formats:
/// - `@type varname: int`
/// - `@type varname: str`
/// - `@type varname: path`
/// - `@type varname: bool`
/// - `@type varname: array`
/// - `@param name: int`
/// - `@returns: int`
pub fn parse_type_annotation(comment: &str) -> Option<TypeAnnotation> {
    let trimmed = comment.trim();

    // @type varname: type
    if let Some(rest) = trimmed.strip_prefix("@type ") {
        let (name, ty, hint) = parse_name_type(rest)?;
        return Some(TypeAnnotation {
            name,
            shell_type: ty,
            type_hint: hint,
            is_return: false,
            is_param: false,
        });
    }

    // @param name: type
    if let Some(rest) = trimmed.strip_prefix("@param ") {
        let (name, ty, hint) = parse_name_type(rest)?;
        return Some(TypeAnnotation {
            name,
            shell_type: ty,
            type_hint: hint,
            is_return: false,
            is_param: true,
        });
    }

    // @returns: type
    if let Some(rest) = trimmed.strip_prefix("@returns: ") {
        let raw_type = rest.trim().to_string();
        let ty = parse_type_name(&raw_type)?;
        return Some(TypeAnnotation {
            name: String::new(),
            shell_type: ty,
            type_hint: raw_type,
            is_return: true,
            is_param: false,
        });
    }

    None
}

/// Parse "name: type" from annotation text, returning (name, ShellType, raw_type_name)
fn parse_name_type(text: &str) -> Option<(String, ShellType, String)> {
    let parts: Vec<&str> = text.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let name = parts[0].trim().to_string();
    let raw_type = parts[1].trim().to_string();
    let ty = parse_type_name(&raw_type)?;
    Some((name, ty, raw_type))
}

/// Parse a type name string into a ShellType
pub fn parse_type_name(name: &str) -> Option<ShellType> {
    match name {
        "int" | "integer" => Some(ShellType::Integer),
        "str" | "string" => Some(ShellType::String),
        "bool" | "boolean" => Some(ShellType::Boolean),
        "path" => Some(ShellType::String), // Path is a string subtype for now
        "array" => Some(ShellType::Array(Box::new(ShellType::String))),
        "fd" => Some(ShellType::FileDescriptor),
        "exit_code" => Some(ShellType::ExitCode),
        _ => None,
    }
}

/// Check gradual compatibility — untyped is compatible with everything
fn is_gradual_compatible(expected: &ShellType, actual: &ShellType) -> bool {
    // Integer is compatible with String context (integers are valid strings)
    // But NOT the reverse — String→Integer should warn (not every string is a number)
    matches!((expected, actual), (ShellType::String, ShellType::Integer))
}

/// Generate a POSIX sh runtime guard for an integer-typed variable
pub fn generate_integer_guard(var_name: &str) -> String {
    format!(
        r#"case "${var}" in
    *[!0-9]*) echo "type error: {var} must be integer" >&2; exit 1 ;;
esac"#,
        var = var_name
    )
}

/// Generate a POSIX sh runtime guard for a path-typed variable
pub fn generate_path_guard(var_name: &str) -> String {
    format!(
        r#"case "${var}" in
    /*|./*|../*) ;;
    *) echo "type error: {var} must be a path" >&2; exit 1 ;;
esac"#,
        var = var_name
    )
}

/// Generate a POSIX sh runtime guard for a non-empty string
pub fn generate_nonempty_guard(var_name: &str) -> String {
    format!(
        r#"if [ -z "${var}" ]; then
    echo "type error: {var} must be non-empty string" >&2; exit 1
fi"#,
        var = var_name
    )
}

/// Generate a runtime guard for a typed variable.
/// `hint` is the original annotation name (e.g., "path") to distinguish subtypes.
pub fn generate_guard_for_type(
    var_name: &str,
    ty: &ShellType,
    hint: Option<&str>,
) -> Option<String> {
    match ty {
        ShellType::Integer => Some(generate_integer_guard(var_name)),
        ShellType::String => {
            if hint == Some("path") {
                Some(generate_path_guard(var_name))
            } else {
                Some(generate_nonempty_guard(var_name))
            }
        }
        ShellType::Boolean => None,
        ShellType::Array(_) => None,
        ShellType::AssocArray { .. } => None,
        ShellType::FileDescriptor => None,
        ShellType::ExitCode => None,
        ShellType::Signal => None,
        ShellType::TypeVar(_) => None,
        ShellType::Union(_) => None,
    }
}

#[cfg(test)]
#[path = "type_check_tests.rs"]
mod tests;
