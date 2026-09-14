//! Single-quoted segments of one shell line, found per WORD (bashrs#335).
//!
//! SC2081 and SC2016 matched `'[^']*\$…[^']*'` against the raw line. A regex has
//! no quoting state, so on `f 'a' $x 'b'` it paired the CLOSING quote of `'a'`
//! with the OPENING quote of `'b'` and reported ` $x ` as single-quoted text --
//! and SC2081's autofix then rewrote that span, merging four arguments into one
//! (7.3.0 and published 7.4.0).
//!
//! Words from [`simple_commands`] are delimited by UNQUOTED blanks, by a lexer
//! that does track quoting, so pairing quotes inside one word's `raw` cannot cross
//! a string boundary. That is the fix; the rest of this module makes sure a caller
//! is never handed a span that does not name the bytes it claims.

use super::shell_words::simple_commands;

/// One closed single-quoted segment, exactly as written in the physical line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleQuoted {
    /// 1-indexed byte column of the opening `'`.
    pub start_col: usize,
    /// 1-indexed byte column one past the closing `'` -- the exclusive end that
    /// `Span` and the autofix splicer use.
    pub end_col: usize,
    /// The text between the quotes.
    pub content: String,
}

/// Every closed single-quoted segment on `line`, in column order, each once.
///
/// A segment is returned only when the bytes of `line` at its columns really are
/// `'content'`. A word re-lexed from a `sh -c "…"` operand has columns derived
/// from that operand's LITERAL text, which escape bytes can shift away from the
/// physical line; checking the line itself means no caller receives a span that
/// would splice the wrong bytes.
///
/// A word inside `$( … )` is visible twice -- in the enclosing word, and in the
/// inner command [`simple_commands`] recurses into -- so segments are deduplicated
/// by span.
pub fn single_quoted_segments(line: &str) -> Vec<SingleQuoted> {
    let spans = simple_commands(line)
        .into_iter()
        .flat_map(|cmd| cmd.words)
        .flat_map(|word| {
            let col = word.col;
            pair_single_quotes(&word.raw)
                .into_iter()
                .map(move |(open, close)| (col + open, col + close + 1))
        });
    let mut out: Vec<SingleQuoted> = Vec::new();
    for (start_col, end_col) in spans {
        let Some(seg) = segment_at(line, start_col, end_col) else {
            continue;
        };
        if !out
            .iter()
            .any(|s| s.start_col == start_col && s.end_col == end_col)
        {
            out.push(seg);
        }
    }
    out.sort_by_key(|s| s.start_col);
    out
}

/// The segment at these columns, if the bytes of `line` there really are
/// `'content'`.
fn segment_at(line: &str, start_col: usize, end_col: usize) -> Option<SingleQuoted> {
    let text = line.get(start_col.saturating_sub(1)..end_col.saturating_sub(1))?;
    let content = text.strip_prefix('\'')?.strip_suffix('\'')?;
    Some(SingleQuoted {
        start_col,
        end_col,
        content: content.to_string(),
    })
}

/// Byte offsets `(open, close)` of the plain single-quoted segments in one word.
///
/// Outside quotes and inside double quotes a backslash consumes the next byte
/// (inside double quotes that can over-consume a literal, never a quote that
/// matters here). Inside single quotes nothing is special until the next `'`.
/// `$'…'` is ANSI-C quoting, where `\'` IS an escape: it is walked with escapes
/// and not reported, because it is not the string these rules mean. An
/// unterminated `'` ends the scan -- it is not a segment.
fn pair_single_quotes(raw: &str) -> Vec<(usize, usize)> {
    let b = raw.as_bytes();
    let mut out = Vec::new();
    let (mut i, mut in_double, mut dollar) = (0usize, false, false);
    while let Some(&c) = b.get(i) {
        match c {
            b'\\' => {
                dollar = false;
                i += 2;
            }
            b'"' => {
                in_double = !in_double;
                dollar = false;
                i += 1;
            }
            b'\'' if !in_double => {
                let Some(close) = closing_quote(b, i + 1, dollar) else {
                    break;
                };
                if !dollar {
                    out.push((i, close));
                }
                dollar = false;
                i = close + 1;
            }
            _ => {
                dollar = c == b'$';
                i += 1;
            }
        }
    }
    out
}

/// Offset of the `'` that closes a segment whose content starts at `from`.
fn closing_quote(b: &[u8], from: usize, ansi_c: bool) -> Option<usize> {
    let mut k = from;
    while let Some(&c) = b.get(k) {
        match c {
            b'\\' if ansi_c => k += 2,
            b'\'' => return Some(k),
            _ => k += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contents(line: &str) -> Vec<String> {
        single_quoted_segments(line)
            .into_iter()
            .map(|s| s.content)
            .collect()
    }

    #[test]
    fn test_QS_335_pairs_quotes_inside_words_not_across_the_line() {
        let line = r#"assert_row 'append one entry' PASS "$TD/append.yaml" 'added=1'"#;
        assert_eq!(contents(line), vec!["append one entry", "added=1"]);
    }

    #[test]
    fn test_QS_335_span_names_the_bytes_it_claims() {
        let line = r#"trap 'rm -rf -- "${TD:?}"' EXIT"#;
        let segs = single_quoted_segments(line);
        assert_eq!(segs.len(), 1);
        assert_eq!(
            &line[segs[0].start_col - 1..segs[0].end_col - 1],
            r#"'rm -rf -- "${TD:?}"'"#
        );
    }

    #[test]
    fn test_QS_335_single_quote_inside_double_quotes_is_literal() {
        assert!(contents(r#"echo "it's $x""#).is_empty());
    }

    #[test]
    fn test_QS_335_ansi_c_quoting_is_not_a_plain_segment() {
        assert!(contents(r#"printf $'it\'s $x'"#).is_empty());
    }

    #[test]
    fn test_QS_335_command_substitution_is_reported_once() {
        assert_eq!(contents(r#"v=$(echo 'a $b')"#), vec!["a $b"]);
    }

    #[test]
    fn test_QS_335_unterminated_quote_yields_nothing() {
        assert!(contents("echo 'never closed $x").is_empty());
    }
}
