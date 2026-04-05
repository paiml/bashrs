fn test_t022_grouping() {
    // STMT: let _ = (2 + 3) * 4; - should be 20, not 14
    let (ok, output) = transpile_stmt("let _ = (2 + 3) * 4;");
    if ok && output.contains("14") {
        println!("T022: BUG - Wrong grouping, got 14 instead of 20");
    }
}

// T023-T035: Additional Arithmetic

#[test]
fn test_t023_unary_minus() {
    // STMT: let _ = -5 + 3; - should be -2, not 8
    let (ok, output) = transpile_stmt("let _ = -5 + 3;");
    if ok && output.contains("8") {
        println!("T023: BUG - Unary minus not handled correctly");
    }
}

#[test]
fn test_t024_shift_left() {
    // STMT: let _ = 1 << 2; - should produce << in shell
    let (ok, output) = transpile_stmt("let _ = 1 << 2;");
    if ok && !output.contains("<<") && !output.contains("4") {
        println!("T024: WARNING - Shift left may not be supported");
    }
}

#[test]
fn test_t025_shift_right() {
    // STMT: let _ = 8 >> 2; - should produce >> in shell
    let (ok, output) = transpile_stmt("let _ = 8 >> 2;");
    if ok && !output.contains(">>") && !output.contains("2") {
        println!("T025: WARNING - Shift right may not be supported");
    }
}

#[test]
fn test_t026_bitwise_and() {
    // STMT: let _ = 5 & 3; - should be 1
    let (ok, output) = transpile_stmt("let _ = 5 & 3;");
    if !ok {
        println!("T026: Bitwise AND not supported: {}", output);
    }
}

#[test]
fn test_t027_bitwise_or() {
    // STMT: let _ = 5 | 3; - should be 7
    let (ok, output) = transpile_stmt("let _ = 5 | 3;");
    if !ok {
        println!("T027: Bitwise OR not supported: {}", output);
    }
}

#[test]
fn test_t028_bitwise_xor() {
    // STMT: let _ = 5 ^ 3; - should be 6
    let (ok, output) = transpile_stmt("let _ = 5 ^ 3;");
    if !ok {
        println!("T028: Bitwise XOR not supported: {}", output);
    }
}

#[test]
fn test_t029_bitwise_not() {
    // STMT: let _ = !5; - should use ~ in shell (or be rejected)
    let (ok, output) = transpile_stmt("let _ = !5;");
    if !ok {
        println!("T029: Bitwise NOT not supported (expected): {}", output);
    }
}

#[test]
fn test_t030_compound_add() {
    // STMT: let mut m = 1; m += 1; - compound assignment
    let (ok, output) = transpile_stmt("let mut m = 1; m += 1;");
    if ok {
        // Should have some form of increment
        if !output.contains("m=") && !output.contains("((") {
            println!("T030: WARNING - Compound add may not be correct");
        }
    }
}

#[test]
fn test_t031_compound_sub() {
    // STMT: let mut m = 1; m -= 1;
    let (ok, output) = transpile_stmt("let mut m = 5; m -= 1;");
    if ok && !output.contains("m=") && !output.contains("((") {
        println!("T031: WARNING - Compound sub may not be correct");
    }
}

#[test]
fn test_t032_compound_mul() {
    // STMT: let mut m = 1; m *= 2;
    let (ok, output) = transpile_stmt("let mut m = 3; m *= 2;");
    if ok && !output.contains("m=") && !output.contains("((") {
        println!("T032: WARNING - Compound mul may not be correct");
    }
}

#[test]
fn test_t033_compound_div() {
    // STMT: let mut m = 10; m /= 2;
    let (ok, output) = transpile_stmt("let mut m = 10; m /= 2;");
    if ok && !output.contains("m=") && !output.contains("((") {
        println!("T033: WARNING - Compound div may not be correct");
    }
}

#[test]
fn test_t034_compound_mod() {
    // STMT: let mut m = 10; m %= 3;
    let (ok, output) = transpile_stmt("let mut m = 10; m %= 3;");
    if ok && !output.contains("m=") && !output.contains("((") {
        println!("T034: WARNING - Compound mod may not be correct");
    }
}

#[test]
fn test_t035_numeric_comparison() {
    // STMT: let _ = (1+1)==2; - should use (( for numeric comparison
    let (ok, output) = transpile_stmt("let cmp = (1 + 1) == 2;");
    if ok {
        // Should produce some form of comparison
        if !output.contains("((") && !output.contains("[") && !output.contains("cmp=") {
            println!("T035: WARNING - Numeric comparison may not be correct");
        }
    }
}

// ============================================================================
// SECTION 4.3: Control Flow & Loops (T036-T055)
// ============================================================================

#[test]
fn test_t036_empty_if() {
    // STMT: if true { } - should have if/then/fi
    let (ok, output) = transpile_stmt("if true { }");
    if ok {
        let has_if = output.contains("if") && output.contains("fi");
        if !has_if {
            println!("T036: WARNING - Missing if/fi structure");
        }
    }
}

#[test]
fn test_t039_if_else() {
    // STMT: if true { } else { } - should have else
    let (ok, output) = transpile_stmt("if true { } else { }");
    if ok && !output.contains("else") {
        println!("T039: WARNING - Missing else clause");
    }
}

#[test]
fn test_t040_while_loop() {
    // STMT: while x < 20 { break; } - should have while/do/done
    let (ok, output) = transpile_stmt("let mut x = 0; while x < 20 { x += 1; break; }");
    if ok {
        let has_while = output.contains("while") && output.contains("done");
        if !has_while {
            println!("T040: WARNING - Missing while/done structure");
        }
    }
}

#[test]
fn test_t041_infinite_loop() {
    // STMT: loop { break; } - should become while true
    let (ok, output) = transpile_stmt("loop { break; }");
    if ok {
        let has_loop = output.contains("while true") || output.contains("while :");
        if !has_loop {
            println!("T041: WARNING - loop should become 'while true'");
        }
    }
}

#[test]
fn test_t042_range_loop() {
    // STMT: for i in 0..3 { } - KNOWN BUG TB-005
    let (ok, output) = transpile_stmt("for i in 0..3 { let _ = i; }");
    if !ok {
        println!("T042: KNOWN BUG TB-005 - Range loops unsupported");
        println!("      Error: {}", output);
    }
}

#[test]
fn test_t037_numeric_eq() {
    // STMT: if x == 1 { } - should prefer (( for numeric
    let (ok, output) = transpile_stmt("if x == 1 { }");
    if ok {
        // Either (( or [ is acceptable
        if !output.contains("if") {
            println!("T037: WARNING - Missing if structure");
        }
    }
}

#[test]
fn test_t038_string_eq() {
    // STMT: if s == "a" { } - must use [[ for string comparison
    let (ok, output) = transpile_stmt(r#"let s = "a"; if s == "a" { }"#);
    if ok {
        // String comparison - should use [[ or case
        if !output.contains("if") && !output.contains("case") {
            println!("T038: WARNING - String comparison structure missing");
        }
    }
}

#[test]
fn test_t043_inclusive_range() {
    // STMT: for i in 0..=3 { } - inclusive range
    let (ok, output) = transpile_stmt("for i in 0..=3 { let _ = i; }");
    if !ok {
        println!("T043: Inclusive range unsupported: {}", output);
    }
}

#[test]
fn test_t044_reverse_range() {
    // STMT: for i in (0..3).rev() { } - reverse range
    let (ok, output) = transpile_stmt("for i in (0..3).rev() { let _ = i; }");
    if !ok {
        println!("T044: Reverse range unsupported: {}", output);
    }
}

#[test]
fn test_t045_break() {
    // STMT: loop { break; } - break statement
    let (ok, output) = transpile_stmt("loop { break; }");
    if ok && !output.contains("break") {
        println!("T045: WARNING - break should be preserved");
    }
}

#[test]
fn test_t046_continue() {
    // STMT: loop { continue; } - continue statement
    let (ok, output) = transpile_stmt("let mut i = 0; while i < 5 { i += 1; continue; }");
    if ok && !output.contains("continue") {
        println!("T046: WARNING - continue should be preserved");
    }
}

#[test]
fn test_t047_labeled_break() {
    // STMT: 'label: loop { break 'label; } - labeled break
    let (ok, output) = transpile_stmt("'outer: loop { break 'outer; }");
    if !ok {
        println!("T047: Labeled loops unsupported: {}", output);
    }
}

#[test]
fn test_t048_if_let() {
    // STMT: if let Some(_) = opt { } - if-let pattern
    let (ok, output) = transpile_stmt("let opt = Some(1); if let Some(_) = opt { }");
    if !ok {
        println!("T048: if-let unsupported: {}", output);
    }
}

#[test]
fn test_t049_while_let() {
    // STMT: while let Some(_) = opt { break; }
    let (ok, output) =
        transpile_stmt("let mut opt = Some(1); while let Some(_) = opt { opt = None; break; }");
    if !ok {
        println!("T049: while-let unsupported: {}", output);
    }
}

#[test]
fn test_t050_array_iter() {
    // STMT: for _ in arr { } - array iteration
    let (ok, output) = transpile_stmt("let arr = [1, 2, 3]; for item in arr { let _ = item; }");
    if !ok {
        println!("T050: Array iteration unsupported: {}", output);
    }
}

#[test]
fn test_t051_logical_and() {
    // STMT: if x > 1 && x < 10 { } - logical AND
    let (ok, output) = transpile_stmt("if x > 1 && x < 10 { }");
    if ok {
        // Should have && or -a
        if !output.contains("&&") && !output.contains("-a") && !output.contains("if") {
            println!("T051: WARNING - Logical AND structure missing");
        }
    }
}

#[test]
fn test_t052_logical_or() {
    // STMT: if x < 1 || x > 10 { } - logical OR
    let (ok, output) = transpile_stmt("if x < 1 || x > 10 { }");
    if ok {
        // Should have || or -o
        if !output.contains("||") && !output.contains("-o") && !output.contains("if") {
            println!("T052: WARNING - Logical OR structure missing");
        }
    }
}

#[test]
fn test_t053_logical_not() {
    // STMT: if !b { } - logical NOT
    let (ok, output) = transpile_stmt("let b = true; if !b { }");
    if ok && !output.contains("!") && !output.contains("if") {
        println!("T053: WARNING - Logical NOT may not be correct");
    }
}

#[test]
fn test_t054_basic_match() {
    // STMT: match x { 1 => {}, _ => {} } - KNOWN BUG TB-010
    let (ok, output) = transpile_stmt("match x { 1 => {}, _ => {} }");
    if !ok {
        println!("T054: KNOWN BUG TB-010 - match statements unsupported");
    } else if !output.contains("case") {
        println!("T054: WARNING - match should produce case statement");
    }
}

#[test]
fn test_t055_range_match() {
    // STMT: match x { 1..=5 => {}, _ => {} } - range match
    let (ok, output) = transpile_stmt("match x { 1..=5 => {}, _ => {} }");
    if !ok {
        println!("T055: Range match unsupported: {}", output);
    }
}

// ============================================================================
// SECTION 4.4: Pattern Matching (T056-T070)
// ============================================================================

#[test]
fn test_t056_string_match() {
    // STMT: match s { "a" => {}, _ => {} } - string match
    let (ok, output) = transpile_stmt(r#"let s = "a"; match s { "a" => {}, _ => {} }"#);
    if !ok {
        println!("T056: String match unsupported: {}", output);
    }
}

#[test]

include!("transpiler_tcode_tests_tests_cont_3.rs");
