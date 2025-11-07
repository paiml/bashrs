//! DOCKER004: Invalid COPY --from reference (multi-stage build validation)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::collections::HashSet;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Collect all stage names
    let mut stage_names = HashSet::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("FROM ") && trimmed.contains(" AS ") {
            if let Some(as_pos) = trimmed.rfind(" AS ") {
                let stage_name = trimmed[as_pos + 4..].trim();
                stage_names.insert(stage_name.to_string());
            }
        }
    }

    // Check COPY --from references
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("COPY ") && trimmed.contains("--from=") {
            if let Some(from_pos) = trimmed.find("--from=") {
                let after_from = &trimmed[from_pos + 7..];
                let stage_ref = after_from.split_whitespace().next().unwrap_or("");

                // Check if it's a numeric index (0, 1, 2...) - always valid
                if stage_ref.parse::<usize>().is_ok() {
                    continue;
                }

                // Check if stage name exists
                if !stage_names.contains(stage_ref) {
                    let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                    let diag = Diagnostic::new(
                        "DOCKER004",
                        Severity::Warning,
                        format!(
                            "Invalid COPY --from='{}' - stage not defined. Available stages: {:?}",
                            stage_ref, stage_names
                        ),
                        span,
                    );
                    result.add(diag);
                }
            }
        }
    }

    result
}
