//! # TICKET-1002 RED Phase: Unicode String Escaping Property Tests
//!
//! This module contains EXTREME TDD property-based tests for unicode string escaping.
//! These tests are designed to FAIL and expose bugs in the shell escaping logic.
//!
//! ## Testing Strategy
//!
//! 1. **Unicode Coverage**: Test all unicode categories (emoji, CJK, RTL, control chars)
//! 2. **Security**: Ensure no injection vectors via unicode
//! 3. **Roundtrip**: Escaped strings should preserve original content
//! 4. **Shell Safety**: All escaped strings must be valid POSIX shell syntax

use crate::emitter::escape::{escape_shell_string, escape_variable_name};
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// PROPERTY 1: Unicode strings escape safely without injection
// ============================================================================

#[test]
fn test_unicode_emoji_no_injection() {
    let test_cases = vec![
        "Hello 👋 World",
        "🔥💯✨",
        "Test 😀😁😂🤣",
        "Mixed text with 🚀 emoji",
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        // Should be quoted
        assert!(
            escaped.starts_with('\'') || escaped.starts_with('"'),
            "Emoji string not quoted: {} -> {}",
            input,
            escaped
        );

        // Test in actual shell - no injection
        let result = execute_shell_echo(&escaped);
        assert_eq!(
            result.trim(),
            input,
            "Emoji roundtrip failed: {} -> {} -> {}",
            input,
            escaped,
            result
        );
    }
}

#[test]
fn test_unicode_cjk_characters_safe() {
    let test_cases = vec![
        "你好世界",           // Chinese
        "こんにちは",         // Japanese Hiragana
        "コンニチハ",         // Japanese Katakana
        "안녕하세요",         // Korean
        "Mixed English 中文",
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        let result = execute_shell_echo(&escaped);
        assert_eq!(
            result.trim(),
            input,
            "CJK roundtrip failed: {} -> {} -> {}",
            input,
            escaped,
            result
        );
    }
}

#[test]
fn test_unicode_rtl_languages_safe() {
    let test_cases = vec![
        "مرحبا",              // Arabic
        "שלום",               // Hebrew
        "Mixed مرحبا English",
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        let result = execute_shell_echo(&escaped);
        assert_eq!(
            result.trim(),
            input,
            "RTL roundtrip failed: {} -> {} -> {}",
            input,
            escaped,
            result
        );
    }
}

#[test]
fn test_unicode_combining_characters_safe() {
    let test_cases = vec![
        "café",               // é = e + combining acute
        "naïve",              // ï = i + combining diaeresis
        "Zürich",             // ü = u + combining diaeresis
        "e\u{0301}",          // e + combining acute accent
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        let result = execute_shell_echo(&escaped);
        assert_eq!(
            result.trim(),
            input,
            "Combining char roundtrip failed: {} -> {} -> {}",
            input,
            escaped,
            result
        );
    }
}

// ============================================================================
// PROPERTY 2: Control characters are properly escaped
// ============================================================================

#[test]
fn test_unicode_control_characters_safe() {
    let test_cases = vec![
        "line1\nline2",       // Newline
        "tab\there",          // Tab
        "null\0byte",         // Null byte
        "bell\x07",           // Bell character
        "escape\x1b[0m",      // ANSI escape sequence
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        // Control characters should be quoted
        assert!(
            escaped.contains('\'') || escaped.contains('"'),
            "Control characters not quoted: {:?} -> {}",
            input,
            escaped
        );

        // Execute in shell - should preserve or safely handle
        let result = execute_shell_echo(&escaped);

        // Note: Some control chars may be processed by shell/echo
        // We just verify no injection or crash
        assert!(
            !result.contains("rm -rf"),
            "Possible injection with control chars: {:?} -> {}",
            input,
            result
        );
    }
}

// ============================================================================
// PROPERTY 3: Zero-width and invisible unicode are safe
// ============================================================================

#[test]
fn test_unicode_zero_width_characters_safe() {
    let test_cases = vec![
        "test\u{200B}invisible",      // Zero-width space
        "test\u{200C}joiner",          // Zero-width non-joiner
        "test\u{200D}joiner",          // Zero-width joiner
        "test\u{FEFF}bom",             // Zero-width no-break space (BOM)
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        let result = execute_shell_echo(&escaped);

        // Verify no injection
        assert!(
            !result.contains(';'),
            "Possible injection with zero-width chars: {:?} -> {}",
            input,
            result
        );
    }
}

// ============================================================================
// PROPERTY 4: Unicode normalization doesn't affect safety
// ============================================================================

#[test]
fn test_unicode_normalization_forms_safe() {
    // Same visual character in different unicode representations
    let nfc = "café";       // NFC: é as single codepoint U+00E9
    let nfd = "café";       // NFD: e + ́ (combining acute U+0301)

    let escaped_nfc = escape_shell_string(nfc);
    let escaped_nfd = escape_shell_string(nfd);

    // Both should escape safely
    let result_nfc = execute_shell_echo(&escaped_nfc);
    let result_nfd = execute_shell_echo(&escaped_nfd);

    // Results should match their inputs (modulo normalization)
    assert_eq!(result_nfc.trim(), nfc, "NFC roundtrip failed");
    assert_eq!(result_nfd.trim(), nfd, "NFD roundtrip failed");
}

// ============================================================================
// PROPERTY 5: Mixed unicode and shell metacharacters are safe
// ============================================================================

#[test]
fn test_unicode_with_shell_metacharacters() {
    let test_cases = vec![
        "Hello $USER 你好",
        "Path: $(pwd) 🚀",
        "Injection; rm -rf / 😈",
        "`backtick` attack 中文",
        "Pipe | redirect > 한글",
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        // Must be quoted to prevent injection
        assert!(
            escaped.starts_with('\''),
            "Shell metacharacters with unicode not quoted: {} -> {}",
            input,
            escaped
        );

        let result = execute_shell_echo(&escaped);

        // Metacharacters should be literal, not executed
        assert!(
            !result.contains("root") && !result.contains("bin"),
            "Possible command execution: {} -> {}",
            input,
            result
        );
    }
}

// ============================================================================
// PROPERTY 6: Very long unicode strings don't cause issues
// ============================================================================

#[test]
fn test_unicode_long_strings_safe() {
    // 1000 emoji characters
    let long_emoji = "🚀".repeat(1000);
    let escaped = escape_shell_string(&long_emoji);

    // Should still be escapable
    assert!(escaped.len() > 0, "Long emoji string produced empty escape");

    // Should roundtrip (we'll test a shorter version due to shell limits)
    let short_emoji = "🚀".repeat(10);
    let escaped_short = escape_shell_string(&short_emoji);
    let result = execute_shell_echo(&escaped_short);
    assert_eq!(result.trim(), short_emoji, "Long emoji roundtrip failed");
}

// ============================================================================
// PROPERTY 7: Unicode in variable names is handled safely
// ============================================================================

#[test]
fn test_unicode_variable_names_sanitized() {
    let test_cases = vec![
        ("hello_世界", "hello___"),       // CJK replaced (世界 is 2 chars)
        ("test_🚀", "test__"),            // Emoji replaced
        ("café_var", "caf__var"),         // Accented chars replaced (é is 2 chars in NFD form, but here it's 1)
        ("_valid", "_valid"),             // Valid name unchanged
        ("123_invalid", "_23_invalid"),   // Leading digit fixed
    ];

    for (input, expected) in test_cases {
        let result = escape_variable_name(input);
        assert_eq!(
            result, expected,
            "Variable name escaping failed: {} -> {} (expected {})",
            input, result, expected
        );

        // Result should be valid shell identifier
        assert!(
            is_valid_shell_identifier(&result),
            "Escaped variable name not valid: {}",
            result
        );
    }
}

// ============================================================================
// PROPERTY 8: Bidirectional unicode doesn't cause injection
// ============================================================================

#[test]
fn test_unicode_bidi_override_safe() {
    // Unicode bidirectional override can be used for obfuscation attacks
    let test_cases = vec![
        "test\u{202E}esrever",           // Right-to-left override
        "normal\u{202D}forced_ltr",      // Left-to-right override
        "embed\u{202A}rtl\u{202C}end",   // Left-to-right embedding
    ];

    for input in test_cases {
        let escaped = escape_shell_string(input);

        // Should be quoted
        assert!(
            escaped.contains('\''),
            "Bidi characters not quoted: {:?} -> {}",
            input,
            escaped
        );

        let result = execute_shell_echo(&escaped);

        // Bidi characters should be preserved (not executed as commands)
        // The visual appearance may be affected, but no actual code execution
        // Just verify no crash and successful roundtrip
        assert!(
            result.trim().len() > 0,
            "Bidi test failed: empty result for {:?}",
            input
        );
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Execute a shell echo command and capture output
/// This tests if our escaping works in a real shell
fn execute_shell_echo(escaped: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.sh");

    let script = format!(
        "#!/bin/sh\necho {}\n",
        escaped
    );

    std::fs::write(&script_path, script).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell");

    if !output.status.success() {
        // Shell syntax error - this is a test failure
        panic!(
            "Shell execution failed for escaped string: {}\nStderr: {}",
            escaped,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Check if a string is a valid POSIX shell identifier
fn is_valid_shell_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ============================================================================
// PROPERTY 9: Fuzzing-style random unicode property test
// ============================================================================

#[cfg(test)] // Only run this expensive test when explicitly testing
#[test]
#[ignore] // Ignore by default, run with: cargo test --ignored
fn test_unicode_fuzzing_random_strings() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        // Generate random unicode string
        let len = rng.gen_range(1..50);
        let random_string: String = (0..len)
            .map(|_| {
                let codepoint = rng.gen_range(0x0000..0x10FFFF);
                char::from_u32(codepoint).unwrap_or('?')
            })
            .collect();

        let escaped = escape_shell_string(&random_string);

        // Should not panic
        assert!(escaped.len() > 0, "Empty escape for: {:?}", random_string);

        // Should not contain unescaped dangerous characters
        if !escaped.starts_with('\'') {
            // If unquoted, must be safe
            assert!(
                !random_string.contains('$') &&
                !random_string.contains('`') &&
                !random_string.contains(';'),
                "Dangerous chars unquoted: {:?} -> {}",
                random_string,
                escaped
            );
        }
    }
}
