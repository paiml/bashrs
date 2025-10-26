// SC2312: Consider invoking command explicitly with $(command)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    
    // This is a placeholder rule - actual implementation would require
    // more sophisticated parsing to detect implicit vs explicit command calls
    // For now, we'll keep it simple and not trigger false positives
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2312_explicit_ok() {
        let code = r#"result=$(echo "test")"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_implicit_ok() {
        let code = r#"result=$(cmd arg1 arg2)"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_comment() {
        let code = r#"# result=$(cmd)"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_nested_ok() {
        let code = r#"result=$(cat $(which bash))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_pipe_ok() {
        let code = r#"result=$(cat file | grep pattern)"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_multiple_ok() {
        let code = r#"
x=$(cmd1)
y=$(cmd2)
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_arithmetic_ok() {
        let code = r#"result=$((x + 1))"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2312_variable_ok() {
        let code = r#"result=$var"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
