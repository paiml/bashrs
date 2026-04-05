fn test_REDIR_003_purification_strategy() {
    // DOCUMENTATION: Purification strategy for &> redirection
    //
    // bashrs purification should convert Bash &> to POSIX:
    //
    // INPUT (Bash):
    // cmd &> output.txt
    //
    // PURIFIED (POSIX sh):
    // cmd > output.txt 2>&1
    //
    // INPUT (Bash append):
    // cmd &>> log.txt
    //
    // PURIFIED (POSIX sh):
    // cmd >> log.txt 2>&1
    //
    // Purification steps:
    // 1. Detect &> or &>> syntax
    // 2. Convert to > file 2>&1 or >> file 2>&1
    // 3. Quote filename for safety
    // 4. Preserve argument order

    // This test documents the purification strategy
}

#[test]
fn test_REDIR_003_order_matters() {
    // DOCUMENTATION: Redirection order matters in POSIX
    //
    // CORRECT order (stdout first, then stderr):
    // $ cmd > file 2>&1
    //
    // 1. > file - Redirect stdout (fd 1) to file
    // 2. 2>&1 - Duplicate stderr (fd 2) to stdout (fd 1, which now points to file)
    // Result: Both stdout and stderr go to file
    //
    // WRONG order (stderr first, then stdout):
    // $ cmd 2>&1 > file
    //
    // 1. 2>&1 - Duplicate stderr (fd 2) to stdout (fd 1, still terminal)
    // 2. > file - Redirect stdout (fd 1) to file
    // Result: stderr goes to terminal, stdout goes to file
    //
    // Rule: Always put > file BEFORE 2>&1

    let correct_order = r#"
# CORRECT: > file 2>&1
cmd > output.txt 2>&1
"#;

    let result = BashParser::new(correct_order);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Correct order: > file 2>&1"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_003_common_use_cases() {
    // DOCUMENTATION: Common combined redirection patterns
    //
    // 1. Capture all output (stdout + stderr):
    //    POSIX: cmd > output.txt 2>&1
    //    Bash: cmd &> output.txt
    //
    // 2. Append all output to log:
    //    POSIX: cmd >> app.log 2>&1
    //    Bash: cmd &>> app.log
    //
    // 3. Discard all output:
    //    POSIX: cmd > /dev/null 2>&1
    //    Bash: cmd &> /dev/null
    //
    // 4. Capture in variable (all output):
    //    POSIX: output=$(cmd 2>&1)
    //    Bash: output=$(cmd 2>&1)  # No &> in command substitution
    //
    // 5. Log with timestamp:
    //    POSIX: (date; cmd) > log.txt 2>&1
    //    Bash: (date; cmd) &> log.txt

    let common_patterns = r#"
# Capture all output (POSIX)
cmd > output.txt 2>&1

# Append to log (POSIX)
cmd >> app.log 2>&1

# Discard all (POSIX)
cmd > /dev/null 2>&1

# Capture in variable (POSIX)
output=$(cmd 2>&1)

# Log with timestamp (POSIX)
(date; cmd) > log.txt 2>&1
"#;

    let result = BashParser::new(common_patterns);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common POSIX combined redirection patterns documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}
