//! RED-phase reproducers for GH-256 and GH-257 (PMAT-251).
//!
//! GH-256: MAKE010 false-positives on (1) a critical-command word that only
//! appears inside a double-quoted string, (2) a subcommand token (`install`
//! in `cargo install`) mistaken for the command name, and (3) a plain
//! (non-continued) recipe line where the rule's `|| exit 1` premise does not
//! hold, because Make already aborts the target when that line's own exit
//! status is non-zero.
//!
//! GH-257: MAKE010 flags `rm -f` / `mkdir -p` / etc as "missing error
//! handling" even though the flag itself already expresses the tolerance the
//! rule is asking for — while a genuinely intolerant command in the same
//! compound recipe body (`cp` in the `restore:` example) must still be
//! reported.
//!
//! All assertions go through the public `lint_makefile` entry point (not the
//! rule's own `check`), per PMAT-251 instructions, using the issues' own
//! Makefile text as input.

use crate::linter::lint_makefile;

fn make010_diagnostics(source: &str) -> Vec<crate::linter::Diagnostic> {
    lint_makefile(source)
        .diagnostics
        .into_iter()
        .filter(|d| d.code == "MAKE010")
        .collect()
}

/// GH-256 (1): `install` inside a double-quoted `echo` string is not a
/// command invocation and must not be flagged.
#[test]
fn test_PMAT251_gh256_command_word_inside_string_is_not_a_command() {
    let makefile = "check:\n\t@tool --version || echo \"not found. Install with: cargo install tool\"\n";
    let diags = make010_diagnostics(makefile);
    assert!(
        diags.is_empty(),
        "expected no MAKE010 findings for a command word inside a string, got: {diags:?}"
    );
}

/// GH-256 (2): `install` is `cargo`'s subcommand, not the command name —
/// only the first word of a simple command is a command name.
#[test]
fn test_PMAT251_gh256_subcommand_token_is_not_the_command() {
    let makefile = "verify:\n\t@cargo install mycrate --force\n";
    let diags = make010_diagnostics(makefile);
    assert!(
        diags.is_empty(),
        "expected no MAKE010 findings for a subcommand token, got: {diags:?}"
    );
}

/// GH-256 (3): a critical command that is the sole command on a plain
/// (non-continued) recipe line already gets Make's own abort-on-failure
/// behavior for free — its own exit status IS the recipe line's exit status.
/// `|| exit 1` there is a no-op, so the finding is noise.
#[test]
fn test_PMAT251_gh256_plain_recipe_line_not_flagged() {
    let makefile = "install:\n\tcp app /usr/bin/app\n";
    let diags = make010_diagnostics(makefile);
    assert!(
        diags.is_empty(),
        "expected no MAKE010 finding on a plain single-command recipe line, got: {diags:?}"
    );
}

/// GH-257: a flag that already declares the tolerance MAKE010 is asking for
/// must satisfy the rule, even when the command is not the last one in its
/// compound recipe body (so the positional GH-256(3) fix alone would not
/// have excused it).
#[test]
fn test_PMAT251_gh257_tolerant_flags_are_not_missing_error_handling() {
    let cases = [
        "clean:\n\trm -f old.txt; \\\n\ttrue\n",
        "clean:\n\trm -rf build; \\\n\ttrue\n",
        "prep:\n\tmkdir -p out; \\\n\ttrue\n",
        "link:\n\tln -sf target link; \\\n\ttrue\n",
        "stage:\n\tcp -f a b; \\\n\ttrue\n",
    ];

    for makefile in cases {
        let diags = make010_diagnostics(makefile);
        assert!(
            diags.is_empty(),
            "expected no MAKE010 finding for a tolerant-flag command in {makefile:?}, got: {diags:?}"
        );
    }
}

/// GH-257: the `restore:` recipe from the issue — `cp b a` is a true
/// positive (a failed `cp` would leave `rm -f b` deleting the only backup)
/// and must still be reported; `rm -f b` must not.
#[test]
fn test_PMAT251_gh257_intolerant_command_in_compound_body_is_still_reported() {
    let makefile = "restore:\n\t@if [ -f b ]; then \\\n\t\tcp b a; \\\n\t\trm -f b; \\\n\tfi\n";
    let diags = make010_diagnostics(makefile);

    assert_eq!(
        diags.len(),
        1,
        "expected exactly one MAKE010 finding (cp), got: {diags:?}"
    );
    assert!(
        diags[0].message.contains("'cp'"),
        "expected the finding to name 'cp', got: {:?}",
        diags[0].message
    );
    assert!(
        !diags.iter().any(|d| d.message.contains("'rm'")),
        "rm -f must not be flagged: {diags:?}"
    );
}
