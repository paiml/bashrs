//! DOCKER004: Invalid COPY --from reference (multi-stage build validation)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::collections::HashSet;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let stage_names = collect_stage_names(source);
    let diagnostics = validate_copy_from_references(source, &stage_names);

    for diag in diagnostics {
        result.add(diag);
    }

    result
}

/// Collect all stage names defined in FROM ... AS <name> directives
fn collect_stage_names(source: &str) -> HashSet<String> {
    let mut stage_names = HashSet::new();

    for line in source.lines() {
        if let Some(name) = parse_stage_name(line) {
            stage_names.insert(name);
        }
    }

    stage_names
}

/// Parse stage name from FROM ... AS <name> line
fn parse_stage_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if trimmed.starts_with("FROM ") && trimmed.contains(" AS ") {
        if let Some(as_pos) = trimmed.rfind(" AS ") {
            let stage_name = trimmed[as_pos + 4..].trim();
            return Some(stage_name.to_string());
        }
    }

    None
}

/// Validate all COPY --from references and return diagnostics for invalid ones
fn validate_copy_from_references(source: &str, stage_names: &HashSet<String>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(diag) = check_copy_from_line(line_num, line, stage_names) {
            diagnostics.push(diag);
        }
    }

    diagnostics
}

/// Check a single COPY --from line for valid stage reference
fn check_copy_from_line(
    line_num: usize,
    line: &str,
    stage_names: &HashSet<String>,
) -> Option<Diagnostic> {
    let trimmed = line.trim();

    if !trimmed.starts_with("COPY ") || !trimmed.contains("--from=") {
        return None;
    }

    let stage_ref = parse_copy_from_reference(trimmed)?;

    // Numeric indices (0, 1, 2...) are always valid
    if stage_ref.parse::<usize>().is_ok() {
        return None;
    }

    // Check if stage name exists
    if stage_names.contains(stage_ref) {
        return None;
    }

    // Invalid reference - create diagnostic
    let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
    Some(Diagnostic::new(
        "DOCKER004",
        Severity::Warning,
        format!(
            "Invalid COPY --from='{}' - stage not defined. Available stages: {:?}",
            stage_ref, stage_names
        ),
        span,
    ))
}

/// Parse stage reference from COPY --from=<reference> line
fn parse_copy_from_reference(line: &str) -> Option<&str> {
    let from_pos = line.find("--from=")?;
    let after_from = &line[from_pos + 7..];
    let stage_ref = after_from.split_whitespace().next().unwrap_or("");

    if stage_ref.is_empty() {
        None
    } else {
        Some(stage_ref)
    }
}
