// SC2220: Wrong number of arguments for arithmetic operation (placeholder - requires AST)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: Complex rule requiring arithmetic expression parsing
    // Would need to:
    // 1. Parse $((expr)) and ((expr)) contents
    // 2. Identify operators and their arities
    // 3. Count operands for each operator
    // 4. Detect missing operands: (( x + )) or (( + ))
    // 5. Detect extra operands: (( x + y + ))
    // Implementation deferred pending AST-based arithmetic parser
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2220_placeholder() {
        let code = r#"(( x = y + z ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2220_missing_operand() {
        let code = r#"(( x = y + ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should flag
    }
    #[test]
    fn test_sc2220_missing_both() {
        let code = r#"(( x = + ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should flag
    }
    #[test]
    fn test_sc2220_trailing_operator() {
        let code = r#"(( result = a + b + ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should flag
    }
    #[test]
    fn test_sc2220_unary_ok() {
        let code = r#"(( x = -y ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Unary minus valid
    }
    #[test]
    fn test_sc2220_increment_ok() {
        let code = r#"(( ++count ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Prefix ++ valid
    }
    #[test]
    fn test_sc2220_complex_expr_ok() {
        let code = r#"(( x = (a + b) * (c - d) ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2220_ternary_ok() {
        let code = r#"(( result = x > 0 ? y : z ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Ternary operator valid
    }
    #[test]
    fn test_sc2220_comment() {
        let code = r#"# (( x = + ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2220_normal_math() {
        let code = r#"(( sum = a + b + c ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
