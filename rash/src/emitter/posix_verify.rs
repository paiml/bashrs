//! The emitted script must define every `rash_*` helper it calls.
//!
//! # Why this exists
//!
//! bashrs#266, found by dogfooding in `paiml/infra`. This program:
//!
//! ```ignore
//! fn main() {
//!     let out = exec("echo hi");
//!     println!("{}", out);
//! }
//! ```
//!
//! produced `out="$(rash_exec 'echo hi')"` while `rash_exec` was never written
//! into the script. `bashrs check` said "✓ compatible with Rash", `bashrs build`
//! said "Successfully transpiled" and exited 0, and the artifact died with
//! `rash_exec: not found`, exit 127.
//!
//! The same shape affected `mkdir`, `mv`, `chmod` and `sleep` in expression
//! position, and any call to a function that is neither stdlib nor user-defined.
//!
//! # Root cause, and why the fix is an invariant rather than five writers
//!
//! `stdlib::is_stdlib_function()` (the whitelist) and
//! `posix_runtime::write_selective_runtime()` (the emitter dispatch) are two
//! hand-maintained lists with nothing tying them together. GH-148 added seven
//! functions to the first and none to the second. Adding the five missing
//! writers would fix today's symptom and leave the class wide open for the next
//! stdlib addition.
//!
//! So instead: a transpiler's "Successfully transpiled" is a claim about the
//! ARTIFACT, and this makes it one. A script that calls a helper it does not
//! define is not a successful transpile, and saying so at build time turns a
//! 3am exit 127 into a compile error.

use crate::models::{Error, Result};
use std::collections::BTreeSet;

/// Prefix every generated runtime helper shares.
const HELPER_PREFIX: &str = "rash_";

/// Names defined in this script, i.e. lines of the form `rash_foo() {`.
fn defined_helpers(script: &str) -> BTreeSet<&str> {
    script
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let name = trimmed.strip_suffix("() {")?;
            if name.starts_with(HELPER_PREFIX) && is_identifier(name) {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

fn is_identifier(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Names INVOKED as commands, i.e. a `rash_*` token not immediately followed by
/// `(`. Excluding the `(` form is what keeps a definition line from counting as
/// a call to itself.
fn called_helpers(script: &str) -> BTreeSet<&str> {
    let mut found = BTreeSet::new();
    for (start, _) in script.match_indices(HELPER_PREFIX) {
        let rest = &script[start..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let name = &rest[..end];
        if name.len() <= HELPER_PREFIX.len() {
            continue;
        }
        // A definition (`rash_foo() {`) is not a call.
        if rest[end..].starts_with('(') {
            continue;
        }
        found.insert(name);
    }
    found
}

/// Fail the build when the script calls a helper it does not define.
///
/// This is deliberately a check on the emitted TEXT rather than on the IR: it is
/// the artifact that runs, and every route to a missing helper — a stdlib
/// function with no writer, a dispatch entry someone forgot, a typo'd user
/// function — converges here.
pub(crate) fn verify_calls_are_defined(script: &str) -> Result<()> {
    let defined = defined_helpers(script);
    let missing: Vec<&str> = called_helpers(script)
        .into_iter()
        .filter(|name| !defined.contains(name))
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    let names = missing.join(", ");
    Err(Error::Validation(format!(
        "internal: the generated script calls {names}, which it does not define. \
         The script would fail at runtime with `not found` (exit 127).\n\
         \n\
         This usually means a stdlib function was used in EXPRESSION position \
         (`let v = mkdir(\"d\")`) when it only has a STATEMENT lowering \
         (`mkdir(\"d\");`). Assigning the result of a void operation has no \
         meaning in shell — call it as a statement.\n\
         \n\
         If the name is not a stdlib function, it is a call to something that \
         does not exist; use exec() or capture() to run an external command.\n\
         \n\
         See bashrs#266."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_is_not_a_call_to_itself() {
        let script = "rash_println() {\n    printf '%s\\n' \"$1\"\n}\n";
        assert!(verify_calls_are_defined(script).is_ok());
    }

    #[test]
    fn defined_and_called_passes() {
        let script = "rash_println() {\n    :\n}\nmain() {\n    rash_println 'hi'\n}\n";
        assert!(verify_calls_are_defined(script).is_ok());
    }

    #[test]
    fn bashrs_266_undefined_helper_in_command_substitution_fails() {
        // The exact shape `let out = exec("echo hi")` produced.
        let script = "main() {\n    out=\"$(rash_exec 'echo hi')\"\n}\n";
        let err = verify_calls_are_defined(script).unwrap_err();
        assert!(format!("{err}").contains("rash_exec"), "{err}");
    }

    #[test]
    fn reports_every_missing_helper_not_just_the_first() {
        let script = "main() {\n    a=\"$(rash_exec 'x')\"\n    b=\"$(rash_mkdir 'd')\"\n}\n";
        let err = format!("{}", verify_calls_are_defined(script).unwrap_err());
        assert!(err.contains("rash_exec"), "{err}");
        assert!(err.contains("rash_mkdir"), "{err}");
    }

    #[test]
    fn indented_definition_still_counts() {
        let script =
            "    rash_exec() {\n        eval \"$1\"\n    }\nmain() {\n    rash_exec 'x'\n}\n";
        assert!(verify_calls_are_defined(script).is_ok());
    }

    #[test]
    fn bare_prefix_is_not_a_helper_name() {
        // `rash_` alone, and `rash_` inside a longer word boundary check.
        let script = "main() {\n    echo 'rash_'\n}\n";
        assert!(verify_calls_are_defined(script).is_ok());
    }
}
