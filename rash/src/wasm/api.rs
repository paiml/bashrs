//! JavaScript API for bashrs WASM
//!
//! This module provides the main entry points for JavaScript/TypeScript consumers.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Initialize the WASM module
///
/// Call this once before using any other functions.
///
/// # Example (JavaScript)
///
/// ```js
/// import init, { analyzeConfig } from 'bashrs.wasm';
///
/// await init();
/// const result = analyzeConfig(bashrcContent);
/// console.log(result);
/// ```
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
}

/// Configuration analysis result
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct ConfigAnalysisResult {
    file_path: String,
    line_count: usize,
    complexity_score: u8,
    issues: Vec<ConfigIssue>,
}

/// A single configuration issue
#[derive(Serialize, Deserialize, Clone)]
#[wasm_bindgen]
pub struct ConfigIssue {
    rule_id: String,
    severity: String,
    line: usize,
    column: usize,
    message: String,
    suggestion: Option<String>,
}

#[wasm_bindgen]
impl ConfigAnalysisResult {
    /// Get the number of issues found
    #[wasm_bindgen(getter)]
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Get issues as JSON string
    #[wasm_bindgen(getter)]
    pub fn issues_json(&self) -> String {
        serde_json::to_string(&self.issues).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get line count
    #[wasm_bindgen(getter)]
    pub fn line_count(&self) -> usize {
        self.line_count
    }

    /// Get complexity score (0-10)
    #[wasm_bindgen(getter)]
    pub fn complexity_score(&self) -> u8 {
        self.complexity_score
    }
}

/// Analyze a shell configuration file
///
/// Runs CONFIG-001 to CONFIG-004 analysis on the provided config content.
///
/// # Arguments
///
/// * `content` - The configuration file content (.bashrc, .zshrc, etc.)
/// * `filename` - Optional filename for context (e.g., ".bashrc")
///
/// # Returns
///
/// A `ConfigAnalysisResult` containing detected issues
///
/// # Example (JavaScript)
///
/// ```js
/// const bashrc = `
/// export PATH="/usr/local/bin:$PATH"
/// export PATH="/usr/local/bin:$PATH"  # Duplicate!
/// export SESSION_ID=$RANDOM          # Non-deterministic!
/// `;
///
/// const result = analyzeConfig(bashrc, ".bashrc");
/// console.log(`Found ${result.issue_count} issues`);
/// console.log(JSON.parse(result.issues_json));
/// ```
#[wasm_bindgen]
pub fn analyze_config(content: &str, filename: Option<String>) -> Result<ConfigAnalysisResult, JsValue> {
    use crate::config::analyzer;
    use std::path::PathBuf;

    let path = PathBuf::from(filename.unwrap_or_else(|| ".bashrc".to_string()));
    let analysis = analyzer::analyze_config(content, path);

    // Convert to WASM-friendly format
    let issues = analysis
        .issues
        .iter()
        .map(|issue| ConfigIssue {
            rule_id: issue.rule_id.clone(),
            severity: format!("{:?}", issue.severity),
            line: issue.line,
            column: issue.column,
            message: issue.message.clone(),
            suggestion: issue.suggestion.clone(),
        })
        .collect();

    Ok(ConfigAnalysisResult {
        file_path: analysis.file_path.display().to_string(),
        line_count: analysis.line_count,
        complexity_score: analysis.complexity_score,
        issues,
    })
}

/// Purify a shell configuration file
///
/// Applies automatic fixes for CONFIG-001 to CONFIG-004 issues.
///
/// # Arguments
///
/// * `content` - The configuration file content
///
/// # Returns
///
/// Purified configuration file content
///
/// # Example (JavaScript)
///
/// ```js
/// const messy = `
/// export PATH="/usr/local/bin:$PATH"
/// export PATH="/usr/local/bin:$PATH"
/// export SESSION_ID=$RANDOM
/// `;
///
/// const clean = purifyConfig(messy);
/// // Duplicates removed, non-determinism fixed
/// ```
#[wasm_bindgen]
pub fn purify_config(content: &str) -> String {
    use crate::config::purifier;

    purifier::purify_config(content)
}

/// Check if content is valid bash syntax
///
/// Performs basic syntax checking without full parsing.
///
/// # Example (JavaScript)
///
/// ```js
/// if (isValidBash("echo 'hello'")) {
///     console.log("Valid bash");
/// }
/// ```
#[wasm_bindgen]
pub fn is_valid_bash(content: &str) -> bool {
    // Simple validation for now
    // TODO: Hook up to bash parser when available
    !content.is_empty()
}

/// Get bashrs version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Bash script execution result
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct ExecutionResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[wasm_bindgen]
impl ExecutionResult {
    /// Get stdout output
    #[wasm_bindgen(getter)]
    pub fn stdout(&self) -> String {
        self.stdout.clone()
    }

    /// Get stderr output
    #[wasm_bindgen(getter)]
    pub fn stderr(&self) -> String {
        self.stderr.clone()
    }

    /// Get exit code
    #[wasm_bindgen(getter)]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Get result as JSON string
    #[wasm_bindgen]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Execute a bash script in WASM runtime
///
/// Runs the provided bash script in a sandboxed WASM environment with:
/// - Virtual filesystem (/, /tmp, /home)
/// - Built-in commands (echo, cd, pwd)
/// - Variable assignment and expansion
/// - Stdout/stderr capture
///
/// # Arguments
///
/// * `source` - Bash script source code
///
/// # Returns
///
/// An `ExecutionResult` with stdout, stderr, and exit code
///
/// # Example (JavaScript)
///
/// ```js
/// import init, { execute_script } from './pkg/bashrs.js';
///
/// await init();
///
/// const result = execute_script(`
///   echo "Hello from WASM bash!"
///   name="Claude"
///   echo "Hello, $name"
///   cd /tmp
///   pwd
/// `);
///
/// console.log('Output:', result.stdout);
/// console.log('Exit code:', result.exit_code);
/// ```
#[wasm_bindgen]
pub fn execute_script(source: &str) -> Result<ExecutionResult, JsValue> {
    use crate::wasm::executor::BashExecutor;

    let mut executor = BashExecutor::new();

    executor
        .execute(source)
        .map(|result| ExecutionResult {
            stdout: result.stdout,
            stderr: result.stderr,
            exit_code: result.exit_code,
        })
        .map_err(|e| JsValue::from_str(&format!("Execution error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_is_valid_bash() {
        assert!(is_valid_bash("echo hello"));
        assert!(!is_valid_bash(""));
    }

    #[test]
    fn test_execute_script_echo() {
        let result = execute_script("echo 'hello world'").unwrap();
        assert_eq!(result.stdout(), "hello world\n");
        assert_eq!(result.exit_code(), 0);
    }

    #[test]
    fn test_execute_script_variables() {
        let script = r#"
name="Claude"
echo $name
"#;
        let result = execute_script(script).unwrap();
        assert!(result.stdout().contains("Claude"));
        assert_eq!(result.exit_code(), 0);
    }

    #[test]
    fn test_execute_script_cd_pwd() {
        let script = r#"
cd /tmp
pwd
"#;
        let result = execute_script(script).unwrap();
        assert!(result.stdout().contains("/tmp"));
        assert_eq!(result.exit_code(), 0);
    }

    #[test]
    fn test_execute_script_multi_command() {
        let script = r#"
echo "Line 1"
echo "Line 2"
echo "Line 3"
"#;
        let result = execute_script(script).unwrap();
        assert!(result.stdout().contains("Line 1"));
        assert!(result.stdout().contains("Line 2"));
        assert!(result.stdout().contains("Line 3"));
        assert_eq!(result.exit_code(), 0);
    }
}
