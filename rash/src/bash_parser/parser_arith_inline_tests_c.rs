//! Parser arithmetic inline tests — tokenizer, c-style for, and arithmetic expressions.
//!
//! Extracted from parser_core_tests.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::ast::*;
use crate::bash_parser::parser::*;
use crate::bash_parser::parser_arith::ArithToken;

#[test]
fn test_TEST_COMPOUND_001_triple_and() {
    let input = r#"[[ "$a" == "1" && "$b" == "2" && "$c" == "3" ]]"#;
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse();
    assert!(
        ast.is_ok(),
        "triple && inside [[ ]] should parse: {:?}",
        ast.err()
    );
}

#[test]
fn test_DOGFOOD_029_edge_cases() {
    let input = r#"result=$(echo "$(basename "$(dirname "$(pwd)")")")
echo "Grandparent: $result"
echo "${UNDEFINED:-default value with spaces}"
outer="hello"
echo "${outer:-${inner:-deep_default}}"
x=10
(( x += 5 ))
echo "x=$x"
for i in 1 2 3; do
for j in a b c; do
    if [[ "$j" == "b" ]]; then
        continue
    fi
    if [[ "$i" == "2" && "$j" == "c" ]]; then
        break 2
    fi
    echo "$i-$j"
done
done
n=5
until [[ $n -le 0 ]]; do
echo "Countdown: $n"
n=$((n - 1))
done
if (( age >= 18 && age < 65 )); then
echo "Working age"
fi
if [ -f /etc/passwd -a -r /etc/passwd ]; then
echo "readable"
fi
"#;
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse();
    assert!(
        ast.is_ok(),
        "dogfood_29 edge cases should parse: {:?}",
        ast.err()
    );
}
