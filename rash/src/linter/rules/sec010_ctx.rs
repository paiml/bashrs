//! GH-336: which lines of a script may have a child-shell script masked.
//!
//! One rule. Masking is ON from the first line of the file. The first line the
//! scanner does not fully understand turns it OFF for that line and for the
//! REST OF THE FILE; it never comes back on. So only the lines before the first
//! here-doc opener, backtick, carriage return, `$'`, quote left open at the end
//! of a line, `$(` that does not close on its line, or `#` comment holding a
//! quote, can ever be masked. There is no here-doc or multi-line-quote tracking.

use super::sec010_quotes::{top_level_single_spans, Scan};

/// One entry per physical line: may `mask_child_shell_scripts` run on it?
pub(super) fn maskable_lines(source: &str) -> Vec<bool> {
    let mut on = !source.contains('\r');
    source
        .lines()
        .map(|line| {
            on = on && understood(line);
            on
        })
        .collect()
}

/// True when the line is fully understood: it scans cleanly to top level and
/// holds none of the constructs that need state carried across lines.
fn understood(line: &str) -> bool {
    matches!(top_level_single_spans(line), Scan::Spans(_))
        && !line.contains("<<")
        && !line.contains("$'")
        && !comment_with_quote(line)
}

/// A `#` followed by a quote character, wherever the `#` sits: the scanner and
/// bash may disagree on whether it starts a comment.
fn comment_with_quote(line: &str) -> bool {
    line.find('#').is_some_and(|p| line[p..].contains(['\'', '"']))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(s: &str) -> Vec<bool> {
        maskable_lines(s)
    }

    #[test]
    fn test_plain_lines_are_maskable() {
        assert_eq!(m("a\nb\n"), vec![true, true]);
    }

    #[test]
    fn test_first_heredoc_opener_turns_masking_off_for_good() {
        assert_eq!(m("a\ncat <<EOF\nx\nEOF\ny"), vec![true, false, false, false, false]);
        assert_eq!(m("cat <<< x\ny"), vec![false, false]);
    }

    #[test]
    fn test_open_quote_turns_masking_off_for_good() {
        assert_eq!(m("a=\"\nx\n\"\ny"), vec![false, false, false, false]);
        assert_eq!(m("a='\nx\n'\ny"), vec![false, false, false, false]);
    }

    #[test]
    fn test_backtick_balanced_or_not_turns_masking_off() {
        assert_eq!(m("a\nx=`d`\ny"), vec![true, false, false]);
        assert_eq!(m("a=`\ny"), vec![false, false]);
    }

    #[test]
    fn test_command_substitution_must_close_on_its_line() {
        assert_eq!(m("a $(b)\nc"), vec![true, true]);
        assert_eq!(m("a $(\nb)\nc"), vec![false, false, false]);
    }

    #[test]
    fn test_carriage_return_anywhere_turns_masking_off_everywhere() {
        assert_eq!(m("a\nb\r\nc"), vec![false, false, false]);
    }

    #[test]
    fn test_ansi_c_quote_turns_masking_off() {
        assert_eq!(m("a\necho $'x'\nb"), vec![true, false, false]);
    }

    #[test]
    fn test_comment_with_quote_turns_masking_off_but_plain_comment_does_not() {
        assert_eq!(m("a # fine\nb"), vec![true, true]);
        assert_eq!(m("a;# it's\nb"), vec![false, false]);
        assert_eq!(m("a # <<EOF\nb"), vec![false, false]);
    }
}
