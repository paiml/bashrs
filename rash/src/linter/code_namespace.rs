//! `SCxxxx` belongs to ShellCheck. `BRS####` is ours.
//!
//! bashrs grew ~225 checks filed under `SCxxxx` numbers. Measuring them against
//! ShellCheck's own output (see `tests/data/shellcheck-registry.tsv`) found 26
//! numbers where the two tools attach the same code to unrelated checks — for
//! example `SC2311`, which ShellCheck uses for "Bash implicitly disabled set -e
//! for this function invocation" and bashrs used for "Use single quotes for
//! literal strings".
//!
//! That is not cosmetic. `# shellcheck disable=SC2311` in a shared codebase
//! silences the wrong diagnostic in one of the two tools; a baseline keyed on
//! the number conflates two checks; and the wiki page a user lands on describes
//! something else.
//!
//! `SEC###`, `DET###` and `IDEM###` never had this problem because they own
//! their namespace. `BRS####` extends that model to the SC-squatting checks.
//!
//! # What this module does NOT do
//!
//! It renames codes. It does not change what any rule detects, and it never
//! drops a diagnostic: `apply()` is a bijection on the diagnostics it touches.
//! A check that fired before fires now, under a code that means what it says.

use crate::linter::LintResult;

/// `(legacy ShellCheck-squatting code, replacement BRS code)`.
///
/// Every entry is a MEASURED collision: bashrs' emitted message and
/// ShellCheck's own message for the same number describe different checks.
/// The ShellCheck side of each pair is quoted in the comment and recorded in
/// `tests/data/shellcheck-registry.tsv`.
pub const MIGRATIONS: &[(&str, &str)] = &[
    // SC1009: "The mentioned syntax error was in this brace group."
    ("SC1009", "BRS0001"),
    // SC2036: "If you wanted to assign the output of the pipeline, use a=$(b | c)."
    ("SC2036", "BRS0002"),
    // SC2066: "Since you double quoted this, it will not word split, and the loop will only run once."
    ("SC2066", "BRS0003"),
    // SC2069: "To redirect stdout+stderr, 2>&1 must be last (or use '{ cmd > file; } 2>&1' to clarify)."
    ("SC2069", "BRS0004"),
    // SC2077: "You need spaces around the comparison operator."
    ("SC2077", "BRS0005"),
    // SC2081: "[ .. ] can't match globs. Use a case statement."
    ("SC2081", "BRS0006"),
    // SC2087: "Quote 'EOF' to make here document expansions happen on the server side rather than on the client."
    ("SC2087", "BRS0007"),
    // SC2095: "ssh may swallow stdin, preventing this loop from working properly."
    ("SC2095", "BRS0008"),
    // SC2096: "On most OS, shebangs can only specify a single parameter."
    ("SC2096", "BRS0009"),
    // SC2104: "In functions, use return instead of break."
    // (bashrs' check here — "Missing space before ]" — is ShellCheck's SC1020,
    //  which bashrs already implements separately as SC1020.)
    ("SC2104", "BRS0010"),
    // SC2114: "Warning: deletes a system directory."
    ("SC2114", "BRS0011"),
    // SC2117: "To run commands as another user, use su -c or sudo."
    ("SC2117", "BRS0012"),
    // SC2141: "This backslash is literal. Did you mean IFS=$'\\n'?"
    ("SC2141", "BRS0013"),
    // SC2165: "This nested loop overrides the index variable of its parent."
    ("SC2165", "BRS0014"),
    // SC2183: "This format string has N variables, but is passed M arguments."
    ("SC2183", "BRS0015"),
    // SC2223: "This default assignment may cause DoS due to globbing. Quote it."
    ("SC2223", "BRS0016"),
    // SC2224: "This mv has no destination. Check the arguments."
    ("SC2224", "BRS0017"),
    // SC2227: "Redirection applies to the find command itself. Rewrite to work per action (or move to end)."
    ("SC2227", "BRS0018"),
    // SC2231: "Quote expansions in this for loop glob to prevent wordsplitting, e.g. \"$dir\"/*.txt ."
    ("SC2231", "BRS0019"),
    // SC2233: "Remove superfluous (..) around condition to avoid subshell overhead."
    ("SC2233", "BRS0020"),
    // SC2266: "Use || for logical OR. Single | will pipe."
    ("SC2266", "BRS0021"),
    // SC2268: "Avoid x-prefix in comparisons as it no longer serves a purpose."
    ("SC2268", "BRS0022"),
    // SC2269: "This variable is assigned to itself, so the assignment does nothing."
    ("SC2269", "BRS0023"),
    // SC2282: "Variable names can't start with numbers, so this is interpreted as a command."
    ("SC2282", "BRS0024"),
    // SC2286: "This empty string is interpreted as a command name. Double check syntax (or use 'true' as a no-op)."
    ("SC2286", "BRS0025"),
    // SC2311: "Bash implicitly disabled set -e for this function invocation because it's inside a command substitution."
    ("SC2311", "BRS0026"),
    // Found by the guard below, not by the corpus census: these fire rarely
    // enough that 45 real scripts never triggered them, which is exactly why a
    // hand-checked list is the wrong instrument.
    // SC2061: "Quote the parameter to -name so the shell won't interpret it." (bashrs' is about `tr`)
    ("SC2061", "BRS0027"),
    // SC2235: "Use { ..; } instead of (..) to avoid subshell overhead."
    ("SC2235", "BRS0028"),
    // SC2248: "Prefer double quoting even when variables don't contain special characters."
    ("SC2248", "BRS0029"),
    // SC2267: "GNU xargs -i is deprecated in favor of -I{}"
    ("SC2267", "BRS0030"),
    // SC2283: "Remove spaces around = to assign (or use [ ] to compare, or quote '=' if literal)."
    ("SC2283", "BRS0031"),
    // SC2287: "This is interpreted as a command name ending with '/'. Double check syntax."
    ("SC2287", "BRS0032"),
    // SC2289: "This is interpreted as a command name containing a linefeed. Double check syntax."
    ("SC2289", "BRS0033"),
    // SC2291: "Quote repeated spaces to avoid them collapsing into one."
    ("SC2291", "BRS0034"),
    // SC2292: "Prefer [[ ]] over [ ] for tests in Bash/Ksh."
    ("SC2292", "BRS0035"),
    // SC2294: "eval negates the benefit of arrays. Drop eval to preserve whitespace/symbols."
    ("SC2294", "BRS0036"),
];

/// Checks withdrawn rather than renamed, with the reason.
///
/// A rename is the right answer for a check that is accurate but misfiled.
/// It is the wrong answer for a check that reports a defect it has not found:
/// moving the noise to a new number keeps burying the real findings. Retiring
/// also releases the ShellCheck number, so the real check can be implemented
/// under it later.
pub const RETIRED: &[(&str, &str)] = &[(
    "SC2032",
    // Fired on every plain `VAR=value` in every script carrying a shebang, on
    // the theory that "variables set in an executed script don't affect the
    // calling shell". That is a property of shell, true of every correct
    // script, not a defect at the flagged line — and the rule cannot tell an
    // executed script (no defect) from one meant to be sourced (the only case
    // where it would matter), because it keys on the shebang alone and stays
    // SILENT on shebang-less files, which is exactly what a sourced file is.
    // Measured precision on the 45-script rmedia corpus: 0 of 461; across 4000
    // scripts under ~/src: 0 of 13404. ShellCheck uses SC2032 for "Use own
    // script or sh -c '..' to run this from sudo.", now released.
    "unfalsifiable: fires on correct code by construction (0/13404 true positives measured)",
)];

/// Codes bashrs shares with ShellCheck where the two checks were compared and
/// found to mean the same thing. Keeping the SC number here is the GOAL — it is
/// what makes a shared `# shellcheck disable=` do the right thing in both tools.
///
/// Membership is a claim that someone read both messages. It is not a place to
/// park a code to quiet the guard.
pub const PARITY: &[&str] = &[
    "SC1003", "SC2001", "SC2021", "SC2024", "SC2043", "SC2050", "SC2054", "SC2094", "SC2103",
    "SC2119", "SC2120", "SC2124", "SC2209", "SC1007", "SC1014", "SC1020", "SC1028", "SC1078",
    "SC1079", "SC1083", "SC1087", "SC1090", "SC1091", "SC2002", "SC2003", "SC2004", "SC2005",
    "SC2006", "SC2007", "SC2015", "SC2016", "SC2018", "SC2019", "SC2025", "SC2028", "SC2029",
    "SC2030", "SC2031", "SC2034", "SC2035", "SC2037", "SC2038", "SC2044", "SC2046", "SC2048",
    "SC2053", "SC2059", "SC2060", "SC2062", "SC2063", "SC2068", "SC2076", "SC2086", "SC2088",
    "SC2089", "SC2090", "SC2091", "SC2097", "SC2098", "SC2102", "SC2105", "SC2112", "SC2113",
    "SC2115", "SC2116", "SC2122", "SC2125", "SC2126", "SC2128", "SC2129", "SC2140", "SC2143",
    "SC2145", "SC2148", "SC2153", "SC2154", "SC2155", "SC2156", "SC2157", "SC2162", "SC2164",
    "SC2166", "SC2168", "SC2178", "SC2181", "SC2182", "SC2188", "SC2198", "SC2204", "SC2206",
    "SC2207", "SC2230", "SC2236", "SC2244", "SC2249", "SC2295", "SC2310",
    // Divergent in SCOPE rather than in subject, and being reworked by the
    // false-positive lanes that own them; renaming them here would collide with
    // that work. Recorded in docs/SC-CODE-COLLISIONS.md, not resolved.
    "SC1035", // ShellCheck: space after `!` only; bashrs generalised to any keyword
    "SC2111", // ShellCheck: `function` + `()` together in ksh; bashrs: `function` in sh
];

/// The code a diagnostic should carry. Total, and idempotent.
pub fn canonical(code: &str) -> &str {
    for (legacy, new) in MIGRATIONS {
        if *legacy == code {
            return new;
        }
    }
    code
}

/// The legacy code a `BRS####` diagnostic used to be reported as, if any.
/// Used to keep existing `# bashrs disable=` pragmas working.
pub fn legacy_alias(code: &str) -> Option<&'static str> {
    MIGRATIONS
        .iter()
        .find(|(_, new)| *new == code)
        .map(|(legacy, _)| *legacy)
}

pub fn is_retired(code: &str) -> bool {
    RETIRED.iter().any(|(c, _)| *c == code)
}

pub fn is_parity(code: &str) -> bool {
    PARITY.contains(&code)
}

/// Rewrite every diagnostic onto its canonical code.
///
/// Must run BEFORE suppression filtering, so a pragma naming the new code
/// works; `SuppressionManager` expands bashrs-syntax pragmas naming the old
/// code, so those keep working too.
pub fn apply(result: &mut LintResult) {
    for diag in &mut result.diagnostics {
        let canon = canonical(&diag.code);
        if canon != diag.code {
            diag.code = canon.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Diagnostic, Severity, Span};

    #[test]
    fn canonical_maps_measured_collisions_out_of_the_sc_namespace() {
        assert_eq!(canonical("SC2311"), "BRS0026");
        assert_eq!(canonical("SC2032"), "SC2032"); // retired, not renamed
        assert_eq!(canonical("SC2086"), "SC2086"); // parity, untouched
    }

    #[test]
    fn canonical_is_idempotent() {
        for (_, new) in MIGRATIONS {
            assert_eq!(canonical(canonical(new)), *new);
        }
    }

    #[test]
    fn legacy_alias_round_trips_every_migration() {
        for (legacy, new) in MIGRATIONS {
            assert_eq!(legacy_alias(new), Some(*legacy));
        }
        assert_eq!(legacy_alias("SC2086"), None);
    }

    /// A rename must not drop or duplicate findings.
    #[test]
    fn apply_preserves_diagnostic_count_and_spans() {
        let mut result = LintResult::new();
        for code in ["SC2311", "SEC011", "SC2086", "SC2266"] {
            result.add(Diagnostic::new(
                code,
                Severity::Error,
                "m",
                Span::new(1, 1, 1, 2),
            ));
        }
        let before = result.diagnostics.len();
        apply(&mut result);
        assert_eq!(result.diagnostics.len(), before);
        let codes: Vec<&str> = result.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["BRS0026", "SEC011", "SC2086", "BRS0021"]);
    }

    #[test]
    fn retired_entries_carry_a_reason() {
        for (code, why) in RETIRED {
            assert!(!why.is_empty(), "{code} was retired without a reason");
        }
    }
}
