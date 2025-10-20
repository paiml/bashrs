//! MAKE018: Parallel-unsafe targets (race conditions)
//!
//! **Rule**: Detect targets that modify shared state without synchronization
//!
//! **Why this matters**:
//! When Make runs with -j (parallel jobs), targets can run concurrently.
//! Targets that write to the same file or directory can have race conditions.
//! This can cause corrupted builds or intermittent failures.
//!
//! **Auto-fix**: Suggest .NOTPARALLEL or order-only prerequisites
//!
//! ## Examples
//!
//! ❌ **BAD** (parallel-unsafe - multiple targets write to same directory):
//! ```makefile
//! install-bin:
//! \tcp app /usr/bin/app
//!
//! install-lib:
//! \tcp lib.so /usr/bin/lib.so
//! ```
//!
//! ✅ **GOOD** (with .NOTPARALLEL):
//! ```makefile
//! .NOTPARALLEL:
//!
//! install-bin:
//! \tcp app /usr/bin/app
//!
//! install-lib:
//! \tcp lib.so /usr/bin/lib.so
//! ```
//!
//! ✅ **GOOD** (with explicit dependencies):
//! ```makefile
//! install-bin:
//! \tcp app /usr/bin/app
//!
//! install-lib: install-bin
//! \tcp lib.so /usr/bin/lib.so
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use std::collections::{HashMap, HashSet};

/// Patterns that indicate shared state modification
const SHARED_STATE_PATTERNS: &[&str] = &[
    "/usr/bin",
    "/usr/local/bin",
    "/usr/lib",
    "/usr/local/lib",
    "/etc",
    "/var",
    "/tmp",
];

/// Check for parallel-unsafe targets
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Empty Makefile - no parallel safety issues
    if source.trim().is_empty() {
        return result;
    }

    // If .NOTPARALLEL is present, no parallel safety issues
    if has_notparallel(source) {
        return result;
    }

    // Collect all targets and their shared state writes
    let targets = collect_targets_with_shared_state(source);

    // Find targets that write to overlapping shared state
    let conflicts = find_parallel_conflicts(&targets);

    // Create diagnostic if conflicts found
    if !conflicts.is_empty() {
        let span = Span::new(1, 1, 1, 1);
        let fix_replacement = format!(".NOTPARALLEL:\n\n{}", source);

        let diag = Diagnostic::new(
            "MAKE018",
            Severity::Warning,
            &format!(
                "Multiple targets write to shared state - parallel-unsafe (conflicts: {})",
                conflicts.join(", ")
            ),
            span,
        )
        .with_fix(Fix::new(&fix_replacement));

        result.add(diag);
    }

    result
}

/// Check if Makefile has .NOTPARALLEL
fn has_notparallel(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == ".NOTPARALLEL:" || trimmed == ".NOTPARALLEL" {
            return true;
        }
    }
    false
}

/// Target with its shared state writes
struct TargetState {
    name: String,
    shared_paths: Vec<String>,
}

/// Collect targets that write to shared state
fn collect_targets_with_shared_state(source: &str) -> Vec<TargetState> {
    let mut targets = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this is a target line
        if line.contains(':')
            && !line.starts_with(char::is_whitespace)
            && !line.trim_start().starts_with('#')
        {
            if let Some(colon_pos) = line.find(':') {
                let target_name = line[..colon_pos].trim().to_string();

                // Skip special targets
                if target_name.starts_with('.') {
                    i += 1;
                    continue;
                }

                // Collect shared state writes in this target's recipes
                let mut shared_paths = Vec::new();
                let mut j = i + 1;

                while j < lines.len() {
                    let recipe_line = lines[j];

                    // Recipe lines start with tab
                    if !recipe_line.starts_with('\t') {
                        break;
                    }

                    // Check for shared state patterns
                    for pattern in SHARED_STATE_PATTERNS {
                        if recipe_line.contains(pattern) {
                            shared_paths.push(pattern.to_string());
                        }
                    }

                    j += 1;
                }

                // Only add target if it writes to shared state
                if !shared_paths.is_empty() {
                    targets.push(TargetState {
                        name: target_name,
                        shared_paths,
                    });
                }

                i = j;
                continue;
            }
        }

        i += 1;
    }

    targets
}

/// Find targets with overlapping shared state writes
fn find_parallel_conflicts(targets: &[TargetState]) -> Vec<String> {
    let mut conflicts = HashSet::new();

    // Build map of shared path -> targets that write to it
    let mut path_to_targets: HashMap<String, Vec<String>> = HashMap::new();

    for target in targets {
        for path in &target.shared_paths {
            path_to_targets
                .entry(path.clone())
                .or_insert_with(Vec::new)
                .push(target.name.clone());
        }
    }

    // Find paths written by multiple targets
    for (path, writing_targets) in path_to_targets {
        if writing_targets.len() > 1 {
            conflicts.insert(path);
        }
    }

    conflicts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE018_detects_parallel_unsafe_targets() {
        let makefile =
            "install-bin:\n\tcp app /usr/bin/app\n\ninstall-lib:\n\tcp lib.so /usr/bin/lib.so";
        let result = check(makefile);

        // Both targets write to /usr/bin - parallel unsafe
        assert!(result.diagnostics.len() >= 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE018");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("parallel"));
    }

    #[test]
    fn test_MAKE018_no_warning_with_notparallel() {
        let makefile = ".NOTPARALLEL:\n\ninstall-bin:\n\tcp app /usr/bin/app\n\ninstall-lib:\n\tcp lib.so /usr/bin/lib.so";
        let result = check(makefile);

        // .NOTPARALLEL prevents parallel execution - safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE018_provides_fix() {
        let makefile =
            "install-bin:\n\tcp app /usr/bin/app\n\ninstall-lib:\n\tcp lib.so /usr/bin/lib.so";
        let result = check(makefile);

        assert!(result.diagnostics.len() > 0);
        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains(".NOTPARALLEL:"));
    }

    #[test]
    fn test_MAKE018_detects_shared_directory_writes() {
        let makefile = "setup-etc:\n\tcp config.conf /etc/app/config.conf\n\nsetup-var:\n\tcp data.db /var/app/data.db\n\nsetup-etc2:\n\tcp default.conf /etc/app/default.conf";
        let result = check(makefile);

        // setup-etc and setup-etc2 both write to /etc - parallel unsafe
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_MAKE018_no_warning_for_different_directories() {
        let makefile = "install-bin:\n\tcp app /usr/bin/app\n\ninstall-config:\n\tcp config.yaml /home/user/.config/app.yaml";
        let result = check(makefile);

        // Different directories - safe to run in parallel
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE018_no_warning_for_single_target() {
        let makefile = "install:\n\tcp app /usr/bin/app\n\tcp lib.so /usr/lib/lib.so";
        let result = check(makefile);

        // Single target - no parallel risk
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE018_detects_tmp_writes() {
        let makefile = "build-a:\n\techo building > /tmp/build.log\n\nbuild-b:\n\techo building > /tmp/build.log";
        let result = check(makefile);

        // Both write to same temp file - parallel unsafe
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_MAKE018_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
