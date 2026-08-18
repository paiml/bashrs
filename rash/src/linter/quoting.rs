//! Quote-region analysis, shared by every lint rule.
//!
//! GH-226: the shell-syntax rules see a flat stream of physical lines with no
//! notion of where a string literal begins and ends, so they match against the
//! *contents* of quoted strings. Any script containing a regex — which is most
//! non-trivial shell — gets spurious findings, and several of them are
//! `Severity::Error`, so they fail gates rather than merely reporting:
//!
//! ```sh
//! export PATTERN="PMAT-[0-9]{4}"   # SC1020: "missing space before closing ]"
//! grep "^Diff in" file.txt         # SC1035: "missing space after 'in' keyword"
//! ```
//!
//! Neither `]` nor `in` is shell syntax there. This module resolves quoting
//! once, up front, so those rules can be applied to *code* only.
//!
//! ## What counts as literal
//!
//! - `'...'` — the text between the quotes; no escapes inside. The quote
//!   characters themselves are boundaries, not content: rules that tokenise on
//!   them (here-strings, `SC1044`) must keep seeing them.
//! - `"..."` — literal, **except** `$(...)`, `${...}` and backticks, which are
//!   code and stay lintable. This matters: in
//!   `echo "[$(date '+%H:%M')] started"` the brackets and the `+%H:%M` are text
//!   while `date` is a real command.
//! - heredoc bodies — data, not shell syntax. (Quoted-delimiter bodies are
//!   already dropped wholesale by [`crate::linter::heredoc`]; marking them here
//!   too is harmless. Unquoted bodies undergo expansion and are deliberately
//!   still linted by GH-217, but they contain no *syntax* for the rules in
//!   [`QUOTE_SENSITIVE_RULES`] to find.)
//! - trailing comments — `cmd # [0-9]` is a note, not a test expression.
//!
//! ## What does not
//!
//! Backslash escapes outside quotes, and the expansions listed above, are code.
//!
//! Applied once where diagnostics are aggregated, as with the heredoc and
//! embedded-program filters — a per-rule fix cannot generalise, and the next
//! line-oriented rule would reintroduce the bug.

/// A scanner context. The stack's top decides whether a byte is literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctx {
    /// Inside `'...'`.
    Single,
    /// Inside `"..."`.
    Double,
    /// Inside `` `...` `` — code.
    Backtick,
    /// Inside `$( ... )` or a nested `( ... )` — code.
    Paren,
    /// Inside `${ ... }` — code.
    Brace,
    /// Inside `$(( ... ))` — code, and `<<` there is a left shift, not a heredoc.
    Arith,
}

/// A pending or in-progress heredoc body.
#[derive(Debug, Clone)]
struct Heredoc {
    delim: String,
    /// `<<-` strips leading tabs from the terminator.
    strip_tabs: bool,
}

/// Column ranges, per line, that are inside a string literal.
///
/// Columns are 1-indexed byte offsets within the line, matching the spans the
/// rules produce.
#[derive(Debug, Default, Clone)]
pub struct QuotedRegions {
    /// Indexed by `line - 1`; each entry is a sorted list of inclusive
    /// `(start_col, end_col)` ranges.
    per_line: Vec<Vec<(usize, usize)>>,
    any: bool,
}

impl QuotedRegions {
    /// Resolve the quoting of an entire script in one pass.
    pub fn analyze(source: &str) -> Self {
        let mut scanner = Scanner::default();
        let mut per_line = Vec::new();
        let mut any = false;

        for line in source.lines() {
            let ranges = coalesce(&scanner.scan_line(line));
            any |= !ranges.is_empty();
            per_line.push(ranges);
        }

        Self { per_line, any }
    }

    /// Is the 1-indexed `(line, col)` position inside a string literal?
    pub fn is_literal(&self, line: usize, col: usize) -> bool {
        let Some(ranges) = line.checked_sub(1).and_then(|i| self.per_line.get(i)) else {
            return false;
        };
        ranges.iter().any(|&(s, e)| col >= s && col <= e)
    }

    /// True when no part of the source is a string literal — lets callers skip
    /// the filter entirely.
    pub fn is_empty(&self) -> bool {
        !self.any
    }
}

/// Rules whose subject is shell *syntax*, and which are therefore meaningless
/// inside a string literal.
///
/// Deliberately an allowlist, not "all SC1xxx": rules that are *about* quoting
/// (SC1003 escapes in single quotes, SC1078 unterminated quote, SC1079, SC1117,
/// SC2016, SC2086, SC2089/SC2090 …) must keep seeing literals or they go blind,
/// which would trade a false positive for a false negative.
pub const QUOTE_SENSITIVE_RULES: &[&str] = &[
    // Test-expression and bracket syntax — `[0-9]` in a regex is not a test.
    "SC1020", // Missing space before closing ]
    "SC1026", // ( ) in [[ ]]
    "SC1140", // Unexpected token after ]
    // Keyword syntax — `in` inside "^Diff in" is a word, not the for/case keyword.
    "SC1035", // Missing space after keyword
    "SC1044", // Unclosed do..done
    "SC1045", // Missing ;; in case
    "SC1046", // Missing fi
    "SC1047", // Missing fi
    "SC1048", // Missing then
    "SC1049", // Missing then
    "SC1050", // Expected then
    "SC1053", // ;; in wrong place
    "SC1058", // Expected do
    "SC1061", // Missing done
    "SC1062", // Expected done
    // Function/parameter syntax — `function(a, b)` inside an awk or SQL string.
    "SC1064", // Expected { after function
    "SC1065", // Function parameters in shell
    "SC1073", // Couldn't parse this
    // Redirection/operator syntax appearing as text.
    "SC1014", // Use `if cmd; then`
    "SC1036", // ( is invalid here
    "SC1037", // Braces required for positionals > 9
    "SC1041", // Expected EOF
    "SC1072", // Unexpected token
];

/// Should a diagnostic from `code` be dropped when it lands inside a literal?
pub fn is_quote_sensitive(code: &str) -> bool {
    QUOTE_SENSITIVE_RULES.contains(&code)
}

/// Rewrite `source` so every string literal becomes inert filler, preserving
/// byte offsets, line structure and every byte of code (GH-226).
///
/// Rules in [`QUOTE_SENSITIVE_RULES`] are run against this instead of the
/// original, so they cannot react to literal content at all — neither by
/// matching it nor by reporting a position derived from it. Filler is `x`:
/// alphanumeric, so it can never be mistaken for an operator, bracket, keyword
/// or quote, and it cannot introduce a finding the original did not have.
pub fn mask_literals(source: &str) -> String {
    let regions = QuotedRegions::analyze(source);
    if regions.is_empty() {
        return source.to_string();
    }

    let mut out: Vec<u8> = Vec::with_capacity(source.len());
    for (idx, line) in source.split('\n').enumerate() {
        if idx > 0 {
            out.push(b'\n');
        }
        mask_line(line, idx + 1, &regions, &mut out);
    }
    // A literal region always begins and ends on an ASCII delimiter, so a
    // multi-byte character is masked whole; the result stays valid UTF-8 and
    // the same length in bytes.
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

fn mask_line(line: &str, line_no: usize, regions: &QuotedRegions, out: &mut Vec<u8>) {
    for (col, byte) in line.bytes().enumerate() {
        if byte != b'\r' && regions.is_literal(line_no, col + 1) {
            out.push(b'x');
        } else {
            out.push(byte);
        }
    }
}

/// Restore original text in messages produced against the masked source.
///
/// A rule that quotes the offending token (SC1140 does) would otherwise print
/// filler. The diagnostic's own span identifies the text exactly, so the
/// substitution is not a guess.
pub fn restore_masked_messages(source: &str, masked: &str, result: &mut crate::linter::LintResult) {
    for diag in result.diagnostics.iter_mut() {
        if !is_quote_sensitive(&diag.code) {
            continue;
        }
        let (Some(from), Some(to)) = (
            span_text(masked, diag.span.start_line, diag.span.start_col, diag.span.end_col),
            span_text(source, diag.span.start_line, diag.span.start_col, diag.span.end_col),
        ) else {
            continue;
        };
        if from != to && !from.is_empty() && diag.message.contains(&from) {
            diag.message = diag.message.replace(&from, &to);
        }
    }
}

/// The bytes of a 1-indexed line between 1-indexed `[start, end)` columns.
fn span_text(source: &str, line: usize, start: usize, end: usize) -> Option<String> {
    if start == 0 || end <= start {
        return None;
    }
    let text = source.lines().nth(line.checked_sub(1)?)?;
    let bytes = text.as_bytes();
    let hi = end.saturating_sub(1).min(bytes.len());
    let lo = start.saturating_sub(1).min(hi);
    Some(String::from_utf8_lossy(&bytes[lo..hi]).into_owned())
}

/// Coalesce a per-byte literal mask into inclusive 1-indexed column ranges.
fn coalesce(marks: &[bool]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;

    for (i, &m) in marks.iter().enumerate() {
        match (m, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                out.push((s + 1, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push((s + 1, marks.len()));
    }
    out
}

/// A character scanner that carries quoting state across lines.
#[derive(Debug, Default)]
struct Scanner {
    stack: Vec<Ctx>,
    /// Heredocs opened on this line, awaiting their bodies, in POSIX order.
    pending: std::collections::VecDeque<Heredoc>,
    /// The body currently being consumed.
    body: Option<Heredoc>,
}

impl Scanner {
    /// Scan one physical line, returning a per-byte "is literal" mask.
    fn scan_line(&mut self, line: &str) -> Vec<bool> {
        let bytes = line.as_bytes();
        let mut marks = vec![false; bytes.len()];

        if self.consume_heredoc_body(line, &mut marks) {
            return marks;
        }

        let mut i = 0;
        while i < bytes.len() {
            i = match self.stack.last().copied() {
                Some(Ctx::Single) => self.step_single(bytes, i, &mut marks),
                Some(Ctx::Double) => self.step_double(bytes, i, &mut marks),
                _ => self.step_code(bytes, i, &mut marks),
            };
        }

        if self.body.is_none() {
            self.body = self.pending.pop_front();
        }
        marks
    }

    /// If a heredoc body is open, mark the whole line and check for its
    /// terminator. Returns true when the line was consumed as body.
    fn consume_heredoc_body(&mut self, line: &str, marks: &mut [bool]) -> bool {
        let Some(doc) = self.body.clone() else {
            return false;
        };

        let candidate = if doc.strip_tabs {
            line.trim_start_matches('\t')
        } else {
            line
        };
        if candidate.trim_end() == doc.delim {
            self.body = self.pending.pop_front();
            return true;
        }

        marks.iter_mut().for_each(|m| *m = true);
        true
    }

    /// Inside `'...'`: everything is literal, the next `'` closes.
    fn step_single(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        if bytes[i] == b'\'' {
            self.stack.pop();
        } else {
            marks[i] = true;
        }
        i + 1
    }

    /// Inside `"..."`: literal, except the expansions that re-enter code.
    fn step_double(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        match bytes[i] {
            b'\\' => {
                marks[i] = true;
                if let Some(m) = marks.get_mut(i + 1) {
                    *m = true;
                }
                i + 2
            }
            b'"' => {
                self.stack.pop();
                i + 1
            }
            b'`' => {
                self.stack.push(Ctx::Backtick);
                i + 1
            }
            b'$' => self.open_expansion(bytes, i, marks),
            _ => {
                marks[i] = true;
                i + 1
            }
        }
    }

    /// Any code context: top level, `$( )`, `${ }`, backticks, `$(( ))`.
    fn step_code(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        match bytes[i] {
            b'\\' => i + 2,
            b'\'' => {
                self.stack.push(Ctx::Single);
                i + 1
            }
            b'"' => {
                self.stack.push(Ctx::Double);
                i + 1
            }
            b'`' => {
                self.toggle_backtick();
                i + 1
            }
            b'$' => self.open_expansion(bytes, i, marks),
            b'#' => self.maybe_comment(bytes, i, marks),
            b'(' => self.open_paren(i),
            b')' => self.close_paren(bytes, i),
            b'}' => {
                self.pop_if(Ctx::Brace);
                i + 1
            }
            b'<' => self.maybe_heredoc(bytes, i),
            _ => i + 1,
        }
    }

    /// `$(`, `$((` and `${` open code regions; a bare `$` is just text.
    fn open_expansion(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        match (bytes.get(i + 1), bytes.get(i + 2)) {
            (Some(b'('), Some(b'(')) => {
                self.stack.push(Ctx::Arith);
                i + 3
            }
            (Some(b'('), _) => {
                self.stack.push(Ctx::Paren);
                i + 2
            }
            (Some(b'{'), _) => {
                self.stack.push(Ctx::Brace);
                i + 2
            }
            _ => {
                // `$VAR`, `$1`, `$@` … are expansions: code, not text. Leaving
                // them visible keeps rules that tokenise on them (here-strings,
                // SC1044) seeing what they saw before.
                if let Some(next) = end_of_simple_expansion(bytes, i) {
                    return next;
                }
                if self.stack.last() == Some(&Ctx::Double) {
                    marks[i] = true;
                }
                i + 1
            }
        }
    }

    fn toggle_backtick(&mut self) {
        if self.stack.last() == Some(&Ctx::Backtick) {
            self.stack.pop();
        } else {
            self.stack.push(Ctx::Backtick);
        }
    }

    /// A `(` only nests when we are already inside one; a stray `(` at top
    /// level (a subshell, or `((` arithmetic) must not unbalance the stack.
    fn open_paren(&mut self, i: usize) -> usize {
        if matches!(self.stack.last(), Some(Ctx::Paren | Ctx::Arith)) {
            self.stack.push(Ctx::Paren);
        }
        i + 1
    }

    fn close_paren(&mut self, bytes: &[u8], i: usize) -> usize {
        if self.stack.last() == Some(&Ctx::Arith) {
            if bytes.get(i + 1) == Some(&b')') {
                self.stack.pop();
                return i + 2;
            }
            return i + 1;
        }
        self.pop_if(Ctx::Paren);
        i + 1
    }

    fn pop_if(&mut self, ctx: Ctx) {
        if self.stack.last() == Some(&ctx) {
            self.stack.pop();
        }
    }

    /// `#` starts a comment only at the start of a word — `${#v}` and `a#b` do
    /// not. The rest of the line is then text.
    fn maybe_comment(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        let starts_word = match i.checked_sub(1).map(|p| bytes[p]) {
            None => true,
            Some(b) => b.is_ascii_whitespace() || matches!(b, b';' | b'&' | b'|' | b'('),
        };
        if !starts_word {
            return i + 1;
        }
        marks[i..].iter_mut().for_each(|m| *m = true);
        bytes.len()
    }

    /// Register a heredoc opener. `<<<` is a here-string (no body), and inside
    /// `$(( ))` a `<<` is a left shift.
    fn maybe_heredoc(&mut self, bytes: &[u8], i: usize) -> usize {
        if bytes.get(i + 1) != Some(&b'<') {
            return i + 1;
        }
        if bytes.get(i + 2) == Some(&b'<') {
            return i + 3;
        }
        if self.stack.last() == Some(&Ctx::Arith) {
            return i + 2;
        }

        let (doc, next) = parse_heredoc_opener(bytes, i + 2);
        if let Some(doc) = doc {
            self.pending.push_back(doc);
        }
        next
    }
}

/// The index just past a `$NAME`, `$1` or `$@`-style expansion at `i`, or
/// `None` when `$` is just a dollar sign.
fn end_of_simple_expansion(bytes: &[u8], i: usize) -> Option<usize> {
    let first = *bytes.get(i + 1)?;
    if matches!(first, b'@' | b'*' | b'#' | b'?' | b'!' | b'$' | b'-') {
        return Some(i + 2);
    }
    if !(first.is_ascii_alphanumeric() || first == b'_') {
        return None;
    }
    let mut end = i + 1;
    while bytes
        .get(end)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        end += 1;
    }
    Some(end)
}

/// Parse `[-][ ]DELIM` after a `<<`, returning the heredoc and the index past
/// the delimiter.
fn parse_heredoc_opener(bytes: &[u8], start: usize) -> (Option<Heredoc>, usize) {
    let mut i = start;
    let strip_tabs = bytes.get(i) == Some(&b'-');
    if strip_tabs {
        i += 1;
    }
    while bytes.get(i).is_some_and(|b| *b == b' ' || *b == b'\t') {
        i += 1;
    }

    match bytes.get(i) {
        Some(&q @ (b'\'' | b'"')) => parse_quoted_delim(bytes, i, q, strip_tabs),
        Some(b) if b.is_ascii_alphabetic() || *b == b'_' => {
            parse_bare_delim(bytes, i, strip_tabs)
        }
        // A numeric or operator "delimiter" is a left shift, not a heredoc.
        _ => (None, i.max(start + 1)),
    }
}

fn parse_quoted_delim(
    bytes: &[u8],
    open: usize,
    quote: u8,
    strip_tabs: bool,
) -> (Option<Heredoc>, usize) {
    let mut end = open + 1;
    while end < bytes.len() && bytes[end] != quote {
        end += 1;
    }
    if end >= bytes.len() {
        return (None, bytes.len());
    }
    let delim = String::from_utf8_lossy(&bytes[open + 1..end]).into_owned();
    let doc = (!delim.is_empty()).then_some(Heredoc { delim, strip_tabs });
    (doc, end + 1)
}

fn parse_bare_delim(bytes: &[u8], start: usize, strip_tabs: bool) -> (Option<Heredoc>, usize) {
    let mut end = start;
    while bytes
        .get(end)
        .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
    {
        end += 1;
    }
    let delim = String::from_utf8_lossy(&bytes[start..end]).into_owned();
    (Some(Heredoc { delim, strip_tabs }), end)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The literal text of a line, as the scanner sees it — `.` marks code.
    fn mask(source: &str) -> Vec<String> {
        let regions = QuotedRegions::analyze(source);
        source
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                line.bytes()
                    .enumerate()
                    .map(|(col, b)| {
                        if regions.is_literal(idx + 1, col + 1) {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_GH226_quoting_single_quotes_are_literal() {
        assert_eq!(mask("echo 'abc'"), vec![".....'abc'".replace('\'', ".")]);
    }

    #[test]
    fn test_GH226_quoting_double_quotes_are_literal() {
        assert_eq!(mask(r#"echo "abc""#), vec!["......abc."]);
    }

    #[test]
    fn test_GH226_quoting_regex_class_in_double_quotes() {
        // The reproduction from the issue: `[0-9]{4}` is regex, not a test.
        let m = mask(r#"export PATTERN="PMAT-[0-9]{4}""#);
        assert_eq!(m[0], "................PMAT-[0-9]{4}.");
    }

    #[test]
    fn test_GH226_quoting_command_substitution_inside_double_quotes_is_code() {
        // `date` must stay lintable; the brackets around it must not.
        let m = mask(r#"echo "[$(date '+%H:%M')] started""#);
        assert_eq!(m[0], "......[........+%H:%M.._ started.".replace('_', "]"));
    }

    #[test]
    fn test_GH226_quoting_nested_quotes_in_command_substitution() {
        // Nothing here is literal *text*: the quotes are boundaries, `curl` is
        // code, and `$url` is an expansion.
        let m = mask(r#"out="$(curl -sSfL "$url")""#);
        assert_eq!(m[0], "..........................");
    }

    #[test]
    fn test_GH226_quoting_expansion_in_quotes_is_not_masked() {
        // GH-226 regression: masking `$HOSTS` made `read -a X <<< "$HOSTS"`
        // look like an unclosed heredoc to SC1044.
        let m = mask(r#"read -r -a HOST_ARR <<< "$HOSTS""#);
        assert_eq!(m[0], "................................");
        let m = mask(r#"echo "prefix ${x[0]} $1 $@ suffix""#);
        // `${x[0]}`, `$1` and `$@` show as code; only the plain words are text.
        assert_eq!(m[0], "......prefix ....... .. .. suffix.");
    }

    #[test]
    fn test_GH226_quoting_escape_outside_quotes_is_code() {
        let m = mask(r#"echo \" hi"#);
        assert_eq!(m[0], "..........");
    }

    #[test]
    fn test_GH226_quoting_escaped_quote_inside_double_quotes() {
        let m = mask(r#"echo "a\"b""#);
        assert_eq!(m[0], r#"......a\"b."#);
    }

    #[test]
    fn test_GH226_quoting_multiline_single_quote() {
        let m = mask("echo 'a\nb' done");
        assert_eq!(m[0], "......a");
        assert_eq!(m[1], "b......");
    }

    #[test]
    fn test_GH226_quoting_trailing_comment_is_literal() {
        let m = mask("cmd arg  # [0-9]");
        assert_eq!(m[0], ".........# [0-9]");
    }

    #[test]
    fn test_GH226_quoting_hash_in_parameter_expansion_is_not_a_comment() {
        let m = mask("echo ${#arr}");
        assert_eq!(m[0], "............");
    }

    #[test]
    fn test_GH226_quoting_heredoc_body_is_literal() {
        let src = "cat <<EOF\nnot [shell] syntax\nEOF\necho ok";
        let m = mask(src);
        assert_eq!(m[0], ".........");
        assert_eq!(m[1], "not [shell] syntax");
        assert_eq!(m[2], "...");
        assert_eq!(m[3], ".......");
    }

    #[test]
    fn test_GH226_quoting_heredoc_apostrophe_does_not_leak() {
        // An apostrophe in a heredoc body must not open a quote for the rest
        // of the file.
        let src = "cat <<EOF\ndon't panic\nEOF\necho 'x'";
        let m = mask(src);
        assert_eq!(m[3], "......x.");
    }

    #[test]
    fn test_GH226_quoting_here_string_has_no_body() {
        let src = "cmd <<< 'word'\necho ok";
        let m = mask(src);
        assert_eq!(m[0], ".........word.");
        assert_eq!(m[1], ".......");
    }

    #[test]
    fn test_GH226_quoting_arith_left_shift_is_not_a_heredoc() {
        let src = "x=$(( 1 << n ))\necho ok";
        let m = mask(src);
        assert_eq!(m[1], ".......", "the rest of the file must not be a heredoc body");
    }

    #[test]
    fn test_GH226_quoting_backtick_substitution_is_code() {
        let m = mask("x=`echo 'a'`");
        assert_eq!(m[0], ".........a..");
    }

    #[test]
    fn test_GH226_quoting_is_empty_for_pure_code() {
        assert!(QuotedRegions::analyze("mkdir -p /tmp/x\nexit 0").is_empty());
        assert!(!QuotedRegions::analyze("echo 'x'").is_empty());
    }

    #[test]
    fn test_GH226_quoting_out_of_range_positions_are_not_literal() {
        let r = QuotedRegions::analyze("echo 'x'");
        assert!(!r.is_literal(0, 1), "line 0 does not exist (1-indexed)");
        assert!(!r.is_literal(99, 1));
        assert!(!r.is_literal(1, 999));
    }

    #[test]
    fn test_GH226_quoting_allowlist_excludes_quote_rules() {
        // Rules whose subject IS the literal must keep seeing it.
        for code in ["SC1003", "SC1078", "SC1079", "SC1117", "SC2016", "SC2086"] {
            assert!(
                !is_quote_sensitive(code),
                "{code} inspects quoting and must not be filtered"
            );
        }
        for code in ["SC1020", "SC1035", "SC1140"] {
            assert!(is_quote_sensitive(code), "{code} is shell syntax");
        }
    }

    #[test]
    fn test_GH226_quoting_unterminated_quote_does_not_panic() {
        for src in ["echo 'unterminated", "echo \"unterminated", "x=$(", "x=${", "cat <<"] {
            let _ = QuotedRegions::analyze(src);
        }
    }
}
