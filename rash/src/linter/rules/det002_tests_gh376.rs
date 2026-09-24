//! GH-376: DET002 resolved a timestamp's destination one PHYSICAL line at a
//! time, so a command continued with a trailing `\` lost its redirect. The
//! same `jq … >> "$audit_log"` passed on one line and failed on four.
//! Continuations are now joined into one logical line before the sink is
//! resolved; the diagnostic still points at the physical line and column of
//! the `date`, because `# bashrs disable-line=DET002` is keyed on it.

#![allow(non_snake_case)]

use super::check;

const ONE_LINE: &str = "#!/bin/sh\n\
audit_log=\"/x/gguf-stage.audit.jsonl\"\n\
jq -cn --arg at \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" '{at:$at}' >> \"$audit_log\"\n";

const SPLIT: &str = "#!/bin/sh\n\
audit_log=\"/x/gguf-stage.audit.jsonl\"\n\
jq -cn \\\n\
  --arg at \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" \\\n\
  '{at:$at}' \\\n\
  >> \"$audit_log\"\n";

#[test]
fn test_GH376_one_line_append_to_log_is_clean() {
    // Control: the form bashrs already accepted.
    assert_eq!(check(ONE_LINE).diagnostics.len(), 0);
}

#[test]
fn test_GH376_continued_append_to_log_is_clean() {
    // Issue #376 reproducer, verbatim: same command, split with `\`.
    let d = check(SPLIT).diagnostics;
    assert_eq!(
        d.len(),
        0,
        "a `\\`-continued command must be judged on its whole logical line: {d:?}"
    );
}

#[test]
fn test_GH376_continued_truncating_redirect_still_fires_at_the_date() {
    // Joining must not launder a real sink: the continued command still ends
    // in a truncating write to an artifact, and the span stays on line 4, the
    // physical line holding `$(date`, at its physical column (a joined line
    // would put it on line 3).
    let src = "#!/bin/sh\n\
out=build.json\n\
jq -cn \\\n\
  --arg at \"$(date -u +%Y%m%d)\" \\\n\
  '{at:$at}' \\\n\
  > \"$out\"\n";
    let d = check(src).diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].span.start_line, 4);
    // Rust's `\`-newline string continuation drops the next line's leading
    // spaces, so line 4 is `--arg at "$(date …`: the `$` is byte 10.
    assert_eq!(d[0].span.start_col, 11);
}

#[test]
fn test_GH376_backslash_in_a_comment_does_not_continue() {
    // A `\` ending a comment is comment text. Joining would fold the next
    // line into the comment and hide the timestamp it writes to VERSION.
    let src = "#!/bin/sh\necho done # see notes \\\ndate +%s > VERSION\n";
    let d = check(src).diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].span.start_line, 3);
}

#[test]
fn test_GH376_escaped_backslash_does_not_continue() {
    // `\\` at end of line is a literal backslash, not a continuation. Joined,
    // line 3 would read as an argument of `echo` (stdout, benign) and vanish;
    // unjoined it is an unused capture, which default-deny reports.
    let src = "#!/bin/sh\necho a\\\\\nts=$(date +%s)\n";
    let d = check(src).diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].span.start_line, 3);
}
