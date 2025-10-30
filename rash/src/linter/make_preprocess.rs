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

use once_cell::sync::Lazy;
use regex::Regex;

/// Match recipe lines (lines starting with tab after target declaration)
static RECIPE_LINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\t").unwrap());

/// Match target declarations (word followed by colon)
static TARGET_DECL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+\s*:").unwrap());

/// Preprocess Makefile source for linting
///
/// This function:
/// 1. Identifies recipe lines (lines that start with tab)
/// 2. Converts $$ → $ in recipes (Make's shell variable escape)
/// 3. Preserves Make variables $(...)
/// 4. Leaves non-recipe lines unchanged
pub fn preprocess_for_linting(source: &str) -> String {
    if source.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut in_recipe = false;

    for line in source.lines() {
        // Check if we're entering or leaving a recipe context
        if TARGET_DECL.is_match(line) {
            // Target declaration - recipes start on next line
            in_recipe = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Empty lines or non-tabbed lines end recipes
        if (line.is_empty() || (!line.starts_with('\t') && !line.starts_with(' ')))
            && !TARGET_DECL.is_match(line)
        {
            in_recipe = false;
        }

        // Process recipe lines
        if in_recipe && RECIPE_LINE.is_match(line) {
            let processed = preprocess_recipe_line(line);
            result.push_str(&processed);
        } else {
            result.push_str(line);
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

        // Verify non-recipe lines unchanged
        assert!(result.contains("PROJECT := myproject"));
        assert!(result.contains("rm -rf *.o"));
    }

    #[test]
    fn test_preprocess_no_recipes() {
        let makefile = "PROJECT := myproject\n";
        let result = preprocess_for_linting(makefile);
        assert_eq!(result, "PROJECT := myproject\n");
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
}
