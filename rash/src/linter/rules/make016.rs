//! MAKE016: retired — Make does not quote prerequisites (GH-209)
//!
//! This rule used to flag `$(VAR)` in a target's prerequisite list and offer to
//! quote it. **That advice is not merely useless, it breaks the build.**
//!
//! GNU Make's prerequisite list is whitespace-separated and performs no
//! shell-style quote removal. A target line
//!
//! ```makefile
//! sakila: "$(SAKILA_DIR)"/sakila-data.sql
//! ```
//!
//! asks Make for a file whose name literally begins with a double-quote
//! character. The build then fails with "No rule to make target", and the
//! failure points at the prerequisite rather than at the linter that
//! introduced it.
//!
//! The rule's own doc-comment carried the broken form as its ✅ GOOD example,
//! so the premise was wrong from the first commit rather than drifting later.
//!
//! ## Why it is silenced rather than corrected
//!
//! There is no correct version of "quote this prerequisite". The conventional
//! Make answer to spaces in filenames is to avoid them, or to escape at the
//! variable level with `$(subst …)` — never quoting in the prerequisite list.
//! Recipes are shell and prerequisites are not, and only the former can be
//! linted this way. Unquoted variables in *recipe* lines are already MAKE003's
//! job, so repurposing this rule there would only duplicate it.
//!
//! ## Why the module still exists
//!
//! The ID stays registered so `--ignore MAKE016` in an existing config keeps
//! parsing, and so this explanation lives where the next person looks. `check`
//! is intentionally total: it returns no diagnostics for any input.
//!
//! Reported by a user whose CI ultimately **dropped the `bashrs lint Makefile`
//! step entirely** because no combination of applied fixes could get a correct
//! Makefile to exit 0. A rule that cannot be satisfied trains people to remove
//! the linter.

use crate::linter::LintResult;

/// Retired. Emits no diagnostics for any input — see the module docs.
pub fn check(_source: &str) -> LintResult {
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact shape from GH-209. The old rule fired here and its autofix
    /// would have produced `sakila: "$(SAKILA_DIR)"/sakila-data.sql`, which
    /// Make reads as a filename starting with `"`.
    #[test]
    fn parameterized_prerequisite_is_not_flagged() {
        let src = "sakila: $(SAKILA_DIR)/sakila-data.sql wait\n\t@echo ok\n";
        assert!(check(src).diagnostics.is_empty());
    }

    #[test]
    fn no_diagnostics_for_any_target_shape() {
        for src in [
            "app: $(FILES)\n",
            "app: $(wildcard src/*.c)\n",
            "%.o: %.c\n",
            ".PHONY: all\n",
            "all:\n",
            "",
        ] {
            assert!(
                check(src).diagnostics.is_empty(),
                "MAKE016 must stay silent, got diagnostics for {src:?}"
            );
        }
    }
}
