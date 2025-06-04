/// Escape a string for safe use in shell scripts (public alias)
pub fn shell_escape(s: &str) -> String {
    escape_shell_string(s)
}

/// Escape a string for safe use in shell scripts
pub fn escape_shell_string(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }

    // Check if the string needs escaping
    if is_safe_unquoted(s) {
        return s.to_string();
    }

    // Use single quotes for simplicity and safety
    if !s.contains('\'') {
        return format!("'{}'", s);
    }

    // Handle strings with single quotes by escaping them
    let escaped = s.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

/// Escape a variable name for shell
pub fn escape_variable_name(name: &str) -> String {
    // Variable names should be valid shell identifiers
    if is_valid_shell_identifier(name) {
        name.to_string()
    } else {
        // Convert invalid characters to underscores
        let mut result = String::new();
        for (i, c) in name.chars().enumerate() {
            if i == 0 {
                // First character must be letter or underscore
                if c.is_alphabetic() || c == '_' {
                    result.push(c);
                } else {
                    result.push('_');
                    // Don't add the invalid first character - skip it
                }
            } else {
                // Subsequent characters can be alphanumeric or underscore
                if c.is_alphanumeric() || c == '_' {
                    result.push(c);
                } else {
                    result.push('_');
                }
            }
        }
        result
    }
}

/// Escape a command name for shell execution
pub fn escape_command_name(cmd: &str) -> String {
    // Commands should not contain special characters
    if is_safe_command_name(cmd) {
        cmd.to_string()
    } else {
        escape_shell_string(cmd)
    }
}

/// Check if a string is safe to use unquoted in shell
fn is_safe_unquoted(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start with alphanumeric or safe characters
    let first_char = s.chars().next().unwrap();
    if !first_char.is_alphanumeric() && first_char != '_' && first_char != '.' && first_char != '/'
    {
        return false;
    }

    // All characters must be safe
    s.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '/' | '-' | '+' | '=' | ':' | '@'))
}

/// Check if a string is a valid shell identifier
fn is_valid_shell_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start with letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    name.chars()
        .skip(1)
        .all(|c| c.is_alphanumeric() || c == '_')
}

/// Check if a command name is safe
fn is_safe_command_name(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }

    // Command names should be simple identifiers or paths
    cmd.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
        && !cmd.starts_with('-') // Commands shouldn't start with dash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_simple_string() {
        assert_eq!(escape_shell_string("hello"), "hello");
        assert_eq!(escape_shell_string("hello world"), "'hello world'");
        assert_eq!(escape_shell_string(""), "''");
    }

    #[test]
    fn test_escape_string_with_quotes() {
        assert_eq!(escape_shell_string("don't"), "'don'\"'\"'t'");
    }

    #[test]
    fn test_variable_name_escaping() {
        assert_eq!(escape_variable_name("valid_name"), "valid_name");
        assert_eq!(escape_variable_name("invalid-name"), "invalid_name");
        assert_eq!(escape_variable_name("123invalid"), "_23invalid");
    }

    #[test]
    fn test_command_name_escaping() {
        assert_eq!(escape_command_name("ls"), "ls");
        assert_eq!(escape_command_name("/bin/ls"), "/bin/ls");
        assert_eq!(escape_command_name("my command"), "'my command'");
    }

    #[test]
    fn test_safe_unquoted() {
        assert!(is_safe_unquoted("simple"));
        assert!(is_safe_unquoted("path/to/file"));
        assert!(is_safe_unquoted("version-1.0"));
        assert!(!is_safe_unquoted("has spaces"));
        assert!(!is_safe_unquoted("has$dollar"));
        assert!(!is_safe_unquoted(""));
    }
}
