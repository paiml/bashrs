//! Translation Patterns for Bash-to-Rash
//!
//! Defines mapping patterns from bash constructs to Rash code.

/// Pattern for translating bash variable assignments
pub struct VariablePattern {
    pub exported: bool,
}

impl VariablePattern {
    pub fn to_rash(&self, name: &str, value_rash: &str) -> String {
        if self.exported {
            format!(
                "let {} = {};\nstd::env::set_var(\"{}\", {});",
                name, value_rash, name, name
            )
        } else {
            format!("let {} = {};", name, value_rash)
        }
    }
}

/// Pattern for translating bash commands
pub struct CommandPattern;

impl CommandPattern {
    pub fn to_rash(cmd: &str, args: &[String]) -> String {
        if args.is_empty() {
            format!("{}();", cmd)
        } else {
            let args_str = args.join(", ");
            format!("{}({});", cmd, args_str)
        }
    }
}

/// Pattern for translating bash if statements
pub struct IfPattern;

impl IfPattern {
    pub fn to_rash(
        condition: &str,
        then_block: &str,
        elif_blocks: &[(String, String)],
        else_block: Option<&str>,
    ) -> String {
        let mut result = format!("if {} {{\n{}}}", condition, then_block);

        for (elif_cond, elif_body) in elif_blocks {
            result.push_str(&format!(" else if {} {{\n{}}}", elif_cond, elif_body));
        }

        if let Some(else_body) = else_block {
            result.push_str(&format!(" else {{\n{}}}", else_body));
        }

        result
    }
}

/// Pattern for translating bash for loops
pub struct ForPattern;

impl ForPattern {
    pub fn to_rash(variable: &str, items: &str, body: &str) -> String {
        format!("for {} in {} {{\n{}}}", variable, items, body)
    }
}

/// Pattern for translating bash while loops
pub struct WhilePattern;

impl WhilePattern {
    pub fn to_rash(condition: &str, body: &str) -> String {
        format!("while {} {{\n{}}}", condition, body)
    }
}

/// Pattern for translating bash functions
pub struct FunctionPattern;

impl FunctionPattern {
    pub fn to_rash(name: &str, body: &str) -> String {
        format!("fn {}() {{\n{}}}", name, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_pattern() {
        let pattern = VariablePattern { exported: false };
        let rash = pattern.to_rash("FOO", "\"bar\"");
        assert!(rash.contains("let FOO = \"bar\";"));
    }

    #[test]
    fn test_exported_variable_pattern() {
        let pattern = VariablePattern { exported: true };
        let rash = pattern.to_rash("PATH", "\"/usr/bin\"");
        assert!(rash.contains("std::env::set_var"));
    }

    #[test]
    fn test_command_pattern() {
        let rash = CommandPattern::to_rash("echo", &["\"hello\"".to_string()]);
        assert!(rash.contains("echo(\"hello\")"));
    }

    #[test]
    fn test_if_pattern() {
        let rash = IfPattern::to_rash("x == 1", "    println!(\"one\");", &[], None);
        assert!(rash.contains("if x == 1"));
    }

    #[test]
    fn test_for_pattern() {
        let rash = ForPattern::to_rash("i", "0..10", "    println!(\"{}\", i);");
        assert!(rash.contains("for i in 0..10"));
    }
}
