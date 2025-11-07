//! DOCKER003: Missing apt-get cleanup (image size optimization)

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Join continuation lines to handle multi-line RUN commands
    let mut current_run = String::new();
    let mut run_start_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Check if this is the start of a RUN command
        if trimmed.starts_with("RUN ") {
            run_start_line = line_num + 1;
            current_run = trimmed.to_string();
        } else if !current_run.is_empty() {
            // Continue building the RUN command if line ends with backslash
            if current_run.ends_with('\\') {
                current_run.push(' ');
                current_run.push_str(trimmed);
            } else {
                // RUN command complete, check it
                check_run_command(&current_run, run_start_line, &mut result);
                current_run.clear();
            }
        }

        // Check if this is a complete single-line RUN
        if trimmed.starts_with("RUN ") && !trimmed.ends_with('\\') {
            check_run_command(trimmed, line_num + 1, &mut result);
            current_run.clear();
        }
    }

    // Check final RUN if any
    if !current_run.is_empty() {
        check_run_command(&current_run, run_start_line, &mut result);
    }

    result
}

fn check_run_command(run_cmd: &str, line_num: usize, result: &mut LintResult) {
    if run_cmd.contains("apt-get install") {
        // Check if cleanup is present in the same RUN command
        let has_cleanup = run_cmd.contains("rm -rf /var/lib/apt");

        if !has_cleanup {
            let span = Span::new(line_num, 1, line_num, run_cmd.len().min(80));
            let diag = Diagnostic::new(
                "DOCKER003",
                Severity::Warning,
                "apt-get install without cleanup - add 'rm -rf /var/lib/apt/lists/*' to reduce image size".to_string(),
                span,
            );
            result.add(diag);
        }
    }
}
