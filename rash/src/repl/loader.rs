//! REPL Script Loading Module
//!
//! Task: REPL-009-001 - Script loading and sourcing
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! Quality targets:
//! - Unit tests: 12+ scenarios
//! - Integration tests: Script loading workflows
//! - Mutation score: ≥90%
//! - Complexity: <10 per function

use crate::bash_parser::{BashParser, BashStmt};
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a loaded script
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedScript {
    /// Path to the script file
    pub path: PathBuf,
    /// Number of lines in the script
    pub line_count: usize,
    /// Function names extracted from the script
    pub functions: Vec<String>,
    /// Whether the script parsed successfully
    pub parsed_ok: bool,
}

/// Result of loading a script
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadResult {
    /// Script loaded successfully
    Success(LoadedScript),
    /// File not found or cannot be read
    FileError(String),
    /// Script has parse errors
    ParseError(String),
}

impl LoadResult {
    /// Format the load result for display
    pub fn format(&self) -> String {
        match self {
            LoadResult::Success(script) => {
                let func_str = if script.functions.is_empty() {
                    "no functions".to_string()
                } else if script.functions.len() == 1 {
                    "1 function".to_string()
                } else {
                    format!("{} functions", script.functions.len())
                };

                format!(
                    "✓ Loaded: {} ({}, {} lines)",
                    script.path.display(),
                    func_str,
                    script.line_count
                )
            }
            LoadResult::FileError(msg) => format!("✗ Error: {}", msg),
            LoadResult::ParseError(msg) => format!("✗ Parse error: {}", msg),
        }
    }
}

/// Load a bash script from a file
///
/// Reads the file, parses it, and extracts function names.
///
/// # Examples
///
/// ```rust,no_run
/// use bashrs::repl::loader::load_script;
///
/// let result = load_script("examples/test.sh");
/// println!("{}", result.format());
/// ```
pub fn load_script<P: AsRef<Path>>(path: P) -> LoadResult {
    let path_ref = path.as_ref();

    // Read the file
    let content = match fs::read_to_string(path_ref) {
        Ok(c) => c,
        Err(e) => {
            return LoadResult::FileError(format!(
                "Cannot read file {}: {}",
                path_ref.display(),
                e
            ));
        }
    };

    // Count lines
    let line_count = content.lines().count();

    // Parse the script
    let mut parser = match BashParser::new(&content) {
        Ok(p) => p,
        Err(e) => {
            return LoadResult::ParseError(format!("Failed to create parser: {}", e));
        }
    };

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            return LoadResult::ParseError(format!("Parse error: {}", e));
        }
    };

    // Extract function names
    let functions = extract_functions(&ast.statements);

    LoadResult::Success(LoadedScript {
        path: path_ref.to_path_buf(),
        line_count,
        functions,
        parsed_ok: true,
    })
}

/// Extract function names from AST statements
fn extract_functions(statements: &[BashStmt]) -> Vec<String> {
    let mut functions = Vec::new();

    for stmt in statements {
        if let BashStmt::Function { name, .. } = stmt {
            functions.push(name.clone());
        }
    }

    functions
}

/// Format a list of functions for display
pub fn format_functions(functions: &[String]) -> String {
    if functions.is_empty() {
        return "No functions available".to_string();
    }

    let mut output = format!("Available functions ({} total):\n", functions.len());
    for (i, func) in functions.iter().enumerate() {
        output.push_str(&format!("  {} {}\n", i + 1, func));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ===== RED PHASE: Unit Tests =====

    #[test]
    fn test_REPL_009_001_load_valid_script() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "#!/bin/bash\necho hello").unwrap();
        temp_file.flush().unwrap();

        let result = load_script(temp_file.path());

        match result {
            LoadResult::Success(script) => {
                assert_eq!(script.line_count, 2);
                assert_eq!(script.functions.len(), 0);
                assert!(script.parsed_ok);
            }
            _ => panic!("Expected Success, got {:?}", result),
        }
    }

    #[test]
    fn test_REPL_009_001_load_script_with_function() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "#!/bin/bash\ngreet() {{\n  echo \"Hello\"\n}}").unwrap();
        temp_file.flush().unwrap();

        let result = load_script(temp_file.path());

        match result {
            LoadResult::Success(script) => {
                assert_eq!(script.functions.len(), 1);
                assert_eq!(script.functions[0], "greet");
            }
            _ => panic!("Expected Success, got {:?}", result),
        }
    }

    #[test]
    fn test_REPL_009_001_load_script_multiple_functions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "#!/bin/bash").unwrap();
        writeln!(temp_file, "func1() {{").unwrap();
        writeln!(temp_file, "  echo \"func1\"").unwrap();
        writeln!(temp_file, "}}").unwrap();
        writeln!(temp_file, "func2() {{").unwrap();
        writeln!(temp_file, "  echo \"func2\"").unwrap();
        writeln!(temp_file, "}}").unwrap();
        writeln!(temp_file, "func3() {{").unwrap();
        writeln!(temp_file, "  echo \"func3\"").unwrap();
        writeln!(temp_file, "}}").unwrap();
        temp_file.flush().unwrap();

        let result = load_script(temp_file.path());

        match result {
            LoadResult::Success(script) => {
                assert_eq!(script.functions.len(), 3);
                assert!(script.functions.contains(&"func1".to_string()));
                assert!(script.functions.contains(&"func2".to_string()));
                assert!(script.functions.contains(&"func3".to_string()));
            }
            _ => panic!("Expected Success, got {:?}", result),
        }
    }

    #[test]
    fn test_REPL_009_001_load_nonexistent_file() {
        let result = load_script("/nonexistent/file/path.sh");

        match result {
            LoadResult::FileError(msg) => {
                assert!(msg.contains("Cannot read file"));
            }
            _ => panic!("Expected FileError, got {:?}", result),
        }
    }

    #[test]
    fn test_REPL_009_001_load_invalid_syntax() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "if then fi").unwrap();
        temp_file.flush().unwrap();

        let result = load_script(temp_file.path());

        match result {
            LoadResult::ParseError(_) => {
                // Expected
            }
            _ => panic!("Expected ParseError, got {:?}", result),
        }
    }

    #[test]
    fn test_REPL_009_001_load_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.flush().unwrap();

        let result = load_script(temp_file.path());

        match result {
            LoadResult::Success(script) => {
                assert_eq!(script.line_count, 0);
                assert_eq!(script.functions.len(), 0);
            }
            _ => panic!("Expected Success, got {:?}", result),
        }
    }

    #[test]
    fn test_REPL_009_001_extract_functions_empty() {
        let functions = extract_functions(&[]);

        assert_eq!(functions.len(), 0);
    }

    #[test]
    fn test_REPL_009_001_extract_functions_no_functions() {
        let statements = vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Literal("hello".to_string())],
            span: Span::new(1, 1, 1, 11),
        }];

        let functions = extract_functions(&statements);

        assert_eq!(functions.len(), 0);
    }

    #[test]
    fn test_REPL_009_001_extract_functions_single() {
        let statements = vec![BashStmt::Function {
            name: "test_func".to_string(),
            body: vec![],
            span: Span::new(1, 1, 1, 20),
        }];

        let functions = extract_functions(&statements);

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0], "test_func");
    }

    #[test]
    fn test_REPL_009_001_format_success() {
        let script = LoadedScript {
            path: PathBuf::from("/test/script.sh"),
            line_count: 10,
            functions: vec!["func1".to_string(), "func2".to_string()],
            parsed_ok: true,
        };

        let formatted = LoadResult::Success(script).format();

        assert!(formatted.contains("Loaded"));
        assert!(formatted.contains("2 functions"));
        assert!(formatted.contains("10 lines"));
    }

    #[test]
    fn test_REPL_009_001_format_file_error() {
        let formatted = LoadResult::FileError("File not found".to_string()).format();

        assert!(formatted.contains("Error"));
        assert!(formatted.contains("File not found"));
    }

    #[test]
    fn test_REPL_009_001_format_parse_error() {
        let formatted = LoadResult::ParseError("Syntax error".to_string()).format();

        assert!(formatted.contains("Parse error"));
        assert!(formatted.contains("Syntax error"));
    }

    #[test]
    fn test_REPL_009_001_format_functions_empty() {
        let formatted = format_functions(&[]);

        assert!(formatted.contains("No functions"));
    }

    #[test]
    fn test_REPL_009_001_format_functions_list() {
        let functions = vec![
            "func1".to_string(),
            "func2".to_string(),
            "func3".to_string(),
        ];

        let formatted = format_functions(&functions);

        assert!(formatted.contains("3 total"));
        assert!(formatted.contains("func1"));
        assert!(formatted.contains("func2"));
        assert!(formatted.contains("func3"));
    }
}
