//! Heredoc region scanning, shared by every lint rule.
//!
//! GH-217: the rules see a flat stream of physical lines with no notion of
//! heredoc regions, so any line-oriented rule fires *inside* heredoc bodies. A
//! **quoted** heredoc body (`<<'EOF'` / `<<"EOF"`) is literal text by
//! definition — that is the entire point of quoting the delimiter — so shell
//! rules must not analyse it. Embedding Python, awk or jq that way is a common
//! idiom, and `SC1007` is `Severity::Error`, so the false positives block
//! commits.
//!
//! This lives in shared scanning code rather than in individual rules on
//! purpose. `sc2006` already had a private copy of this logic (issue #96) and
//! the other 384 rules did not, which is precisely how GH-217 happened: a
//! per-rule fix cannot generalise, and the next line-oriented rule would have
//! reintroduced the bug. Applying it once, where diagnostics are aggregated,
//! covers every rule that exists and every rule not yet written.
//!
//! **Unquoted heredocs are deliberately NOT skipped.** Their bodies undergo
//! parameter expansion and command substitution, so they really are shell and
//! rules like SC2006 should still fire there.

use std::collections::HashSet;

/// Parse the opening delimiter of a quoted heredoc, if this token starts one.
///
/// Returns the delimiter without quotes. Handles `<<'X'`, `<< 'X'`, `<<-'X'`
/// and the double-quoted forms.
fn parse_quoted_delimiter(rest: &str) -> Option<String> {
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    let rest = rest.trim_start();
    let quote = rest.chars().next()?;
    if quote != '\'' && quote != '"' {
        // Unquoted heredoc: body IS expanded, so it is genuinely shell. Skip.
        return None;
    }
    let after = &rest[quote.len_utf8()..];
    let end = after.find(quote)?;
    let delim = &after[..end];
    if delim.is_empty() {
        return None;
    }
    Some(delim.to_string())
}

/// Every `<<`-opener on a line, in order. A single command may open more than
/// one (`cmd <<'A' <<'B'`), and POSIX consumes their bodies in that order.
fn quoted_openers_on_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            // `<<<` is a here-STRING, not a heredoc — it has no body.
            if bytes.get(i + 2) == Some(&b'<') {
                i += 3;
                continue;
            }
            if let Some(delim) = parse_quoted_delimiter(&line[i + 2..]) {
                out.push(delim);
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// 1-based line numbers that fall inside a **quoted** heredoc body.
///
/// The opening line itself is excluded — it is real shell and rules should
/// still see it. The terminating delimiter line is excluded too.
///
/// Implemented as a state machine rather than "regex-match every line", because
/// scanning every line for openers also matches openers that appear *inside* a
/// body (e.g. a heredoc containing shell examples), which silently extends the
/// suppressed region past where it should end.
pub fn quoted_heredoc_lines(source: &str) -> HashSet<usize> {
    let mut inside = HashSet::new();
    let mut pending: Vec<String> = Vec::new();
    let mut active: Option<String> = None;

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;

        if let Some(delim) = active.clone() {
            // `<<-` permits a tab-indented terminator; trimming is the lenient
            // reading and matches the previous behaviour in sc2006.
            if line.trim() == delim {
                // Terminator reached; a second opener from the same command
                // (`cmd <<'A' <<'B'`) starts consuming immediately.
                active = if pending.is_empty() {
                    None
                } else {
                    Some(pending.remove(0))
                };
            } else {
                inside.insert(line_num);
            }
            continue;
        }

        let mut openers = quoted_openers_on_line(line);
        if !openers.is_empty() {
            active = Some(openers.remove(0));
            pending = openers;
        }
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_python_body_is_a_region() {
        // The exact reproduction from GH-217.
        let src = "#!/bin/sh\npython3 - <<'PY'\np = 1\nq = 2\nPY\necho done\n";
        let r = quoted_heredoc_lines(src);
        assert!(r.contains(&3), "python line 3 must be inside the region");
        assert!(r.contains(&4), "python line 4 must be inside the region");
        assert!(!r.contains(&2), "the opening line is real shell");
        assert!(!r.contains(&5), "the terminator is not body");
        assert!(!r.contains(&6), "code after the heredoc must still be linted");
    }

    #[test]
    fn unquoted_heredoc_is_not_a_region() {
        // Body IS expanded, so it is shell and rules must still fire.
        let src = "cat <<EOF\n`date`\nEOF\n";
        assert!(quoted_heredoc_lines(src).is_empty());
    }

    #[test]
    fn double_quoted_and_dash_forms() {
        let src = "cat <<-\"EOF\"\n\tbody\n\tEOF\nafter\n";
        let r = quoted_heredoc_lines(src);
        assert!(r.contains(&2));
        assert!(!r.contains(&4));
    }

    #[test]
    fn here_string_has_no_body() {
        let src = "cat <<<'literal'\necho after\n";
        assert!(quoted_heredoc_lines(src).is_empty());
    }

    #[test]
    fn opener_inside_a_body_does_not_extend_the_region() {
        // A heredoc whose body documents another heredoc. The naive
        // scan-every-line approach treats line 2 as a new opener and swallows
        // everything after it.
        let src = "cat <<'DOC'\nexample: cat <<'INNER'\nDOC\necho after\n";
        let r = quoted_heredoc_lines(src);
        assert!(r.contains(&2), "the doc line is body");
        assert!(!r.contains(&4), "code after the outer heredoc is still linted");
    }

    #[test]
    fn two_heredocs_on_one_line_consume_bodies_in_order() {
        let src = "diff <<'A' <<'B'\na1\nA\nb1\nB\nafter\n";
        let r = quoted_heredoc_lines(src);
        assert!(r.contains(&2), "first body");
        assert!(r.contains(&4), "second body");
        assert!(!r.contains(&6), "code after both is linted");
    }
}
