#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_level_suppression() {
        let source = "# bashrs disable-file=SC2086,DET002\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 2));
        assert!(manager.is_suppressed("DET002", 2));
        assert!(!manager.is_suppressed("SC2046", 2));
    }

    #[test]
    fn test_parse_next_line_suppression() {
        let source = "# bashrs disable-next-line=SC2086\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 2));
        assert!(!manager.is_suppressed("SC2086", 1));
        assert!(!manager.is_suppressed("SC2086", 3));
    }

    #[test]
    fn test_parse_inline_suppression() {
        let source = "echo $var  # bashrs disable-line=SC2086\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 1));
        assert!(!manager.is_suppressed("SC2086", 2));
    }

    #[test]
    fn test_multiple_rules() {
        let source = "# bashrs disable-next-line=SC2086,SC2046,DET002\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 2));
        assert!(manager.is_suppressed("SC2046", 2));
        assert!(manager.is_suppressed("DET002", 2));
    }

    #[test]
    fn test_no_suppression() {
        let source = "echo $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(!manager.is_suppressed("SC2086", 1));
    }

    // =====================================================
    // Shorthand syntax tests (Issue #70)
    // =====================================================

    #[test]
    fn test_shorthand_disable_syntax() {
        // Issue #70: Support shorthand # bashrs disable=RULE
        let source = "# bashrs disable=SEC010\nmkdir -p \"${BASELINE_DIR}\"\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SEC010", 2));
        assert!(!manager.is_suppressed("SEC010", 1));
        assert!(!manager.is_suppressed("SEC010", 3));
    }

    #[test]
    fn test_shorthand_disable_multiple_rules() {
        let source = "# bashrs disable=SEC010,DET002\nmkdir -p \"${BASELINE_DIR}\"\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SEC010", 2));
        assert!(manager.is_suppressed("DET002", 2));
    }

    #[test]
    fn test_shorthand_does_not_match_specific_patterns() {
        // Ensure shorthand doesn't interfere with specific patterns
        let source = "# bashrs disable-file=SEC010\nline2\nline3\n";
        let manager = SuppressionManager::from_source(source);

        // File-level should suppress all lines
        assert!(manager.is_suppressed("SEC010", 1));
        assert!(manager.is_suppressed("SEC010", 2));
        assert!(manager.is_suppressed("SEC010", 3));
    }

    // =====================================================
    // Shellcheck syntax compatibility tests (Issue #58)
    // =====================================================

    #[test]
    fn test_shellcheck_disable_next_line() {
        // Shellcheck disable directives AFTER code apply to the next line only
        // Issue #130: Directives at top of file are file-level, so we need code first
        let source = "echo start\n# shellcheck disable=SC2086\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        assert!(manager.is_suppressed("SC2086", 3)); // next line after directive
        assert!(!manager.is_suppressed("SC2086", 1)); // line before directive
        assert!(!manager.is_suppressed("SC2086", 2)); // directive line itself
    }

    #[test]
    fn test_shellcheck_disable_multiple_rules() {
        // Issue #130: Directives at top of file (before any code) are file-level
        let source = "# shellcheck disable=SC2086,SC2046,DET002\necho $var\n";
        let manager = SuppressionManager::from_source(source);

        // File-level suppression applies to all lines
        assert!(manager.is_suppressed("SC2086", 2));
        assert!(manager.is_suppressed("SC2046", 2));
        assert!(manager.is_suppressed("DET002", 2));
    }

    #[test]
    fn test_shellcheck_disable_does_not_affect_other_lines() {
        // After code, shellcheck directive only affects next line
        let source = "echo start\n# shellcheck disable=SC2086\necho $var\necho $another\n";
        let manager = SuppressionManager::from_source(source);

        // Only line 3 should be suppressed (next line after directive)
        assert!(manager.is_suppressed("SC2086", 3));
        assert!(!manager.is_suppressed("SC2086", 4)); // Line after that is NOT suppressed
    }

    #[test]
    fn test_mixed_bashrs_and_shellcheck_syntax() {
        let source = r#"
# shellcheck disable=SC2086
echo $var
# bashrs disable-next-line=SC2046
echo $(cat file)
"#;
        let manager = SuppressionManager::from_source(source);

        // SC2086 suppressed on line 3 (after shellcheck directive on line 2)
        assert!(manager.is_suppressed("SC2086", 3));
        // SC2046 suppressed on line 5 (after bashrs directive on line 4)
        assert!(manager.is_suppressed("SC2046", 5));
    }

    // Issue #130: Shellcheck file-level suppression tests

    #[test]
    fn test_shellcheck_file_level_suppression_at_top() {
        // Issue #130: Shellcheck directives at top of file (before any code)
        // should apply to the entire file
        let source = r#"#!/bin/bash
# shellcheck disable=SC2086
# shellcheck disable=SEC010
set -euo pipefail
echo $var
mkdir -p "$PATH/dir"
"#;
        let manager = SuppressionManager::from_source(source);

        // Directives at top should apply to all lines
        assert!(manager.is_suppressed("SC2086", 5)); // echo $var
        assert!(manager.is_suppressed("SC2086", 6)); // mkdir
        assert!(manager.is_suppressed("SEC010", 6)); // mkdir
    }

    #[test]
    fn test_shellcheck_mid_file_is_next_line_only() {
        // Shellcheck directive in the middle of code should only apply to next line
        let source = r#"#!/bin/bash
echo "hello"
# shellcheck disable=SC2086
echo $var
echo $another
"#;
        let manager = SuppressionManager::from_source(source);

        // After code, shellcheck directive is next-line only
        assert!(manager.is_suppressed("SC2086", 4)); // next line
        assert!(!manager.is_suppressed("SC2086", 5)); // NOT the line after
    }

    #[test]
    fn test_shellcheck_file_level_with_shebang_and_comments() {
        // Issue #130: Real-world pattern from raid-targets.sh
        let source = r#"#!/usr/bin/env bash
# raid-targets.sh - Symlink Cargo target directories to NVMe RAID
#
# shellcheck disable=SC2145  # $* is intentional for log concatenation
# shellcheck disable=SEC010  # Paths validated via validate_path()
# shellcheck disable=IDEM003 # ln -sfn IS idempotent

set -euo pipefail

log_info() { echo -e "$*"; }
mkdir -p "$RAID_PATH"
ln -sfn "$target" "$link"
"#;
        let manager = SuppressionManager::from_source(source);

        // All directives should be file-level since they appear before any code
        assert!(manager.is_suppressed("SC2145", 10)); // log_info
        assert!(manager.is_suppressed("SEC010", 11)); // mkdir
        assert!(manager.is_suppressed("IDEM003", 12)); // ln
    }
}
