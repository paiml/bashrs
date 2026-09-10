//! PMAT-251: SC2188 scope gap (GH-249).
//!
//! `LONE_REDIRECT` was `^\s*[<>]`, so a line beginning with a file-descriptor
//! digit (`2>&1` alone) was never examined at all, while `>&2` alone was
//! reported — both are "redirection without a command". Widened to
//! `^\s*\d*[<>]` in `sc2188.rs`.
//!
//! Each test here runs the issue's own reproducer through the public
//! `lint_shell` entry point, so it exercises the real pipeline: quote
//! masking (SC2188 is quote-sensitive), the code-namespace pass and inline
//! suppression - not just the bare `check()` function.

use crate::linter::lint_shell;

fn codes(src: &str) -> Vec<String> {
    lint_shell(src)
        .diagnostics
        .into_iter()
        .map(|d| d.code)
        .collect()
}

/// Sound: a lone fd-prefixed redirection is reported, exactly like a lone
/// `>&2` already was.
#[test]
fn test_PMAT251_gh249_lone_fd_redirect_is_reported() {
    for src in [
        "#!/bin/sh\n2>&1\n",
        "#!/bin/sh\n2>&1 \n",
        "#!/bin/sh\n1>&2\n",
    ] {
        let cs = codes(src);
        assert!(
            cs.contains(&"SC2188".to_string()),
            "SC2188 must fire on a lone fd-prefixed redirection `{}`, got {:?}",
            src.trim(),
            cs
        );
    }
}

/// Precise: a redirection attached to a real command - before it, after it,
/// or via `exec` - is not a lone redirect and must never be reported.
/// Every one of these is accepted by `dash -n` (measured).
#[test]
fn test_PMAT251_gh249_redirect_attached_to_a_command_is_not_reported() {
    for src in [
        "#!/bin/sh\ncmd 2>&1\n",
        "#!/bin/sh\nexec 2>&1\n",
        "#!/bin/sh\n2>&1 cmd\n",
        "#!/bin/sh\n1>&2 echo hi\n",
        "#!/bin/sh\n3<file cat\n",
    ] {
        let cs = codes(src);
        assert!(
            !cs.contains(&"SC2188".to_string()),
            "SC2188 must NOT fire on a redirection attached to a command `{}`, got {:?}",
            src.trim(),
            cs
        );
    }
}

/// Precise: an fd-prefixed redirection inside a heredoc body, or inside a
/// comment, is text - not a redirection - and must not be reported. The
/// widened regex must not turn a documentation string into a false positive.
#[test]
fn test_PMAT251_gh249_lone_redirect_in_heredoc_or_comment_is_not_reported() {
    let heredoc = "#!/bin/sh\ncat <<EOF\nexample: 2>&1\nEOF\n";
    let comment = "#!/bin/sh\n# 2>&1\necho hi\n";
    for src in [heredoc, comment] {
        let cs = codes(src);
        assert!(
            !cs.contains(&"SC2188".to_string()),
            "SC2188 must NOT fire on text inside a heredoc/comment `{}`, got {:?}",
            src,
            cs
        );
    }
    // Guard the guard: a genuine lone redirect elsewhere in the same kind of
    // script is still caught, so this is not just a rule gone silent.
    let still_fires = "#!/bin/sh\ncat <<EOF\nexample text\nEOF\n2>&1\n";
    assert!(
        codes(still_fires).contains(&"SC2188".to_string()),
        "a real lone redirect after a heredoc must still be reported"
    );
}
