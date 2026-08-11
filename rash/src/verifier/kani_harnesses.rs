#![cfg(kani)]
//! Kani verification harnesses for RASH
//!
//! These harnesses verify critical safety properties using bounded model
//! checking.
//!
//! ## GH-212: this file did not compile
//!
//! Every harness here was written without ever being built — the crate has not
//! compiled under `cfg(kani)` since these were added, and `make verify-kani`
//! swallowed the failure with `|| true`. Repairing it surfaced three separate
//! problems, only the first of which was a typo:
//!
//! 1. `kani::assert!` is not a macro Kani exports. Kani intercepts the standard
//!    `assert!`/`assert_eq!`, which is what the harnesses should have used.
//! 2. `escape_shell_value`, `is_valid_rust0` and `validate_rust0_ast` do not
//!    exist in this crate and there is no evidence they ever did.
//! 3. `String`/`&str` are not `kani::Arbitrary` and cannot be — see
//!    `crate::kani_bounded`.
//!
//! Two harnesses were also removed rather than repaired, because compiling them
//! would have produced proofs of nothing:
//!
//! - `verify_array_bounds_safety` asserted `true` on one branch and, on the
//!   other, that `format!("if [ {} -lt {} ]; then", ..)` contains `"-lt"`. That
//!   is a property of the format string literal. No code under test was reached.
//! - `verify_parser_soundness` called the real parser on an arbitrary string and
//!   checked it against the two functions that do not exist. Even with those
//!   supplied, a full Rust parser over symbolic input is not tractable under
//!   BMC; a passing version of it would only mean the bound was too small.
//!
//! A harness that cannot fail is worse than a missing one: it consumes the
//! verification budget and reports success.

use crate::emitter::escape::{escape_shell_string, escape_variable_name};
use crate::kani_bounded::{any_bounded_identifier, any_bounded_string};

/// Verify shell string escaping prevents injection.
///
/// This is the one original harness that exercised production code, and it is
/// kept as-is apart from the bounded input: `escape_shell_string` is the real
/// function every emitted script depends on.
#[kani::proof]
#[kani::unwind(3)]
fn verify_escape_safety() {
    let input = any_bounded_string::<2>();

    let escaped = escape_shell_string(&input);

    // Property 1: the result is always single-quoted
    assert!(escaped.starts_with('\'') && escaped.ends_with('\''));

    // Property 2: no unescaped metacharacter can survive
    assert!(!contains_unescaped_metachar(&escaped));

    // Property 3: content is preserved modulo escaping (round-trip)
    let unescaped = unescape_shell_string(&escaped);
    assert!(unescaped == input);
}

/// Verify that variable-name escaping accepts every valid identifier.
///
/// The original asserted that `format!("\"${{{}}}\"", name)` starts with `"` and
/// contains `"${"` — true of the literal regardless of `name`, so it held even
/// if `escape_variable_name` were the identity function. This calls the real
/// `escape_variable_name` instead, which is what the emitter uses.
#[kani::proof]
#[kani::unwind(3)]
fn verify_variable_expansion_safety() {
    let var_name = any_bounded_identifier::<2>();

    let escaped = escape_variable_name(&var_name);

    // A valid identifier must survive escaping unchanged — if it does not, the
    // emitter is rewriting names the user chose.
    assert!(escaped == var_name);

    // And the expansion built from it is quoted, so word-splitting cannot occur.
    let expansion = format!("\"${{{escaped}}}\"");
    assert!(expansion.starts_with('"') && expansion.ends_with('"'));
    assert!(!contains_unescaped_metachar(&escaped));
}

/// Verify no injection is possible through an escaped argument.
///
/// The original called `escape_shell_value` (which does not exist) and checked
/// the result with `can_inject_command`, a simplified re-implementation local to
/// this file — so it verified a toy model against a toy oracle. This runs the
/// real escaper and asserts the property directly on its output.
#[kani::proof]
#[kani::unwind(3)]
fn verify_injection_safety() {
    let user_input = any_bounded_string::<2>();

    let escaped = escape_shell_string(&user_input);
    let context = format!("echo {escaped}");

    // Everything after `echo ` is a single quoted word, so no separator in the
    // user's input can escape into command position.
    assert!(!contains_unescaped_metachar(&context));
}

/// Helper: does the string contain a shell metacharacter outside quotes?
fn contains_unescaped_metachar(s: &str) -> bool {
    let mut in_quotes = false;
    let mut escaped = false;

    for ch in s.chars() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '\'' => in_quotes = !in_quotes,
            ';' | '&' | '|' | '`' | '$' | '(' | ')' | '<' | '>' | '\n' => {
                if !in_quotes {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// Helper: invert `escape_shell_string`, for the round-trip property.
fn unescape_shell_string(s: &str) -> String {
    if s.starts_with('\'') && s.ends_with('\'') {
        let inner = &s[1..s.len() - 1];
        inner.replace("'\"'\"'", "'")
    } else {
        s.to_string()
    }
}
