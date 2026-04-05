// REPL Linter Integration Module
//
// Task: REPL-006-001 - Run linter from REPL
// Test Approach: RED → GREEN → REFACTOR → INTEGRATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Integration tests: CLI workflow
// - Complexity: <10 per function

use crate::linter::{lint_shell, LintResult, Severity};

/// Lint bash input and return diagnostics
///
/// # Examples
///
/// ```
/// use bashrs::repl::linter::lint_bash;
///
/// let result = lint_bash("cat file.txt | grep pattern");
/// assert!(result.is_ok());
/// ```
pub fn lint_bash(input: &str) -> anyhow::Result<LintResult> {
    let result = lint_shell(input);
    Ok(result)
}

/// Format lint results for display in REPL
pub fn format_lint_results(result: &LintResult) -> String {
    let mut output = String::new();

    if result.diagnostics.is_empty() {
        output.push_str("✓ No issues found!\n");
        return output;
    }

    // Count by severity
    let errors = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let info = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();

    output.push_str(&format!("Found {} issue(s):\n", result.diagnostics.len()));
    if errors > 0 {
        output.push_str(&format!("  ✗ {} error(s)\n", errors));
    }
    if warnings > 0 {
        output.push_str(&format!("  ⚠ {} warning(s)\n", warnings));
    }
    if info > 0 {
        output.push_str(&format!("  ℹ {} info\n", info));
    }

    output.push('\n');

    // Show diagnostics
    for (i, diag) in result.diagnostics.iter().enumerate() {
        let severity_icon = match diag.severity {
            Severity::Error => "✗",
            Severity::Warning => "⚠",
            Severity::Info => "ℹ",
            Severity::Note => "📝",
            Severity::Perf => "⚡",
            Severity::Risk => "⚠",
        };

        output.push_str(&format!(
            "[{}] {} {} - {}\n",
            i + 1,
            severity_icon,
            diag.code,
            diag.message
        ));

        if diag.span.start_line > 0 {
            output.push_str(&format!("    Line {}\n", diag.span.start_line));
        }
    }

    output
}

/// Format lint violations with source code context (REPL-014-003)
///
/// Displays each violation with:
/// - Line numbers (±2 lines of context)
/// - Source code at that location
/// - Visual indicator (caret) pointing to the issue
/// - Diagnostic message with rule code
/// - Fix suggestion if available
///
/// # Examples
///
/// ```no_run
/// use bashrs::repl::linter::{format_violations_with_context, lint_bash};
///
/// let source = "echo $RANDOM\nmkdir /app\n";
/// let result = lint_bash(source).unwrap();
/// let formatted = format_violations_with_context(&result, source);
/// ```
pub fn format_violations_with_context(result: &LintResult, source: &str) -> String {
    let mut output = String::new();

    if result.diagnostics.is_empty() {
        return "✓ No violations\n".to_string();
    }

    let lines: Vec<&str> = source.lines().collect();
    let max_line_num = lines.len();
    let line_num_width = max_line_num.to_string().len().max(3);

    for diagnostic in &result.diagnostics {
        let line_idx = diagnostic.span.start_line.saturating_sub(1);

        // Show context: ±2 lines
        let start_line = line_idx.saturating_sub(2);
        let end_line = (line_idx + 3).min(lines.len());

        output.push('\n');

        // Show context lines
        for i in start_line..end_line {
            if i < lines.len() {
                let line_num = i + 1;
                let prefix = if i == line_idx { ">" } else { " " };
                if let Some(line) = lines.get(i) {
                    output.push_str(&format!(
                        "{} {:>width$} | {}\n",
                        prefix,
                        line_num,
                        line,
                        width = line_num_width
                    ));

                    // Show indicator on the problematic line
                    if i == line_idx {
                        let col = diagnostic.span.start_col.saturating_sub(1);
                        let indicator_width =
                            if diagnostic.span.end_line == diagnostic.span.start_line {
                                diagnostic
                                    .span
                                    .end_col
                                    .saturating_sub(diagnostic.span.start_col)
                                    .max(1)
                            } else {
                                line.len().saturating_sub(col).max(1)
                            };

                        output.push_str(&format!(
                            "  {:>width$} | {}{} {} [{}]: {}\n",
                            "",
                            " ".repeat(col),
                            "^".repeat(indicator_width),
                            diagnostic.severity,
                            diagnostic.code,
                            diagnostic.message,
                            width = line_num_width
                        ));
                    }
                }
            }
        }

        // Show fix suggestion if available
        if let Some(fix) = &diagnostic.fix {
            output.push_str("\n  Suggested fix:\n");
            output.push_str(&format!(
                "  {:>width$} | {}\n",
                line_idx + 1,
                fix.replacement,
                width = line_num_width
            ));
        }
    }

    output
}

#[cfg(test)]
#[path = "linter_tests_repl_006.rs"]
mod tests_extracted;
