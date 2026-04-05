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
