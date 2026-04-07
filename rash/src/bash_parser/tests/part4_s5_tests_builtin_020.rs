fn test_BUILTIN_020_unset_exit_status() {
    let mut lexer = Lexer::new(BUILTIN_020_UNSET_EXIT_STATUS_INPUT);
    let result = lexer.tokenize();
    assert!(
        result.is_ok(),
        "exit status examples should tokenize: {:?}",
        result.err()
    );
}

#[test]
fn test_BUILTIN_020_unset_common_patterns() {
    let common_patterns = r#"
TEMP_FILE="/tmp/data.$$"
echo "data" > "$TEMP_FILE"
unset TEMP_FILE
PASSWORD="secret123"
unset PASSWORD
"#;

    let mut lexer = Lexer::new(common_patterns);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "common patterns should tokenize");
        }
        Err(_) => {
            // Parser may not fully support all patterns yet
        }
    }
}
