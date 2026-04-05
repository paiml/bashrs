//! Parser arithmetic inline tests — tokenizer, c-style for, and arithmetic expressions.
//!
//! Extracted from parser_core_tests.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::ast::*;
use crate::bash_parser::parser::*;
use crate::bash_parser::parser_arith::ArithToken;

    #[test]
    fn test_FOR_C_STYLE_019_body_with_assignment() {
        let (_, _, _, body_len) = parse_c_for("for ((i=0; i<3; i++)); do\nx=1\necho $x\ndone");
        assert!(body_len >= 2);
    }

    #[test]
    fn test_FOR_C_STYLE_020_complex_increment_expression() {
        let (_, _, incr, _) = parse_c_for("for ((i=0; i<100; i+=10)); do echo $i; done");
        // The increment should contain something representing i+=10
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FOR_C_STYLE_021_decrementing_loop() {
        let (init, cond, _, _) = parse_c_for("for ((i=10; i>0; i--)); do echo $i; done");
        assert!(init.contains("10"));
        assert!(cond.contains(">"));
    }

    #[test]
    fn test_FOR_C_STYLE_022_from_content_basic() {
        // This exercises parse_for_c_style_from_content via ArithmeticExpansion token
        // The lexer may combine ((...)) into a single token
        let input = "for ((x=1; x<5; x++)); do\necho $x\ndone";
        let (init, cond, incr, body_len) = parse_c_for(input);
        assert!(!init.is_empty());
        assert!(!cond.is_empty());
        assert!(!incr.is_empty());
        assert!(body_len >= 1);
    }

    #[test]
    fn test_FOR_C_STYLE_023_from_content_with_variables() {
        let input = "for ((n=0; n<max; n++)); do\necho $n\ndone";
        let (init, cond, incr, _) = parse_c_for(input);
        assert!(init.contains("n"));
        assert!(cond.contains("n") || cond.contains("max"));
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FOR_C_STYLE_024_single_body_command() {
        let (_, _, _, body_len) = parse_c_for("for ((i=0; i<1; i++)); do echo only; done");
        assert_eq!(body_len, 1);
    }

    #[test]
    fn test_FOR_C_STYLE_025_all_comparison_operators_together() {
        // Verify different operators parse correctly in separate loops
        let ops = vec![
            ("for ((i=0; i<10; i++)); do echo x; done", "<"),
            ("for ((i=0; i>0; i++)); do echo x; done", ">"),
            ("for ((i=0; i<=10; i++)); do echo x; done", "<="),
            ("for ((i=0; i>=0; i++)); do echo x; done", ">="),
            ("for ((i=0; i==0; i++)); do echo x; done", "=="),
            ("for ((i=0; i!=0; i++)); do echo x; done", "!="),
        ];
        for (input, expected_op) in ops {
            let (_, cond, _, _) = parse_c_for(input);
            assert!(
                cond.contains(expected_op),
                "Expected condition to contain '{expected_op}', got '{cond}' for input: {input}"
            );
        }
    }
}

// ============================================================================
// Coverage Tests - parse_arithmetic_expr (ARITH_EXPR_001-042)
// Comprehensive tests for all 15 precedence levels of arithmetic parsing
// ============================================================================
mod parse_arithmetic_expr_tests {
    #![allow(clippy::unwrap_used)]

    use crate::bash_parser::ast::*;
    use crate::bash_parser::parser::*;

    /// Helper: parse an arithmetic expression string into ArithExpr
    fn parse_arith(input: &str) -> ArithExpr {
        let mut parser = BashParser::new("echo x").unwrap();
        parser.parse_arithmetic_expr(input).unwrap()
    }

    /// Helper: parse expecting an error
    fn parse_arith_err(input: &str) -> ParseError {
        let mut parser = BashParser::new("echo x").unwrap();
        parser.parse_arithmetic_expr(input).unwrap_err()
    }

    // ── Primary (Level 15) ────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_001_number_literal() {
        assert_eq!(parse_arith("42"), ArithExpr::Number(42));
    }

    #[test]
    fn test_ARITH_EXPR_002_variable() {
        assert_eq!(parse_arith("x"), ArithExpr::Variable("x".to_string()));
    }

    #[test]
    fn test_ARITH_EXPR_003_parenthesized_expression() {
        assert_eq!(parse_arith("(7)"), ArithExpr::Number(7));
    }

    #[test]
    fn test_ARITH_EXPR_004_nested_parentheses() {
        assert_eq!(
            parse_arith("((1 + 2))"),
            ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            )
        );
    }

    // ── Unary (Level 14) ─────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_005_unary_minus() {
        // -5 becomes Sub(Number(0), Number(5))
        assert_eq!(
            parse_arith("-5"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(0)),
                Box::new(ArithExpr::Number(5)),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_006_unary_plus() {
        // +5 passes through to Number(5)
        assert_eq!(parse_arith("+5"), ArithExpr::Number(5));
    }

    #[test]
    fn test_ARITH_EXPR_007_bitwise_not() {
        // ~x becomes Sub(Number(-1), Variable("x"))
        assert_eq!(
            parse_arith("~x"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(-1)),
                Box::new(ArithExpr::Variable("x".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_008_logical_not() {
        // !x becomes Sub(Number(-1), Variable("x"))
        assert_eq!(
            parse_arith("!x"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(-1)),
                Box::new(ArithExpr::Variable("x".to_string())),
            )
        );
    }

    // ── Multiplicative (Level 13) ────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_009_multiply() {
        assert_eq!(
            parse_arith("a * b"),
            ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_010_divide() {
        assert_eq!(
            parse_arith("a / b"),
            ArithExpr::Div(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_011_modulo() {
        assert_eq!(
            parse_arith("a % b"),
            ArithExpr::Mod(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_012_chained_multiplicative() {
        // a * b / c  =>  Div(Mul(a, b), c)  (left-to-right associativity)
        assert_eq!(
            parse_arith("a * b / c"),
            ArithExpr::Div(
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Variable("a".to_string())),
                    Box::new(ArithExpr::Variable("b".to_string())),
                )),
                Box::new(ArithExpr::Variable("c".to_string())),
            )
        );
    }

    // ── Additive (Level 12) ──────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_013_addition() {
        assert_eq!(
            parse_arith("a + b"),
            ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_014_subtraction() {
        assert_eq!(
            parse_arith("a - b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_015_mixed_additive() {
        // a + b - c  =>  Sub(Add(a, b), c)
        assert_eq!(
            parse_arith("a + b - c"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Variable("a".to_string())),
                    Box::new(ArithExpr::Variable("b".to_string())),
                )),
                Box::new(ArithExpr::Variable("c".to_string())),
            )
        );
    }

    // ── Shift (Level 11) ─────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_016_shift_left() {
        // a << b  =>  Mul(a, b)
        assert_eq!(
            parse_arith("a << b"),
            ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_017_shift_right() {
        // a >> b  =>  Div(a, b)
        assert_eq!(
            parse_arith("a >> b"),
            ArithExpr::Div(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Comparison (Level 10) ────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_018_less_than() {
        // a < b  =>  Sub(a, b)
        assert_eq!(
            parse_arith("a < b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_019_less_equal() {
        assert_eq!(
            parse_arith("a <= b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_020_greater_than() {
        assert_eq!(
            parse_arith("a > b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_021_greater_equal() {
        assert_eq!(
            parse_arith("a >= b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Equality (Level 9) ───────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_022_equality() {
        // a == b  =>  Sub(a, b)
        assert_eq!(
            parse_arith("a == b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_023_not_equal() {
        // a != b  =>  Sub(a, b)
        assert_eq!(
            parse_arith("a != b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Bitwise AND (Level 8) ────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_024_bitwise_and() {
        // a & b  =>  Mul(a, b)
        assert_eq!(
            parse_arith("a & b"),
            ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Bitwise XOR (Level 7) ────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_025_bitwise_xor() {
        // a ^ b  =>  Sub(a, b)
        assert_eq!(
            parse_arith("a ^ b"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Bitwise OR (Level 6) ─────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_026_bitwise_or() {
        // a | b  =>  Add(a, b)
        assert_eq!(
            parse_arith("a | b"),
            ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Logical AND (Level 5) ────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_027_logical_and() {
        // a && b  =>  Mul(a, b)
        assert_eq!(
            parse_arith("a && b"),
            ArithExpr::Mul(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Logical OR (Level 4) ─────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_028_logical_or() {
        // a || b  =>  Add(a, b)
        assert_eq!(
            parse_arith("a || b"),
            ArithExpr::Add(
                Box::new(ArithExpr::Variable("a".to_string())),
                Box::new(ArithExpr::Variable("b".to_string())),
            )
        );
    }

    // ── Ternary (Level 3) ────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_029_ternary() {
        // a ? b : c  =>  Add(Mul(a, b), Mul(Sub(1, a), c))
        assert_eq!(
            parse_arith("a ? b : c"),
            ArithExpr::Add(
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Variable("a".to_string())),
                    Box::new(ArithExpr::Variable("b".to_string())),
                )),
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Sub(
                        Box::new(ArithExpr::Number(1)),
                        Box::new(ArithExpr::Variable("a".to_string())),
                    )),
                    Box::new(ArithExpr::Variable("c".to_string())),
                )),
            )
        );
    }

    // ── Comma (Level 1) ──────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_030_comma() {
        // a , b  =>  returns b (right value)
        assert_eq!(parse_arith("a , b"), ArithExpr::Variable("b".to_string()));
    }

    // ── Precedence / Complex ─────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_031_precedence_mul_over_add() {
        // 1 + 2 * 3  =>  Add(1, Mul(2, 3))
        assert_eq!(
            parse_arith("1 + 2 * 3"),
            ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Number(2)),
                    Box::new(ArithExpr::Number(3)),
                )),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_032_parentheses_override_precedence() {
        // (1 + 2) * 3  =>  Mul(Add(1, 2), 3)
        assert_eq!(
            parse_arith("(1 + 2) * 3"),
            ArithExpr::Mul(
                Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Number(1)),
                    Box::new(ArithExpr::Number(2)),
                )),
                Box::new(ArithExpr::Number(3)),
            )
        );
    }

