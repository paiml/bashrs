#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Property-Based Falsification Tests (Popperian)
//!
//! These tests use proptest to attempt falsification of universal invariants
//! across thousands of random inputs. Each property is a ∀-quantified claim:
//! "for ALL inputs satisfying precondition P, invariant I holds."
//!
//! A single counterexample falsifies the property. 256 random cases
//! (proptest default) that survive constitute strong corroboration.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

use proptest::prelude::*;

// ============================================================================
// escape_shell_string: ∀ strings, output neutralizes shell metacharacters
// ============================================================================

proptest! {
    /// ∀ strings: escape_shell_string never produces empty output
    #[test]
    fn prop_escape_shell_never_empty(s in ".*") {
        let escaped = bashrs::emitter::escape::escape_shell_string(&s);
        prop_assert!(!escaped.is_empty(),
            "escape_shell_string must never return empty for input {:?}", s);
    }

    /// ∀ strings: escaped output is either safe-unquoted or starts with single quote
    #[test]
    fn prop_escape_shell_always_safe(s in ".*") {
        let escaped = bashrs::emitter::escape::escape_shell_string(&s);
        let is_safe_unquoted = escaped.chars().all(|c|
            c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-' | '+' | '=' | ':' | '@')
        );
        let is_quoted = escaped.starts_with('\'');
        prop_assert!(is_safe_unquoted || is_quoted,
            "escaped output must be safe-unquoted or single-quoted, got: {:?} for input {:?}",
            escaped, s);
    }

    /// ∀ strings with shell metacharacters: output is always quoted
    #[test]
    fn prop_escape_shell_metachars_quoted(s in ".*[$`|;()&].*") {
        let escaped = bashrs::emitter::escape::escape_shell_string(&s);
        prop_assert!(escaped.starts_with('\''),
            "metachar input {:?} must be single-quoted, got: {:?}", s, escaped);
    }

    /// ∀ strings: escape is deterministic
    #[test]
    fn prop_escape_shell_deterministic(s in ".*") {
        let a = bashrs::emitter::escape::escape_shell_string(&s);
        let b = bashrs::emitter::escape::escape_shell_string(&s);
        prop_assert_eq!(a, b, "escape must be deterministic for input {:?}", s);
    }
}

// ============================================================================
// escape_variable_name: ∀ strings, output is valid POSIX identifier
// ============================================================================

proptest! {
    /// ∀ non-empty strings: result is a valid POSIX shell identifier
    #[test]
    fn prop_varname_always_valid_identifier(s in ".+") {
        let result = bashrs::emitter::escape::escape_variable_name(&s);
        prop_assert!(!result.is_empty(), "varname must not be empty");

        let first = result.chars().next().unwrap();
        prop_assert!(first.is_ascii_alphabetic() || first == '_',
            "varname must start with letter or _, got {:?} for input {:?}", first, s);

        for c in result.chars().skip(1) {
            prop_assert!(c.is_ascii_alphanumeric() || c == '_',
                "varname char {:?} invalid in result {:?} for input {:?}", c, result, s);
        }
    }

    /// ∀ strings: varname escape is idempotent
    #[test]
    fn prop_varname_idempotent(s in ".+") {
        let once = bashrs::emitter::escape::escape_variable_name(&s);
        let twice = bashrs::emitter::escape::escape_variable_name(&once);
        prop_assert_eq!(once, twice,
            "varname escape must be idempotent for input {:?}", s);
    }
}

// ============================================================================
// escape_command_name: ∀ strings, output is safe for shell execution
// ============================================================================

proptest! {
    /// ∀ strings: command name escape never produces empty output
    #[test]
    fn prop_cmdname_never_empty(s in ".+") {
        let result = bashrs::emitter::escape::escape_command_name(&s);
        prop_assert!(!result.is_empty(),
            "command name must not be empty for input {:?}", s);
    }

    /// ∀ strings with semicolons: command name is always quoted
    #[test]
    fn prop_cmdname_semicolon_quoted(s in ".*[;|&].*") {
        let result = bashrs::emitter::escape::escape_command_name(&s);
        prop_assert!(result.starts_with('\''),
            "command with shell operator must be quoted, got {:?} for input {:?}", result, s);
    }
}

// ============================================================================
// Parser: ∀ inputs, parser never panics (robustness)
// ============================================================================

proptest! {
    /// ∀ arbitrary byte strings: parser does not panic
    /// (may return Err, but must not crash)
    #[test]
    fn prop_parser_never_panics(s in "[ -~]{0,100}") {
        // We only care that it doesn't panic — Ok or Err both fine
        let result = bashrs::bash_parser::BashParser::new(&s);
        match result {
            Ok(mut parser) => { let _ = parser.parse(); },
            Err(_) => { /* lexer error is fine */ },
        }
    }

    /// ∀ valid simple bash: parse is deterministic
    #[test]
    fn prop_parser_deterministic(
        cmd in "(echo|ls|cat|grep|mkdir|rm|cp|mv|chmod)",
        arg in "[a-zA-Z0-9_./-]{1,20}"
    ) {
        let input = format!("{} {}", cmd, arg);
        let r1 = bashrs::bash_parser::BashParser::new(&input)
            .and_then(|mut p| p.parse());
        let r2 = bashrs::bash_parser::BashParser::new(&input)
            .and_then(|mut p| p.parse());
        match (r1, r2) {
            (Ok(a), Ok(b)) => prop_assert_eq!(a, b, "parse must be deterministic"),
            (Err(_), Err(_)) => { /* both error — consistent */ },
            _ => prop_assert!(false, "parse returned different Ok/Err for same input"),
        }
    }
}

// ============================================================================
// Purification: ∀ valid bash, purify(purify(x)) == purify(x) (idempotence)
// ============================================================================

proptest! {
    /// ∀ simple valid bash: purification is idempotent
    #[test]
    fn prop_purify_idempotent(
        cmd in "(echo|ls|cat|mkdir|rm|cp|mv)",
        arg in "[a-zA-Z0-9_./]{1,15}"
    ) {
        let input = format!("{} {}", cmd, arg);
        if let Ok(once) = bashrs::repl::purifier::purify_bash(&input) {
            if let Ok(twice) = bashrs::repl::purifier::purify_bash(&once) {
                prop_assert_eq!(once, twice,
                    "purify must be idempotent for input {:?}", input);
            }
        }
    }

    /// ∀ simple valid bash: purified output never contains $RANDOM
    #[test]
    fn prop_purify_removes_random(
        prefix in "[a-zA-Z_]{1,5}",
    ) {
        let input = format!("{}=$RANDOM", prefix);
        if let Ok(output) = bashrs::repl::purifier::purify_bash(&input) {
            prop_assert!(!output.contains("$RANDOM"),
                "purified output must not contain $RANDOM, got {:?}", output);
        }
    }
}

// ============================================================================
// Transpiler: ∀ valid Rust inputs, transpile is deterministic
// ============================================================================

proptest! {
    /// ∀ simple let+println programs: transpile is deterministic
    #[test]
    fn prop_transpile_deterministic(
        var in "[a-z]{1,8}",
        val in "[0-9]{1,5}",
    ) {
        let input = format!("fn main() {{ let {} = {}; println!(\"{{}}\", {}); }}", var, val, var);
        let config = bashrs::Config::default();
        if let Ok(r1) = bashrs::transpile(&input, &config) {
            if let Ok(r2) = bashrs::transpile(&input, &config) {
                prop_assert_eq!(r1, r2,
                    "transpile must be deterministic");
            }
        }
    }
}
