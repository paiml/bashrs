
use super::super::*;
use crate::bash_parser::parser_arith::ArithToken;
#[test]
fn test_parse_remove_longest_prefix() {
    let input = "echo ${x##pattern}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Command { args, .. } => {
            assert!(matches!(&args[0], BashExpr::RemoveLongestPrefix { .. }));
        }
        _ => panic!("Expected Command with RemoveLongestPrefix"),
    }
}

#[test]
fn test_parse_remove_suffix() {
    let input = "echo ${x%pattern}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Command { args, .. } => {
            assert!(matches!(&args[0], BashExpr::RemoveSuffix { .. }));
        }
        _ => panic!("Expected Command with RemoveSuffix"),
    }
}

#[test]
fn test_parse_remove_longest_suffix() {
    let input = "echo ${x%%pattern}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Command { args, .. } => {
            assert!(matches!(&args[0], BashExpr::RemoveLongestSuffix { .. }));
        }
        _ => panic!("Expected Command with RemoveLongestSuffix"),
    }
}

// ============================================================================
// Coverage Tests - Arithmetic Operations
// ============================================================================

#[test]
fn test_parse_arithmetic_subtraction() {
    let input = "x=$((a - b))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Assignment { value, .. } => match value {
            BashExpr::Arithmetic(arith) => {
                assert!(matches!(arith.as_ref(), ArithExpr::Sub(_, _)));
            }
            _ => panic!("Expected Arithmetic expression"),
        },
        _ => panic!("Expected Assignment"),
    }
}

#[test]
fn test_parse_arithmetic_division() {
    let input = "x=$((a / b))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Assignment { value, .. } => match value {
            BashExpr::Arithmetic(arith) => {
                assert!(matches!(arith.as_ref(), ArithExpr::Div(_, _)));
            }
            _ => panic!("Expected Arithmetic expression"),
        },
        _ => panic!("Expected Assignment"),
    }
}

#[test]
fn test_parse_arithmetic_modulo() {
    let input = "x=$((a % b))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Assignment { value, .. } => match value {
            BashExpr::Arithmetic(arith) => {
                assert!(matches!(arith.as_ref(), ArithExpr::Mod(_, _)));
            }
            _ => panic!("Expected Arithmetic expression"),
        },
        _ => panic!("Expected Assignment"),
    }
}

#[test]
fn test_parse_arithmetic_negative() {
    let input = "x=$((-5))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(matches!(&ast.statements[0], BashStmt::Assignment { .. }));
}

#[test]
fn test_parse_arithmetic_parentheses() {
    let input = "x=$(((1 + 2) * 3))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(matches!(&ast.statements[0], BashStmt::Assignment { .. }));
}

// ============================================================================
// Coverage Tests - Arithmetic Tokenizer & Parser (ARITH_COV_001-040)
// ============================================================================

/// Helper: parse arithmetic expression from `x=$((expr))` pattern
fn parse_arith(expr: &str) -> ArithExpr {
    let input = format!("x=$(({expr}))");
    let mut parser = BashParser::new(&input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Assignment { value, .. } => match value {
            BashExpr::Arithmetic(arith) => arith.as_ref().clone(),
            other => panic!("Expected Arithmetic, got {other:?}"),
        },
        other => panic!("Expected Assignment, got {other:?}"),
    }
}

// --- Tokenizer: comparison operators ---

#[test]
fn test_ARITH_COV_001_less_than() {
    let arith = parse_arith("a < b");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_002_less_equal() {
    let arith = parse_arith("a <= b");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_003_greater_than() {
    let arith = parse_arith("a > b");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_004_greater_equal() {
    let arith = parse_arith("a >= b");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_005_shift_left() {
    let arith = parse_arith("a << b");
    // Shift left represented as Mul
    assert!(matches!(arith, ArithExpr::Mul(_, _)));
}

#[test]
fn test_ARITH_COV_006_shift_right() {
    let arith = parse_arith("a >> b");
    // Shift right represented as Div
    assert!(matches!(arith, ArithExpr::Div(_, _)));
}

// --- Tokenizer: equality operators ---

#[test]
fn test_ARITH_COV_007_equal() {
    let arith = parse_arith("a == b");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_008_not_equal() {
    let arith = parse_arith("a != b");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

// --- Tokenizer: logical operators ---

#[test]
fn test_ARITH_COV_009_logical_and() {
    let arith = parse_arith("a && b");
    // Logical AND represented as Mul
    assert!(matches!(arith, ArithExpr::Mul(_, _)));
}

#[test]
fn test_ARITH_COV_010_logical_or() {
    let arith = parse_arith("a || b");
    // Logical OR represented as Add
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_011_logical_not() {
    let arith = parse_arith("!a");
    // Logical NOT represented as Sub(-1, operand)
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

// --- Tokenizer: bitwise operators ---

#[test]
fn test_ARITH_COV_012_bit_and() {
    let arith = parse_arith("a & b");
    // Bitwise AND represented as Mul
    assert!(matches!(arith, ArithExpr::Mul(_, _)));
}

#[test]
fn test_ARITH_COV_013_bit_or() {
    let arith = parse_arith("a | b");
    // Bitwise OR represented as Add
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_014_bit_xor() {
    let arith = parse_arith("a ^ b");
    // Bitwise XOR represented as Sub
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_015_bit_not() {
    let arith = parse_arith("~a");
    // Bitwise NOT represented as Sub(-1, operand)
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

// --- Tokenizer: ternary operator ---

#[test]
fn test_ARITH_COV_016_ternary() {
    let arith = parse_arith("a ? 1 : 0");
    // Ternary represented as Add(Mul(cond, then), Mul(Sub(1, cond), else))
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

// --- Tokenizer: comma operator ---

#[test]
fn test_ARITH_COV_017_comma() {
    let arith = parse_arith("1, 2");
    // Comma returns the right value
    assert!(matches!(arith, ArithExpr::Number(2)));
}

// --- Tokenizer: assignment ---

#[test]
fn test_ARITH_COV_018_assign() {
    // Single = in arithmetic is assignment; parsed through assign level
    // The tokenizer produces Assign token, but parse_assign just calls parse_ternary
    // So this just tests that '=' alone doesn't crash
    let input = "x=$((y = 5))";
    let mut parser = BashParser::new(input).unwrap();
    let _ast = parser.parse();
    // May or may not parse successfully depending on grammar, just ensure no panic
}

// --- Tokenizer: hex and octal numbers ---

#[test]
fn test_ARITH_COV_019_hex_number() {
    let arith = parse_arith("0xff");
    assert!(matches!(arith, ArithExpr::Number(255)));
}

#[test]
fn test_ARITH_COV_020_hex_uppercase() {
    let arith = parse_arith("0XFF");
    assert!(matches!(arith, ArithExpr::Number(255)));
}

#[test]
fn test_ARITH_COV_021_octal_number() {
    let arith = parse_arith("077");
    assert!(matches!(arith, ArithExpr::Number(63)));
}

#[test]
fn test_ARITH_COV_022_zero_literal() {
    let arith = parse_arith("0");
    assert!(matches!(arith, ArithExpr::Number(0)));
}

// --- Tokenizer: dollar variable ---

#[test]
fn test_ARITH_COV_023_dollar_variable() {
    let arith = parse_arith("$x + 1");
    match arith {
        ArithExpr::Add(left, right) => {
            assert!(matches!(left.as_ref(), ArithExpr::Variable(v) if v == "x"));
            assert!(matches!(right.as_ref(), ArithExpr::Number(1)));
        }
        other => panic!("Expected Add, got {other:?}"),
    }
}

// --- Tokenizer: whitespace handling ---

#[test]
fn test_ARITH_COV_024_whitespace_tab_newline() {
    let arith = parse_arith("\t1\n+\t2\n");
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

// --- Parser: unary plus ---

#[test]
fn test_ARITH_COV_025_unary_plus() {
    let arith = parse_arith("+5");
    assert!(matches!(arith, ArithExpr::Number(5)));
}

// --- Parser: complex expressions hitting multiple levels ---

#[test]
fn test_ARITH_COV_026_comparison_chain() {
    let arith = parse_arith("a < b < c");
    // Two comparisons chained
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_027_equality_chain() {
    let arith = parse_arith("a == b != c");
    assert!(matches!(arith, ArithExpr::Sub(_, _)));
}

#[test]
fn test_ARITH_COV_028_nested_ternary() {
    let arith = parse_arith("a ? b ? 1 : 2 : 3");
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_029_all_bitwise_combined() {
    // a | b ^ c & d — exercises bitwise OR, XOR, AND levels
    let arith = parse_arith("a | b ^ c & d");
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_030_logical_combined() {
    // a || b && c — exercises logical OR and AND levels
    let arith = parse_arith("a || b && c");
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_031_shift_combined() {
    // 1 << 2 >> 3 — exercises both shift directions
    let arith = parse_arith("1 << 2 >> 3");
    assert!(matches!(arith, ArithExpr::Div(_, _)));
}

#[test]
fn test_ARITH_COV_032_hex_arithmetic() {
    let arith = parse_arith("0xa + 0xb");
    match arith {
        ArithExpr::Add(left, right) => {
            assert!(matches!(left.as_ref(), ArithExpr::Number(10)));
            assert!(matches!(right.as_ref(), ArithExpr::Number(11)));
        }
        other => panic!("Expected Add, got {other:?}"),
    }
}

#[test]
fn test_ARITH_COV_033_octal_arithmetic() {
    let arith = parse_arith("010 + 010");
    match arith {
        ArithExpr::Add(left, right) => {
            assert!(matches!(left.as_ref(), ArithExpr::Number(8)));
            assert!(matches!(right.as_ref(), ArithExpr::Number(8)));
        }
        other => panic!("Expected Add, got {other:?}"),
    }
}

#[test]
fn test_ARITH_COV_034_underscore_variable() {
    let arith = parse_arith("_foo + _bar");
    match arith {
        ArithExpr::Add(left, right) => {
            assert!(matches!(left.as_ref(), ArithExpr::Variable(v) if v == "_foo"));
            assert!(matches!(right.as_ref(), ArithExpr::Variable(v) if v == "_bar"));
        }
        other => panic!("Expected Add, got {other:?}"),
    }
}

#[test]
fn test_ARITH_COV_035_complex_precedence() {
    // 1 + 2 * 3 — mul before add
    let arith = parse_arith("1 + 2 * 3");
    match &arith {
        ArithExpr::Add(left, right) => {
            assert!(matches!(left.as_ref(), ArithExpr::Number(1)));
            assert!(matches!(right.as_ref(), ArithExpr::Mul(_, _)));
        }
        other => panic!("Expected Add(1, Mul(2,3)), got {other:?}"),
    }
}

#[test]
fn test_ARITH_COV_036_unary_minus_in_expression() {
    let arith = parse_arith("-a + b");
    match arith {
        ArithExpr::Add(left, _right) => {
            // Unary minus is Sub(0, a)
            assert!(matches!(left.as_ref(), ArithExpr::Sub(_, _)));
        }
        other => panic!("Expected Add(Sub(0,a), b), got {other:?}"),
    }
}

#[test]
fn test_ARITH_COV_037_parenthesized_comma() {
    // Comma in parenthesized expression
    let arith = parse_arith("(1, 2) + 3");
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_038_nested_parentheses() {
    let arith = parse_arith("((a + b))");
    assert!(matches!(arith, ArithExpr::Add(_, _)));
}

#[test]
fn test_ARITH_COV_039_multi_digit_number() {
    let arith = parse_arith("12345");
    assert!(matches!(arith, ArithExpr::Number(12345)));
}

#[test]
fn test_ARITH_COV_040_all_multiplicative_ops() {
    // 10 * 3 / 2 % 5 — exercises all three multiplicative operators
    let arith = parse_arith("10 * 3 / 2 % 5");
    assert!(matches!(arith, ArithExpr::Mod(_, _)));
}

// ============================================================================
// Coverage Tests - Command Substitution
// ============================================================================

#[test]
fn test_parse_command_substitution() {
    let input = "x=$(pwd)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Assignment { value, .. } => {
            assert!(matches!(value, BashExpr::CommandSubst(_)));
        }
        _ => panic!("Expected Assignment with CommandSubst"),
    }
}

// ============================================================================
// Coverage Tests - Comments
// ============================================================================
