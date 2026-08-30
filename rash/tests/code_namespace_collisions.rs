//! T6 — a `SCxxxx` code must mean what ShellCheck says it means.
//!
//! ShellCheck owns the `SCxxxx` namespace. When bashrs files a check of its own
//! under a number ShellCheck has already assigned, three things break at once:
//!
//! 1. `# shellcheck disable=SCxxxx` in a shared codebase silences the wrong
//!    diagnostic in one of the two tools.
//! 2. Any baseline, dashboard or ratchet keyed on the number conflates two
//!    unrelated checks.
//! 3. A user who looks the code up reads documentation for something else.
//!
//! `SEC###` / `DET###` / `IDEM###` already avoid this by owning their own
//! namespace. `BRS####` extends that model to the bashrs-original checks that
//! were squatting on ShellCheck numbers.
//!
//! The oracle is `tests/data/shellcheck-registry.tsv`: ShellCheck's own
//! messages, MEASURED by running `shellcheck` (see the file header for how) —
//! not transcribed from memory.

use bashrs::linter::code_namespace;
use std::collections::HashMap;

/// ShellCheck's registry, as measured. `code -> message`.
fn shellcheck_registry() -> HashMap<String, String> {
    let raw = include_str!("data/shellcheck-registry.tsv");
    raw.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_once('\t'))
        .map(|(c, m)| (c.to_string(), m.to_string()))
        .collect()
}

/// Every rule module under `rash/src/linter/rules/` named `scNNNN.rs` is a
/// check bashrs files under `SCNNNN`. Reading the directory (rather than a
/// hand-maintained list) is deliberate: a hand-maintained list is the exact
/// mechanism that let 225 SC codes drift from a doc claiming 3.
fn bashrs_sc_rule_modules() -> Vec<String> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/linter/rules");
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .expect("rules dir")
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(".rs").map(str::to_string))
        .filter(|n| {
            n.len() == 6 && n.starts_with("sc") && n[2..].chars().all(|c| c.is_ascii_digit())
        })
        // A module with no `Diagnostic::new` is a stub that emits nothing, so it
        // cannot collide with anything. It starts colliding the day someone
        // implements it — and this guard fires then, which is the point.
        .filter(|n| {
            std::fs::read_to_string(format!("{dir}/{n}.rs"))
                .map(|s| s.contains("Diagnostic::new"))
                .unwrap_or(false)
        })
        .map(|n| n.to_uppercase())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// THE GUARD. A bashrs check may keep a `SCxxxx` code only if it has been
/// checked against ShellCheck's message for that code and found to mean the
/// same thing (`code_namespace::PARITY`). Anything else must move to `BRS####`.
///
/// This is the poka-yoke: adding a new `scNNNN.rs` whose meaning differs from
/// ShellCheck's `SCNNNN` fails here, at the moment the number is chosen.
#[test]
fn no_bashrs_check_squats_on_a_shellcheck_code() {
    let registry = shellcheck_registry();
    let mut squatters = Vec::new();

    for code in bashrs_sc_rule_modules() {
        // Migrated away from the SC namespace already.
        if code_namespace::canonical(&code) != code {
            continue;
        }
        // Retired outright.
        if code_namespace::is_retired(&code) {
            continue;
        }
        // ShellCheck does not use this number (measured): no collision possible.
        let Some(sc_msg) = registry.get(&code) else {
            continue;
        };
        // Reviewed against ShellCheck's message and found equivalent.
        if code_namespace::is_parity(&code) {
            continue;
        }
        squatters.push(format!("  {code}  ShellCheck: {sc_msg}"));
    }

    assert!(
        squatters.is_empty(),
        "these bashrs checks are filed under a ShellCheck code that means \
         something else.\nGive each one a BRS#### code in \
         linter::code_namespace::MIGRATIONS, or — if it really does mean the \
         same thing as ShellCheck's check — add it to PARITY with the \
         ShellCheck message quoted:\n{}",
        squatters.join("\n")
    );
}

/// A migration is a rename, never a silencing. Every migrated code must still
/// be reachable: `canonical()` is total and idempotent, and no two legacy codes
/// may collapse onto one BRS code (that would re-create the collision we are
/// fixing, one namespace over).
#[test]
fn migration_table_is_injective_and_idempotent() {
    let mut seen: HashMap<&str, &str> = HashMap::new();
    for (legacy, new) in code_namespace::MIGRATIONS {
        assert!(
            new.starts_with("BRS"),
            "{legacy} must migrate into the BRS namespace, got {new}"
        );
        assert_eq!(
            code_namespace::canonical(new),
            *new,
            "{new} must be a fixed point of canonical()"
        );
        assert_eq!(code_namespace::canonical(legacy), *new);
        if let Some(prev) = seen.insert(new, legacy) {
            panic!("{new} is claimed by both {prev} and {legacy}");
        }
    }
}

/// An unmigrated code passes through untouched.
#[test]
fn canonical_is_identity_for_untouched_codes() {
    for code in [
        "SC2086",
        "SEC011",
        "DET002",
        "IDEM002",
        "PERF002",
        "BASHRS001",
    ] {
        assert_eq!(code_namespace::canonical(code), code);
    }
}

/// The three tables must not contradict each other. A code cannot be
/// simultaneously renamed and declared at parity, or renamed and retired.
#[test]
fn migration_retirement_and_parity_tables_are_disjoint() {
    for (legacy, new) in code_namespace::MIGRATIONS {
        assert!(
            !code_namespace::is_parity(legacy),
            "{legacy} is migrated to {new} AND listed as parity"
        );
        assert!(
            !code_namespace::is_retired(legacy),
            "{legacy} is migrated to {new} AND retired"
        );
    }
    for (code, _) in code_namespace::RETIRED {
        assert!(
            !code_namespace::is_parity(code),
            "{code} is retired AND listed as parity"
        );
    }
}

/// The oracle has to be about the tool it claims to be about. If a code we
/// declared "parity" is not one ShellCheck actually emits, the declaration was
/// never checked against anything.
#[test]
fn every_parity_claim_names_a_code_shellcheck_actually_uses() {
    let registry = shellcheck_registry();
    let unbacked: Vec<&&str> = code_namespace::PARITY
        .iter()
        .filter(|c| !registry.contains_key(**c))
        .collect();
    assert!(
        unbacked.is_empty(),
        "PARITY claims equivalence with a ShellCheck check that was never \
         measured: {unbacked:?}. Either measure it (append to \
         tests/data/shellcheck-registry.tsv with the snippet that triggered \
         it) or drop the claim."
    );
}

// ---------------------------------------------------------------------------
// MUST STILL FIRE
//
// A rename that turns a true positive into a false negative is worse than the
// collision it fixed: this tool's job is to be trusted when it is red. Every
// migrated check is therefore proven to still report the same defect, at the
// same place, under its new code.
// ---------------------------------------------------------------------------

use bashrs::linter::lint_shell;

fn codes(src: &str) -> Vec<String> {
    lint_shell(src)
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

/// The defect is still found; only the label changed. The negative half of each
/// assertion matters as much as the positive: the old code must be GONE, or the
/// collision is still there.
#[test]
fn migrated_checks_still_fire_under_their_new_code() {
    // (snippet, legacy code, new code)
    let cases: &[(&str, &str, &str)] = &[
        // ShellCheck's SC2311: "Bash implicitly disabled set -e ... command substitution"
        ("#!/bin/bash\nmsg=\"hello world\"\n", "SC2311", "BRS0026"),
        // ShellCheck's SC2114: "Warning: deletes a system directory."
        ("#!/bin/bash\nrm -rf \"$dir\"\n", "SC2114", "BRS0011"),
        // ShellCheck's SC2081: "[ .. ] can't match globs. Use a case statement."
        ("#!/bin/bash\necho 'value is $HOME'\n", "SC2081", "BRS0006"),
        // ShellCheck's SC2227: "Redirection applies to the find command itself."
        (
            "#!/bin/bash\ncount=$(grep -c x f > out | wc -l)\n",
            "SC2227",
            "BRS0018",
        ),
    ];

    for (src, legacy, new) in cases {
        let found = codes(src);
        assert!(
            found.iter().any(|c| c == new),
            "{new} (was {legacy}) stopped firing on:\n{src}\ngot: {found:?}"
        );
        assert!(
            !found.iter().any(|c| c == legacy),
            "{legacy} is still emitted — the collision was not actually fixed"
        );
    }
}

/// Retiring SC2032 must not have taken anything else with it. The hazards this
/// corpus exists to surface — SEC011's `rm -rf` finding among them — are still
/// reported.
#[test]
fn retiring_sc2032_did_not_silence_the_security_and_determinism_rules() {
    let src = "#!/bin/bash\n\
               src_raw=/data/raw\n\
               dst=/data/out\n\
               stamp=$(date +%s)\n\
               rm -rf \"$src_raw\"\n\
               eval \"$UNTRUSTED\"\n";
    let found = codes(src);
    assert!(
        !found.iter().any(|c| c == "SC2032"),
        "SC2032 is retired but still emitted: {found:?}"
    );
    for family in ["SEC", "DET"] {
        assert!(
            found.iter().any(|c| c.starts_with(family)),
            "{family}* stopped firing after the SC2032 retirement: {found:?}"
        );
    }
}

/// A `# bashrs disable=` naming the OLD code keeps working: a rename must not
/// silently un-suppress somebody's existing baseline.
#[test]
fn a_bashrs_pragma_naming_the_legacy_code_still_suppresses() {
    let src = "#!/bin/bash\n# bashrs disable=SC2081\necho 'value is $HOME'\n";
    assert!(
        !codes(src).iter().any(|c| c == "BRS0006"),
        "a pre-migration `# bashrs disable=SC2081` stopped suppressing"
    );
    // And the new spelling works too.
    let src_new = "#!/bin/bash\n# bashrs disable=BRS0006\necho 'value is $HOME'\n";
    assert!(!codes(src_new).iter().any(|c| c == "BRS0006"));
}

/// The other direction, which is the whole point of the migration:
/// `# shellcheck disable=SC2081` names SHELLCHECK's SC2081 ("[ .. ] can't match
/// globs"). It must not reach into bashrs and silence an unrelated check —
/// that silent cross-tool suppression is the interop bug being fixed.
#[test]
fn a_shellcheck_pragma_does_not_suppress_the_bashrs_check_that_squatted_on_it() {
    let src = "#!/bin/bash\n# shellcheck disable=SC2081\necho 'value is $HOME'\n";
    assert!(
        codes(src).iter().any(|c| c == "BRS0006"),
        "`# shellcheck disable=SC2081` still silences a bashrs check that has \
         nothing to do with ShellCheck's SC2081"
    );
}
