//! Makefile recipe preprocessing for bash linting
//!
//! When linting Makefile recipes, we need to preprocess them to handle
//! Make-specific syntax that differs from plain bash:
//!
//! 1. $$ → $ (Make's escape for shell variables)
//! 2. $(VAR) stays as-is (Make variables, not shell command substitution)
//!
//! This module provides preprocessing to convert Make recipe syntax to
//! equivalent bash syntax for linting purposes.

use regex::Regex;

/// Match recipe lines (lines starting with tab after target declaration)
static RECIPE_LINE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^\t").unwrap());

/// Match target declarations (word followed by colon)
static TARGET_DECL: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+\s*:").unwrap());

/// Preprocess Makefile source for linting
///
/// Only three shell rules (SC2133, SC2168, SC2299) ever see this output
/// (`lint_makefile` in `rules/mod_std.rs`); MAKE001..MAKE020 read the
/// original source. So this function's only job is to make sure those
/// three shell rules never read Make grammar as shell:
///
/// 1. Recipe lines (lines that start with a tab) are converted: `$$` → `$`
///    (Make's shell-variable escape) with `$(...)` left as-is, exactly as
///    before (see `preprocess_recipe_line`).
/// 2. Every other line — target declarations, variable assignments,
///    directives (`include`, `.PHONY:`, ...), comments, blank lines, and
///    `define`/`endef` bodies — is replaced with an empty line. None of
///    that is shell, so no shell rule may read it; blanking (rather than
///    dropping) keeps the line count and every remaining line's number
///    identical to the input, which is what diagnostics report against.
pub fn preprocess_for_linting(source: &str) -> String {
    if source.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut in_recipe = false;

    for line in source.lines() {
        // Check if we're entering or leaving a recipe context
        if TARGET_DECL.is_match(line) {
            // Target declaration - recipes start on next line.
            // Not shell: blank it out, but keep the line (and count).
            in_recipe = true;
            result.push('\n');
            continue;
        }

        // Empty lines or non-tabbed lines end recipes
        if line.is_empty() || (!line.starts_with('\t') && !line.starts_with(' ')) {
            in_recipe = false;
        }

        // Process recipe lines; everything else (variable assignments,
        // directives, comments, blank lines, define/endef bodies) is
        // Make syntax, never shell, so it becomes an empty line.
        if in_recipe && RECIPE_LINE.is_match(line) {
            let processed = preprocess_recipe_line(line);
            result.push_str(&processed);
        }

        result.push('\n');
    }

    result
}

/// Preprocess a single recipe line
///
/// Converts $$ → $ for shell variable access
/// Preserves $(...) as Make variables
fn preprocess_recipe_line(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&next) = chars.peek() {
                if next == '$' {
                    // $$ → $ (shell variable in Make recipe)
                    chars.next(); // consume second $
                    result.push('$');
                } else if next == '(' {
                    // $(...) - keep as-is (Make variable or command sub)
                    result.push(c);
                } else {
                    // $X - keep as-is
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_recipe_double_dollar() {
        let line = "\t@CORES=$$(nproc)";
        let result = preprocess_recipe_line(line);
        assert_eq!(result, "\t@CORES=$(nproc)");
    }

    #[test]
    fn test_preprocess_recipe_arithmetic() {
        let line = "\t@THREADS=$$((CORES > 2 ? CORES - 2 : 1))";
        let result = preprocess_recipe_line(line);
        assert_eq!(result, "\t@THREADS=$((CORES > 2 ? CORES - 2 : 1))");
    }

    #[test]
    fn test_preprocess_preserves_make_variables() {
        let line = "\techo $(PROJECT_NAME)";
        let result = preprocess_recipe_line(line);
        assert_eq!(result, "\techo $(PROJECT_NAME)");
    }

    #[test]
    fn test_preprocess_mixed_syntax() {
        let line = "\t@echo $$USER logged into $(HOSTNAME)";
        let result = preprocess_recipe_line(line);
        assert_eq!(result, "\t@echo $USER logged into $(HOSTNAME)");
    }

    #[test]
    fn test_preprocess_full_makefile() {
        let makefile = r#"
PROJECT := myproject

build:
	@CORES=$$(nproc)
	@THREADS=$$((CORES > 2 ? CORES - 2 : 1))
	echo "Building with $$THREADS threads"

clean:
	rm -rf *.o
"#;
        let result = preprocess_for_linting(makefile);

        // Verify $$ converted to $ in recipes
        assert!(result.contains("@CORES=$(nproc)"));
        assert!(result.contains("@THREADS=$((CORES > 2 ? CORES - 2 : 1))"));
        assert!(result.contains("echo \"Building with $THREADS threads\""));

        // Verify non-recipe lines (target decls, variable assignments) are
        // blanked, not left as Make syntax for a shell rule to misread.
        assert!(!result.contains("PROJECT := myproject"));
        assert!(!result.contains("build:"));
        assert!(!result.contains("clean:"));
        // But `rm -rf *.o` is a recipe line under `clean:`, so it survives.
        assert!(result.contains("rm -rf *.o"));
        // Line count must be preserved.
        assert_eq!(result.lines().count(), makefile.lines().count());
    }

    #[test]
    fn test_preprocess_no_recipes() {
        let makefile = "PROJECT := myproject\n";
        let result = preprocess_for_linting(makefile);
        // A bare variable assignment is not shell; it is blanked, not left
        // for a shell rule to read as Make syntax.
        assert_eq!(result, "\n");
    }

    #[test]
    fn test_preprocess_empty() {
        let result = preprocess_for_linting("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_makefile_arithmetic_with_dollar_dollar() {
        let makefile = r#"
target:
	@CORES=$$(nproc) && THREADS=$$((CORES > 2 ? CORES - 2 : 1))
"#;
        let result = preprocess_for_linting(makefile);

        // After preprocessing, should NOT trigger SC2133
        // because $$ is converted to $
        assert!(result.contains("@CORES=$(nproc) && THREADS=$((CORES > 2 ? CORES - 2 : 1))"));
    }

    #[test]
    fn test_makefile_swap_arithmetic() {
        let makefile = r#"
check-resources:
	@SWAP_USED=$$(free | grep Swap | awk '{print $$3}')
	@SWAP_TOTAL=$$(free | grep Swap | awk '{print $$2}')
	@if [ $$((SWAP_USED * 100 / SWAP_TOTAL)) -gt 80 ]; then echo "High swap"; fi
"#;
        let result = preprocess_for_linting(makefile);

        // All $$ should be converted to $
        assert!(result.contains("@SWAP_USED=$(free | grep Swap | awk '{print $3}')"));
        assert!(result.contains("@SWAP_TOTAL=$(free | grep Swap | awk '{print $2}')"));
        assert!(result.contains("@if [ $((SWAP_USED * 100 / SWAP_TOTAL)) -gt 80 ]"));
    }

    // -----------------------------------------------------------------
    // PMAT-248 phase 5 (F-LCX-007, issue #255): non-recipe lines must
    // never reach a shell rule as text, only as an empty line at the
    // same position.
    // -----------------------------------------------------------------

    #[test]
    fn test_pmat248_gh255_target_line_with_comment_is_blanked() {
        let makefile = "dev-setup: ## Set up local dev environment\n\t@echo hi\n";
        let result = preprocess_for_linting(makefile);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        // The target line (with its trailing `## ... local ...` comment)
        // must be blank so SC2168 never sees the English word "local".
        assert_eq!(lines[0], "");
        assert!(!lines[0].contains("local"));
        // The recipe line is preserved untouched (no $$ to convert here).
        assert_eq!(lines[1], "\t@echo hi");
    }

    #[test]
    fn test_pmat248_variable_assignment_with_trailing_comment_is_blanked() {
        let makefile = "x := 1 # local note\n";
        let result = preprocess_for_linting(makefile);
        assert_eq!(result, "\n");
    }

    #[test]
    fn test_pmat248_recipe_local_declaration_is_preserved() {
        // Recipe lines keep today's treatment untouched; SC2168 firing on
        // this is covered end-to-end by lexer_context_tests.rs.
        let makefile = "build:\n\tlocal x=1\n";
        let result = preprocess_for_linting(makefile);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "");
        assert_eq!(lines[1], "\tlocal x=1");
    }

    #[test]
    fn test_pmat248_recipe_dollar_dollar_home_still_converted() {
        let makefile = "build:\n\techo $$HOME\n";
        let result = preprocess_for_linting(makefile);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "");
        assert_eq!(lines[1], "\techo $HOME");
    }

    #[test]
    fn test_pmat248_line_count_preserved_multi_target() {
        let makefile = r#"# top comment
PROJECT := myproject

build: ## build target
	@CORES=$$(nproc)
	echo done

clean:
	rm -rf *.o
"#;
        let result = preprocess_for_linting(makefile);
        assert_eq!(result.lines().count(), makefile.lines().count());
    }

    #[test]
    fn test_pmat248_phony_and_include_directives_are_blanked() {
        let makefile = ".PHONY: build\ninclude common.mk\nbuild:\n\t@echo hi\n";
        let result = preprocess_for_linting(makefile);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], ""); // .PHONY: build
        assert_eq!(lines[1], ""); // include common.mk
        assert_eq!(lines[2], ""); // build:
        assert_eq!(lines[3], "\t@echo hi");
    }
}
