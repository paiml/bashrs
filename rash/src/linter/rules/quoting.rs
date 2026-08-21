//! Shared quote-context helpers for line-based lint rules.
//!
//! WHY THIS EXISTS
//!
//! Shell-syntax rules that scan a line with a regex must not interpret the
//! CONTENTS of quoted strings as shell syntax. At least eight rules had each
//! reimplemented that check privately (det003, make003, sc1004, sc1045, sc1079,
//! sc1097, sc1099, ...), and the ones that had NOT reimplemented it produced
//! false positives at `Severity::Error`:
//!
//!   * SC2104 flagged the `]` in `echo 'usage: prog [--a|--b]'` (issue #244)
//!   * SC1028 flagged the parens in `log "waiting (${n}s elapsed)"` (issue #243)
//!
//! Both are shellcheck-clean, both are ordinary shell, and neither has a correct
//! rewrite — so at error severity they made common idioms unlintable, which is
//! how a gate teaches people to bypass it.
//!
//! New rules should call these instead of hand-rolling a ninth copy.

/// True if byte position `pos` on `line` falls inside a single- or
/// double-quoted string.
///
/// Inside single quotes a backslash is literal (POSIX), so escapes are honoured
/// only within double quotes.
pub fn is_inside_quoted_string(line: &str, pos: usize) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let limit = pos.min(chars.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < limit {
        match chars[i] {
            '\\' if in_double => {
                i += 2;
                continue;
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            _ => {}
        }
        i += 1;
    }

    in_single || in_double
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_inside_single_quotes() {
        let line = "echo 'a [b] c'";
        assert!(
            is_inside_quoted_string(line, 9),
            "position 9 is inside '...'"
        );
    }

    #[test]
    fn detects_inside_double_quotes() {
        let line = "log \"waiting (5s)\"";
        assert!(
            is_inside_quoted_string(line, 14),
            "position 14 is inside \"...\""
        );
    }

    #[test]
    fn outside_quotes_is_false() {
        let line = "[ -n \"$x\" ] && echo hi";
        assert!(!is_inside_quoted_string(line, 0));
        assert!(!is_inside_quoted_string(line, line.len() - 1));
    }

    /// A quote that has been CLOSED must not leave the tracker stuck open —
    /// otherwise every rule using this would silently stop firing after the
    /// first string on the line.
    #[test]
    fn closed_quotes_do_not_leak() {
        let line = "echo 'done' ; [ -n \"$x\"]";
        assert!(
            !is_inside_quoted_string(line, line.len() - 1),
            "the trailing ] is OUTSIDE the closed quotes and must remain lintable"
        );
    }

    #[test]
    fn escaped_quote_inside_double_does_not_close() {
        let line = "echo \"a \\\" b (c)\"";
        assert!(is_inside_quoted_string(line, line.len() - 2));
    }
}
