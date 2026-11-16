//! GH-336: which lines of a script may have a child-shell script masked.
//!
//! A line is maskable only if it starts at top level: not inside ANY here-doc
//! body (quoted or not: the parent expands an unquoted body) and not inside a
//! quoted string opened on an earlier line. Anything uncertain is not maskable.

use super::sec010_quotes::{top_level_single_spans, Carry, Scan};

/// One entry per physical line: may `mask_child_shell_scripts` run on it?
pub(super) fn maskable_lines(source: &str) -> Vec<bool> {
    let mut carry = Carry::Top;
    let mut pending: Vec<(String, bool)> = Vec::new();
    let mut out = Vec::new();
    for line in source.lines() {
        if let Some((delim, strip)) = pending.first() {
            let t = if *strip { line.trim_start_matches('\t') } else { line };
            if t.trim_end_matches('\r') == delim {
                pending.remove(0);
            }
            out.push(false);
            continue;
        }
        if carry == Carry::Top {
            let scan = top_level_single_spans(line);
            out.push(matches!(scan, Scan::Spans(_)));
            carry = after(scan);
            if !line.trim_start().starts_with('#') {
                pending.extend(heredoc_openers(line));
            }
        } else {
            out.push(false);
            carry = carried_line(carry, line);
        }
    }
    out
}

/// State after a line that starts inside a quote (or in unknown state).
fn carried_line(carry: Carry, line: &str) -> Carry {
    let rest_after = |p: usize| after(top_level_single_spans(&line[p + 1..]));
    match carry {
        Carry::Single => line.find('\'').map_or(Carry::Single, rest_after),
        Carry::Double => match closing_double(line) {
            None => Carry::Double,
            Some(_) if line.contains("$(") || line.contains('`') => Carry::Unknown,
            Some(p) => rest_after(p),
        },
        other => other,
    }
}

fn after(scan: Scan) -> Carry {
    match scan {
        Scan::Spans(_) | Scan::Uncertain => Carry::Top,
        Scan::Open(c) => c,
    }
}

/// Index of the first unescaped `"`.
fn closing_double(line: &str) -> Option<usize> {
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 1,
            b'"' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Every `<<` / `<<-` opener on a line as (delimiter, strip leading tabs).
/// Over-detection (`1 << 2`) only makes later lines non-maskable.
fn heredoc_openers(line: &str) -> Vec<(String, bool)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] != b'<' || b[i + 1] != b'<' {
            i += 1;
            continue;
        }
        if b.get(i + 2) == Some(&b'<') {
            i += 3;
            continue;
        }
        let mut rest = &line[i + 2..];
        let strip = rest.starts_with('-');
        if strip {
            rest = &rest[1..];
        }
        let rest = rest.trim_start();
        let delim = match rest.chars().next() {
            Some(q @ ('\'' | '"')) => rest[1..].split(q).next().unwrap_or(""),
            _ => rest.split(|c: char| " \t;|&<>()".contains(c)).next().unwrap_or(""),
        };
        if !delim.is_empty() {
            out.push((delim.to_string(), strip));
        }
        i += 2;
    }
    out
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
    fn test_heredoc_body_is_not_maskable_and_ends_at_delimiter() {
        assert_eq!(m("cat <<EOF\nx\nEOF\ny"), vec![true, false, false, true]);
        assert_eq!(m("cat <<'EOF'\nx\nEOF\ny"), vec![true, false, false, true]);
        assert_eq!(m("cat <<-EOF\nx\n\tEOF\ny"), vec![true, false, false, true]);
        assert_eq!(m("a <<A <<B\n1\nA\n2\nB\ny"), vec![true, false, false, false, false, true]);
    }

    #[test]
    fn test_here_string_has_no_body() {
        assert_eq!(m("cat <<< x\ny"), vec![true, true]);
    }

    #[test]
    fn test_unterminated_heredoc_masks_nothing_after() {
        assert_eq!(m("cat <<EOF\nx\ny"), vec![true, false, false]);
    }

    #[test]
    fn test_multiline_double_quote() {
        assert_eq!(m("a=\"\nx\n\"\ny"), vec![false, false, false, true]);
        assert_eq!(m("a=\"x\\\"\ny\"\nz"), vec![false, false, true]);
    }

    #[test]
    fn test_multiline_single_quote() {
        assert_eq!(m("a='\nx\n'\ny"), vec![false, false, false, true]);
    }

    #[test]
    fn test_uncertain_state_is_sticky() {
        assert_eq!(m("a=\"$(\nx\n)\"\ny"), vec![false, false, false, false]);
        assert_eq!(m("a=`\nx\ny"), vec![false, false, false]);
    }

    #[test]
    fn test_comment_does_not_open_a_heredoc() {
        assert_eq!(m("# cat <<EOF\ny"), vec![true, true]);
    }
}
