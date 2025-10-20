//! MAKE006: Missing target dependencies
//!
//! **Rule**: Detect when targets don't declare necessary dependencies
//!
//! **Why this matters**:
//! Targets should explicitly declare their dependencies to ensure correct
//! build order and enable parallel builds. Missing dependencies can lead to
//! race conditions and incorrect builds.
//!
//! **Auto-fix**: Suggest common dependencies based on recipe commands
//!
//! ## Examples
//!
//! ❌ **BAD** (missing dependencies):
//! ```makefile
//! app:
//! 	gcc main.c utils.c -o app
//! ```
//!
//! ✅ **GOOD** (dependencies declared):
//! ```makefile
//! app: main.c utils.c
//! 	gcc main.c utils.c -o app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use std::collections::HashSet;

/// Check for targets with missing dependencies
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let phony_targets = find_phony_targets(source);
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if let Some((target_info, next_line)) = parse_target_line(&lines, i, &phony_targets) {
            if let Some(diag) = check_target_dependencies(&target_info, &lines, i) {
                result.add(diag);
            }
            i = next_line;
        } else {
            i += 1;
        }
    }

    result
}

/// Information about a parsed target line
struct TargetInfo {
    name: String,
    declared_deps: HashSet<String>,
}

/// Parse a target line and extract target name and dependencies
fn parse_target_line<'a>(
    lines: &'a [&str],
    line_idx: usize,
    phony_targets: &HashSet<String>,
) -> Option<(TargetInfo, usize)> {
    let line = lines[line_idx];

    // Skip lines that aren't target definitions
    if should_skip_line(line) {
        return None;
    }

    // Find colon position
    let colon_pos = line.find(':')?;

    // Skip recipe lines (start with tab)
    if line.starts_with('\t') {
        return None;
    }

    // Extract target name
    let target = line[..colon_pos].trim();

    // Skip .PHONY targets
    if phony_targets.contains(target) {
        return None;
    }

    // Extract declared dependencies
    let deps_part = &line[colon_pos + 1..];
    let declared_deps: HashSet<String> = deps_part
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    Some((
        TargetInfo {
            name: target.to_string(),
            declared_deps,
        },
        line_idx + 1,
    ))
}

/// Check if a line should be skipped (not a target definition)
fn should_skip_line(line: &str) -> bool {
    line.trim().is_empty()
        || line.trim_start().starts_with('#')
        || line.trim_start().starts_with('.')
        || !line.contains(':')
        || line.contains('=')
}

/// Check if a target is missing dependencies
fn check_target_dependencies(
    target_info: &TargetInfo,
    lines: &[&str],
    start_idx: usize,
) -> Option<Diagnostic> {
    // Collect source files from recipe lines
    let source_files = collect_recipe_source_files(lines, start_idx + 1);

    // Find missing dependencies
    let missing_deps = find_missing_dependencies(&source_files, &target_info.declared_deps);

    if missing_deps.is_empty() {
        return None;
    }

    // Create diagnostic with fix suggestion
    Some(create_missing_deps_diagnostic(
        &target_info.name,
        &target_info.declared_deps,
        &missing_deps,
        start_idx,
    ))
}

/// Collect source files from recipe lines
fn collect_recipe_source_files(lines: &[&str], start_idx: usize) -> HashSet<String> {
    let mut source_files = HashSet::new();
    let mut i = start_idx;

    while i < lines.len() && lines[i].starts_with('\t') {
        extract_source_files(lines[i], &mut source_files);
        i += 1;
    }

    source_files
}

/// Find dependencies that are missing from declared dependencies
fn find_missing_dependencies(
    source_files: &HashSet<String>,
    declared_deps: &HashSet<String>,
) -> Vec<String> {
    let mut missing: Vec<String> = source_files
        .iter()
        .filter(|f| !declared_deps.contains(*f))
        .cloned()
        .collect();

    missing.sort();
    missing
}

/// Create a diagnostic for missing dependencies
fn create_missing_deps_diagnostic(
    target: &str,
    declared_deps: &HashSet<String>,
    missing_deps: &[String],
    line_idx: usize,
) -> Diagnostic {
    let span = Span::new(line_idx + 1, 1, line_idx + 1, target.len() + 1);

    // Create fix suggestion with all dependencies
    let mut all_deps: Vec<String> = declared_deps.iter().cloned().collect();
    all_deps.extend_from_slice(missing_deps);
    all_deps.sort();
    all_deps.dedup();

    let fix_replacement = format!("{}: {}", target, all_deps.join(" "));

    Diagnostic::new(
        "MAKE006",
        Severity::Warning,
        &format!(
            "Target '{}' may be missing dependencies: {}",
            target,
            missing_deps.join(", ")
        ),
        span,
    )
    .with_fix(Fix::new(&fix_replacement))
}

/// Find all .PHONY targets in the Makefile
fn find_phony_targets(source: &str) -> HashSet<String> {
    let mut phony = HashSet::new();

    for line in source.lines() {
        if line.trim_start().starts_with(".PHONY:") {
            if let Some(targets_str) = line.split(':').nth(1) {
                for target in targets_str.split_whitespace() {
                    phony.insert(target.to_string());
                }
            }
        }
    }

    phony
}

/// Extract source files (.c, .cpp, .h, .rs, etc.) from a recipe line
fn extract_source_files(recipe: &str, files: &mut HashSet<String>) {
    // Common source file extensions
    let extensions = [".c", ".cpp", ".cc", ".h", ".hpp", ".rs", ".go"];

    for word in recipe.split_whitespace() {
        for ext in &extensions {
            if word.ends_with(ext) {
                // Remove any flags or options attached to the filename
                let clean = word.trim_start_matches('-');
                files.insert(clean.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE006_detects_missing_dependencies_basic() {
        let makefile = r#"app:
	gcc main.c -o app"#;
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE006");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("dependencies"));
    }

    #[test]
    fn test_MAKE006_detects_missing_source_files() {
        let makefile = r#"app:
	gcc main.c utils.c -o app"#;
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("main.c"));
    }

    #[test]
    fn test_MAKE006_no_warning_with_dependencies() {
        let makefile = r#"app: main.c utils.c
	gcc main.c utils.c -o app"#;
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE006_no_warning_for_phony_targets() {
        let makefile = r#".PHONY: clean
clean:
	rm -f *.o"#;
        let result = check(makefile);

        // .PHONY targets don't need dependencies
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE006_provides_fix() {
        let makefile = r#"app:
	gcc main.c -o app"#;
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("main.c"));
    }

    #[test]
    fn test_MAKE006_detects_multiple_missing_dependencies() {
        let makefile = r#"app:
	gcc main.c utils.c helpers.c -o app"#;
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("main.c"));
        assert!(fix.replacement.contains("utils.c"));
        assert!(fix.replacement.contains("helpers.c"));
    }

    #[test]
    fn test_MAKE006_no_warning_for_variable_only() {
        let makefile = r#"CC = gcc
CFLAGS = -Wall"#;
        let result = check(makefile);

        // Variable assignments shouldn't trigger
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE006_partial_dependencies() {
        let makefile = r#"app: main.c
	gcc main.c utils.c -o app"#;
        let result = check(makefile);

        // Should warn about missing utils.c dependency
        assert_eq!(result.diagnostics.len(), 1);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("utils.c"));
    }
}
