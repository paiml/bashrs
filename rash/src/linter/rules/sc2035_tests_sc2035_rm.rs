#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2035_rm_glob() {
        let code = r#"rm *.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2035");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("./*"));
    }

    #[test]
    fn test_sc2035_cat_glob() {
        let code = r#"cat *.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_grep_glob() {
        let code = r#"grep pattern *.sh"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_safe_dotslash_ok() {
        let code = r#"rm ./*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_absolute_path_ok() {
        let code = r#"rm /tmp/*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_variable_path_ok() {
        let code = r#"rm "$dir"/*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_multiple_globs() {
        let code = r#"rm *.txt *.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2035_mv_glob() {
        let code = r#"mv *.bak /backup/"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_chmod_glob() {
        let code = r#"chmod 644 *.conf"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_no_glob_ok() {
        let code = r#"rm file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Issue #96: find -name pattern tests =====
    // Patterns after find -name/-iname/-path are for find, not shell expansion

    #[test]
    fn test_FP_096_find_name_not_flagged() {
        let code = r#"find . -name '*.json'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag find -name patterns"
        );
    }

    #[test]
    fn test_FP_096_find_iname_not_flagged() {
        let code = r#"find /tmp -iname '*.TXT'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag find -iname patterns"
        );
    }

    #[test]
    fn test_FP_096_find_path_not_flagged() {
        let code = r#"find . -path '*.log'"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag find -path patterns"
        );
    }

    #[test]
    fn test_FP_096_find_name_unquoted_still_flagged() {
        // Unquoted glob after find -name IS dangerous (shell expands before find sees it)
        let code = r#"find . -name *.json"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2035 SHOULD flag unquoted find -name patterns"
        );
    }

    #[test]
    fn test_FP_096_find_name_double_quoted_not_flagged() {
        let code = r#"find . -name "*.json""#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag double-quoted find -name patterns"
        );
    }

    // ===== Issue #104: grep pattern tests =====
    // Patterns after grep are regex patterns, not shell globs

    #[test]
    fn test_FP_104_grep_quoted_pattern_not_flagged() {
        let code = r#"grep -r '*.c' ."#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag quoted grep patterns"
        );
    }

    #[test]
    fn test_FP_104_grep_e_pattern_not_flagged() {
        let code = r#"grep -e '*.log' files.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag grep -e patterns"
        );
    }

    #[test]
    fn test_FP_104_grep_E_pattern_not_flagged() {
        let code = r#"grep -E '.*\.txt' ."#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag grep -E patterns"
        );
    }

    #[test]
    fn test_FP_104_egrep_pattern_not_flagged() {
        let code = r#"egrep '*.json' file"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag egrep patterns"
        );
    }

    #[test]
    fn test_FP_104_grep_unquoted_still_flagged() {
        // Unquoted glob after grep IS a shell glob
        let code = r#"grep pattern *.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2035 SHOULD flag unquoted globs as file args to grep"
        );
    }

    // ===== FP018: Stderr redirect handling =====
    // When stderr is redirected to /dev/null, user is handling "no match" case

    #[test]
    fn test_FP018_glob_with_stderr_redirect_not_flagged() {
        // User redirects stderr - they're handling "no files match" scenario
        let code = r#"ls *.txt 2>/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs when stderr is redirected to /dev/null"
        );
    }

    #[test]
    fn test_FP018_glob_with_stderr_redirect_space_not_flagged() {
        let code = r#"ls *.txt 2> /dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs with spaced stderr redirect"
        );
    }

    #[test]
    fn test_FP018_glob_with_stderr_ampersand_redirect_not_flagged() {
        // &>/dev/null redirects both stdout and stderr
        let code = r#"ls *.txt &>/dev/null"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs with &>/dev/null"
        );
    }

    #[test]
    fn test_FP018_glob_with_stderr_redirect_or_not_flagged() {
        // cmd || true also handles errors
        let code = r#"ls *.txt 2>/dev/null || true"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2035 must NOT flag globs with stderr redirect and || true"
        );
    }

    #[test]
    fn test_FP018_glob_without_redirect_still_flagged() {
        // Without redirect, should still flag
        let code = r#"ls *.txt"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "SC2035 SHOULD flag globs without error handling"
        );
    }
}
