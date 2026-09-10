//! PMAT-251: SC1009 false positive on a comment-led non-empty body (GH-238).
//!
//! `bashrs lint` reported "Comment here is not a command" whenever a comment
//! was the FIRST line after `then`/`do`/`else`/`{`, regardless of what
//! followed it. The body of the reported case is not empty, so the
//! diagnostic's own premise was false.
//!
//! Measured on the released 7.0.3 binary: `lint_shell` reports the
//! diagnostic under code `BRS0001`, not `SC1009` - `code_namespace::MIGRATIONS`
//! renames it because ShellCheck's own SC1009 means something else. These
//! tests go through `lint_shell`, so they see the code a real caller sees.

use crate::linter::lint_shell;

fn codes(src: &str) -> Vec<String> {
    lint_shell(src)
        .diagnostics
        .into_iter()
        .map(|d| d.code)
        .collect()
}

/// Precise: a comment that merely LEADS a non-empty body is not reported.
#[test]
fn test_PMAT251_gh238_comment_led_nonempty_block_is_not_reported() {
    let src = "#!/bin/bash\nif true; then\n  # just a comment\n  echo hi\nfi\n";
    let cs = codes(src);
    assert!(
        !cs.contains(&"BRS0001".to_string()),
        "BRS0001 (SC1009) must NOT fire when a real command follows the leading comment, got {:?}",
        cs
    );
}

/// Sound: a block whose body is ONLY comments (no command at all) must
/// still be reported, or the fix has simply switched the rule off.
#[test]
fn test_PMAT251_gh238_comment_only_block_is_still_reported() {
    let src = "#!/bin/bash\nif true; then\n  # just a comment\nfi\n";
    let cs = codes(src);
    assert!(
        cs.contains(&"BRS0001".to_string()),
        "BRS0001 (SC1009) must still fire on a genuinely comment-only body, got {:?}",
        cs
    );
}
