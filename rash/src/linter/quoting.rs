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
    /// Inside `$(( ... ))` or a bare `(( ... ))` — code, and `<<` there is a
    /// left shift, not a heredoc.
    Arith,
    /// Inside `$'...'` — literal, but `\'` is an escape, unlike `'...'`.
    Ansi,
}

/// A pending or in-progress heredoc body.
#[derive(Debug, Clone)]
struct Heredoc {
    delim: String,
    /// `<<-` strips leading tabs from the terminator.
    strip_tabs: bool,
    /// `<<'X'` / `<<"X"` — the body is literal text with no expansion.
    quoted: bool,
    /// 1-indexed line the body starts on, for the unterminated fail-safe.
    body_start: usize,
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
    /// 1-indexed lines inside a body whose delimiter was quoted. Those bodies
    /// are literal text by definition, so every rule is dropped there — see
    /// [`quoted_heredoc_lines`].
    quoted_heredoc: std::collections::HashSet<usize>,
}

impl QuotedRegions {
    /// Resolve the quoting of an entire script in one pass.
    pub fn analyze(source: &str) -> Self {
        let mut scanner = Scanner::default();
        let mut per_line = Vec::new();
        let mut any = false;

        for (idx, line) in source.lines().enumerate() {
            scanner.line_no = idx + 1;
            let ranges = coalesce(&scanner.scan_line(line));
            any |= !ranges.is_empty();
            per_line.push(ranges);
        }

        // Fail-safe: a region that never closes means everything after it was
        // masked on a guess. Discard that part of the mask rather than let the
        // rules go blind for the rest of the file — an unterminated quote is
        // SC1078's finding and an unterminated heredoc is SC1044's, and neither
        // is in the allowlist.
        let mut discard_at = scanner.quote_open_at;
        if let Some(open) = scanner.body.as_ref() {
            // A heredoc still open at EOF never had a terminator. Its body was
            // a guess, so neither mask it nor report it as a heredoc body.
            let from = (open.body_start, 1);
            discard_at = Some(min_position(discard_at, from));
            scanner
                .quoted_heredoc
                .retain(|line| *line < open.body_start);
        }
        if let Some((line, col)) = discard_at {
            discard_from(&mut per_line, line, col);
            any = per_line.iter().any(|ranges| !ranges.is_empty());
        }

        Self {
            per_line,
            any,
            quoted_heredoc: scanner.quoted_heredoc,
        }
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

    /// 1-indexed lines inside a body whose heredoc delimiter was quoted.
    pub fn quoted_heredoc_lines(&self) -> &std::collections::HashSet<usize> {
        &self.quoted_heredoc
    }

    /// The literal ranges on a 1-indexed line, in ascending order.
    fn ranges_for(&self, line: usize) -> &[(usize, usize)] {
        line.checked_sub(1)
            .and_then(|i| self.per_line.get(i))
            .map_or(&[], Vec::as_slice)
    }
}

/// The earlier of two optional 1-indexed `(line, col)` positions.
fn min_position(a: Option<(usize, usize)>, b: (usize, usize)) -> (usize, usize) {
    match a {
        Some(a) if a <= b => a,
        _ => b,
    }
}

/// 1-indexed lines inside a body whose heredoc delimiter was quoted (GH-217).
///
/// A quoted-delimiter body is literal text by definition — that is what quoting
/// the delimiter means — so no shell rule should analyse it.
///
/// This resolves quoting first. The previous implementation scanned raw bytes
/// for `<<` with no notion of comments or strings, so a line of documentation
/// mentioning `<<'PY'` opened a body that never closed and silenced EVERY rule
/// for the rest of the file. An unterminated heredoc does not silence anything
/// either: see the fail-safe in [`QuotedRegions::analyze`].
pub fn quoted_heredoc_lines(source: &str) -> std::collections::HashSet<usize> {
    QuotedRegions::analyze(source).quoted_heredoc
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
    // Function/parameter syntax — `function(a, b)` inside an awk or SQL program.
    "SC1065", // Function parameters in shell
    // Redirection/operator syntax appearing as text.
    "SC1007", // Remove space after = — `skip = 0` inside an awk program is awk
    "SC1014", // Use `if cmd; then`
    "SC1036", // ( is invalid here
    "SC1037", // Braces required for positionals > 9
    "SC1041", // Expected EOF
    // Characters that are a typo in code and ordinary text in a message.
    "SC1100", // Unicode dash — an em-dash in `echo "gate FAILED — fix it"` is prose
    // Paren and bracket syntax that is ordinary punctuation in a message.
    "SC1028", // Bare ( ) in a test — `log "waiting (${n}s elapsed)"` is prose
    "SC2104", // Missing space before ] — `"usage: rag [--source|--videos]"` is prose
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
    // Walk the line's ranges with a cursor rather than calling `is_literal` per
    // byte: that was a linear scan of the range list for every byte, so a long
    // single line with many literals went quadratic (a 400 KB one-line script
    // took 15s).
    let ranges = regions.ranges_for(line_no);
    let mut next = 0;

    for (col, byte) in line.bytes().enumerate() {
        let col = col + 1;
        while next < ranges.len() && ranges[next].1 < col {
            next += 1;
        }
        let literal = ranges
            .get(next)
            .is_some_and(|&(start, end)| col >= start && col <= end);
        out.push(if literal && byte != b'\r' { b'x' } else { byte });
    }
}

/// Restore original text in messages produced against the masked source.
///
/// A rule that quotes the offending token (SC1140 does) would otherwise print
/// filler. The diagnostic's own span identifies the text exactly, so the
/// substitution is not a guess.
pub fn restore_masked_messages(source: &str, masked: &str, result: &mut crate::linter::LintResult) {
    if result.diagnostics.is_empty() {
        return;
    }
    // Index the lines once. `lines().nth(n)` per diagnostic is quadratic on a
    // large file with many findings.
    let src_lines: Vec<&str> = source.lines().collect();
    let masked_lines: Vec<&str> = masked.lines().collect();

    for diag in result.diagnostics.iter_mut() {
        if !is_quote_sensitive(&diag.code) {
            continue;
        }
        let (line, lo, hi) = (diag.span.start_line, diag.span.start_col, diag.span.end_col);
        let (Some(from), Some(to)) = (
            span_text(&masked_lines, line, lo, hi),
            span_text(&src_lines, line, lo, hi),
        ) else {
            continue;
        };
        if from != to && !from.is_empty() && diag.message.contains(&from) {
            diag.message = diag.message.replace(&from, &to);
        }
    }
}

/// The bytes of a 1-indexed line between 1-indexed `[start, end)` columns.
fn span_text(lines: &[&str], line: usize, start: usize, end: usize) -> Option<String> {
    if start == 0 || end <= start {
        return None;
    }
    let bytes = lines.get(line.checked_sub(1)?)?.as_bytes();
    let hi = end.saturating_sub(1).min(bytes.len());
    let lo = start.saturating_sub(1).min(hi);
    Some(String::from_utf8_lossy(&bytes[lo..hi]).into_owned())
}

/// Drop every marked range at or after 1-indexed `(line, col)`.
fn discard_from(per_line: &mut [Vec<(usize, usize)>], line: usize, col: usize) {
    for (idx, ranges) in per_line.iter_mut().enumerate() {
        let ln = idx + 1;
        if ln > line {
            ranges.clear();
        } else if ln == line {
            ranges.retain_mut(|range| {
                if range.0 >= col {
                    return false;
                }
                range.1 = range.1.min(col - 1);
                true
            });
        }
    }
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
    /// 1-indexed lines inside a quoted-delimiter heredoc body.
    quoted_heredoc: std::collections::HashSet<usize>,
    /// 1-indexed line currently being scanned.
    line_no: usize,
    /// Where the outermost currently-open quote started, for the EOF fail-safe.
    quote_open_at: Option<(usize, usize)>,
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
                Some(Ctx::Ansi) => self.step_ansi(bytes, i, &mut marks),
                Some(Ctx::Double) => self.step_double(bytes, i, &mut marks),
                _ => self.step_code(bytes, i, &mut marks),
            };
        }

        if self.body.is_none() {
            self.body = self.take_pending();
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
            self.body = self.take_pending();
            return true;
        }

        if doc.quoted {
            self.quoted_heredoc.insert(self.line_no);
        }
        marks.iter_mut().for_each(|m| *m = true);
        true
    }

    /// The next pending heredoc, with its body starting on the next line.
    fn take_pending(&mut self) -> Option<Heredoc> {
        let mut doc = self.pending.pop_front()?;
        doc.body_start = self.line_no + 1;
        Some(doc)
    }

    /// Inside `'...'`: everything is literal, the next `'` closes.
    fn step_single(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        if bytes[i] == b'\'' {
            self.pop_quote();
        } else {
            marks[i] = true;
        }
        i + 1
    }

    /// Inside `$'...'`: literal, but backslash escapes — so `$'don\'t'` is one
    /// word, not two. Treating it as a plain `'...'` flipped quote parity for
    /// the rest of the file.
    fn step_ansi(&mut self, bytes: &[u8], i: usize, marks: &mut [bool]) -> usize {
        match bytes[i] {
            b'\\' => {
                marks[i] = true;
                if let Some(m) = marks.get_mut(i + 1) {
                    *m = true;
                }
                i + 2
            }
            b'\'' => {
                self.pop_quote();
                i + 1
            }
            _ => {
                marks[i] = true;
                i + 1
            }
        }
    }

    /// Remember where the OUTERMOST open quote began, so the EOF fail-safe can
    /// discard a mask built on an unterminated one.
    fn push_quote(&mut self, ctx: Ctx, i: usize) {
        if self.quote_open_at.is_none() {
            self.quote_open_at = Some((self.line_no, i + 1));
        }
        self.stack.push(ctx);
    }

    fn pop_quote(&mut self) {
        self.stack.pop();
        if !self
            .stack
            .iter()
            .any(|c| matches!(c, Ctx::Single | Ctx::Double | Ctx::Ansi))
        {
            self.quote_open_at = None;
        }
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
                self.pop_quote();
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
                self.push_quote(Ctx::Single, i);
                i + 1
            }
            b'"' => {
                self.push_quote(Ctx::Double, i);
                i + 1
            }
            b'`' => {
                self.toggle_backtick();
                i + 1
            }
            b'$' => self.open_expansion(bytes, i, marks),
            b'#' => self.maybe_comment(bytes, i, marks),
            b'(' => self.open_paren(bytes, i),
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
            // `$'...'` is ANSI-C quoting and `$"..."` is locale translation;
            // both are string literals whose opening delimiter is two bytes.
            (Some(b'\''), _) => {
                self.push_quote(Ctx::Ansi, i);
                i + 2
            }
            (Some(b'"'), _) => {
                self.push_quote(Ctx::Double, i);
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
    fn open_paren(&mut self, bytes: &[u8], i: usize) -> usize {
        // A bare `(( … ))` is bash's arithmetic COMMAND. Without this it left
        // the stack empty, so the `<<` in `(( a << b ))` looked like a heredoc
        // opener and masked the rest of the file as its body.
        if bytes.get(i + 1) == Some(&b'(') {
            self.stack.push(Ctx::Arith);
            return i + 2;
        }
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
        Some(b) if b.is_ascii_alphabetic() || *b == b'_' => parse_bare_delim(bytes, i, strip_tabs),
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
    let doc = (!delim.is_empty()).then_some(Heredoc {
        delim,
        strip_tabs,
        quoted: true,
        body_start: 0,
    });
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
    (
        Some(Heredoc {
            delim,
            strip_tabs,
            quoted: false,
            body_start: 0,
        }),
        end,
    )
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
        assert_eq!(
            m[1], ".......",
            "the rest of the file must not be a heredoc body"
        );
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
    fn test_GH226_quoting_unterminated_quote_does_not_blind_later_lines() {
        // Fail-safe: without it, the unclosed quote on line 2 masks everything
        // after it and SC1020 stops seeing the real defect on line 3.
        let src = "echo start\necho 'unterminated\n[ -f x]\n";
        let regions = QuotedRegions::analyze(src);
        assert!(!regions.is_literal(3, 7), "line 3 must stay lintable");
        assert!(
            !regions.is_literal(2, 8),
            "the guessed region is discarded too"
        );
        assert_eq!(mask_literals(src), src, "nothing may be masked");
    }

    #[test]
    fn test_GH226_quoting_terminated_multiline_quote_is_still_masked() {
        // The fail-safe must not fire when the quote does close.
        let src = "echo 'a\nb'\n[ -f x]\n";
        let regions = QuotedRegions::analyze(src);
        assert!(regions.is_literal(1, 7), "the quoted text is still literal");
        assert!(!regions.is_literal(3, 7));
    }

    /// Every allowlist entry, paired with the rule it names.
    ///
    /// Each right-hand side is a compile-time reference to the rule module, so
    /// an entry naming a rule that does not exist cannot be added — which is
    /// how 11 of the original 22 entries (SC1046..SC1073) went unnoticed.
    type LintCheck = fn(&str) -> crate::linter::LintResult;

    fn allowlisted_checks() -> Vec<(&'static str, LintCheck)> {
        use crate::linter::rules::*;
        vec![
            ("SC1014", sc1014::check),
            ("SC1020", sc1020::check),
            ("SC1026", sc1026::check),
            ("SC1035", sc1035::check),
            ("SC1036", sc1036::check),
            ("SC1037", sc1037::check),
            ("SC1007", sc1007::check),
            ("SC1041", sc1041::check),
            ("SC1100", sc1100::check),
            ("SC1044", sc1044::check),
            ("SC1045", sc1045::check),
            ("SC1065", sc1065::check),
            ("SC1140", sc1140::check),
            ("SC1028", sc1028::check),
            ("SC2104", sc2104::check),
        ]
    }

    #[test]
    fn test_GH226_quoting_allowlist_names_only_rules_that_exist() {
        let known = allowlisted_checks();
        for code in QUOTE_SENSITIVE_RULES {
            assert!(
                known.iter().any(|(c, _)| c == code),
                "{code} is allowlisted but names no rule module"
            );
        }
        assert_eq!(
            known.len(),
            QUOTE_SENSITIVE_RULES.len(),
            "allowlist and module list must stay in step"
        );
    }

    #[test]
    fn test_GH226_quoting_allowlisted_rules_find_nothing_in_a_pure_literal() {
        // The property the allowlist exists for: given a line that is entirely
        // a quoted string, no allowlisted rule may report anything once the
        // literal is masked — whatever shell-looking text the string contains.
        let sources = [
            // No `$10` here: an expansion inside double quotes is real code,
            // and SC1037 is right to report it. Only the *text* is masked.
            r#"echo "if [ x] ; then for i in done ] function(a,b) ( ) fi""#,
            r#"echo 'case x in [0-9]) do done esac ] { } function f(a) [[ ]] $10'"#,
            r#"printf '  x Found [[ ]] (bash-specific) and function keyword
'"#,
        ];
        for src in sources {
            let masked = mask_literals(src);
            for (code, check) in allowlisted_checks() {
                let found = check(&masked);
                assert!(
                    found.diagnostics.is_empty(),
                    "{code} fired inside a string literal: {:?} on {src}",
                    found
                        .diagnostics
                        .iter()
                        .map(|d| &d.message)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    // ---- adversarial review regressions ----

    #[test]
    fn test_GH226_quoting_heredoc_marker_in_a_comment_opens_nothing() {
        // The worst regression the review found: a line of documentation
        // mentioning `<<'PY'` opened a body that never closed, and once the
        // heredoc filter reached the CLI that silenced EVERY rule for the rest
        // of the file.
        let src = "#!/bin/sh\n# embed python with <<'PY' ... PY\neval \"$USER_INPUT\"\n";
        assert!(quoted_heredoc_lines(src).is_empty());
        assert_eq!(mask_literals(src).lines().nth(2), src.lines().nth(2));
    }

    #[test]
    fn test_GH226_quoting_heredoc_marker_in_a_string_opens_nothing() {
        let src = "sed -i \"s/<<'EOF'/<<EOF/\" gen.sh\neval \"$X\"\n";
        assert!(quoted_heredoc_lines(src).is_empty());
    }

    #[test]
    fn test_GH226_quoting_unterminated_heredoc_does_not_blind_the_file() {
        // Mirror of the unterminated-quote fail-safe.
        let src = "cat <<'EOF'\nreport\neval \"$USER_INPUT\"\n";
        assert!(quoted_heredoc_lines(src).is_empty());
        assert_eq!(mask_literals(src), src);
    }

    #[test]
    fn test_GH226_quoting_terminated_heredoc_is_still_reported() {
        let src = "cat <<'EOF'\nreport [0-9]\nEOF\necho ok\n";
        assert_eq!(quoted_heredoc_lines(src), [2].into_iter().collect());
    }

    #[test]
    fn test_GH226_quoting_unquoted_heredoc_is_not_a_quoted_region() {
        let src = "cat <<EOF\nvalue $x\nEOF\n";
        assert!(quoted_heredoc_lines(src).is_empty());
    }

    #[test]
    fn test_GH226_quoting_bare_arithmetic_left_shift_is_not_a_heredoc() {
        // `(( a << b ))` left the stack empty, so `<<` looked like a heredoc
        // opener and masked the rest of the file as its body.
        let src = "(( mask = one << shift ))\nif [ -f y]; then :; fi\n";
        assert_eq!(mask_literals(src), src);
        assert!(quoted_heredoc_lines(src).is_empty());
    }

    #[test]
    fn test_GH226_quoting_ansi_c_escaped_apostrophe_keeps_parity() {
        // `$'don\'t'` is ONE word. Treating `$'` as a bare `$` plus an ordinary
        // `'` flipped quote parity for the rest of the file, in both directions.
        let src = concat!(r#"x=$'don\'t' ; echo 'ok'"#, "\n");
        let regions = QuotedRegions::analyze(src);
        // The ANSI-C literal body is text ...
        assert!(
            regions.is_literal(1, 7),
            "the `don` inside $'...' is literal"
        );
        // ... and the trailing `'ok'` is still recognised as its own literal,
        // which only holds if the escaped apostrophe did not close the region.
        assert!(regions.is_literal(1, 21), "'ok' must still be a literal");
    }

    #[test]
    fn test_GH226_quoting_ansi_c_does_not_leak_into_later_lines() {
        let src = concat!(
            r#"printf $'bad \'%s\'' "$c""#,
            "\n",
            "if [ -f y]; then :; fi\n"
        );
        let regions = QuotedRegions::analyze(src);
        assert!(!regions.is_literal(2, 10), "line 2 must stay lintable");
    }

    #[test]
    fn test_GH226_quoting_unterminated_quote_does_not_panic() {
        for src in [
            "echo 'unterminated",
            "echo \"unterminated",
            "x=$(",
            "x=${",
            "cat <<",
        ] {
            let _ = QuotedRegions::analyze(src);
        }
    }

    /// No allowlisted rule may report a position that lies inside a literal —
    /// checked through `lint_shell`, the path callers actually use.
    ///
    /// This is general over the whole allowlist, so it needs no per-rule
    /// fixture, and it guards a divergence the module-existence test above
    /// cannot see: `mod_lint_2.rs` derives masked-vs-unmasked from
    /// [`is_quote_sensitive`], but `mod_lint.rs` HARDCODES the choice at each
    /// call site (`sc1014::check(&masked)` beside `sc1017::check(source)`).
    /// Adding a code to the allowlist therefore did nothing on that path, and
    /// nothing failed. SC1028 and SC2104 were added and were still quote-blind
    /// until their call sites were changed by hand; this test is what makes the
    /// next such addition fail loudly instead of silently.
    #[test]
    fn test_GH226_no_allowlisted_rule_reports_inside_a_literal() {
        // Real lines from the infra fleet's machine scripts, each of which put
        // shell-looking punctuation inside a string where it is prose.
        let corpus = [
            r#"[ $# -gt 0 ] || { echo 'usage: rag [--source|--videos]' >&2; exit 2; }"#,
            r#"log "still waiting for the NAS (${waited}s elapsed)""#,
            r#"echo "corpus built; intel's rag-reindex.timer runs daily." >&2"#,
            r#"die "usage: nas-move.sh <source-dir> [--execute]""#,
            r#"grep "^Diff in" "$file""#,
            r#"export PATTERN="PMAT-[0-9]{4}""#,
        ];

        for src in corpus {
            let regions = QuotedRegions::analyze(src);
            for d in crate::linter::lint_shell(src).diagnostics {
                if !is_quote_sensitive(&d.code) {
                    continue;
                }
                assert!(
                    !regions.is_literal(d.span.start_line, d.span.start_col),
                    "{} reported at {}:{}, which is inside a string literal — \
                     it is allowlisted, so it should have been given masked source.\n  {src}",
                    d.code,
                    d.span.start_line,
                    d.span.start_col
                );
            }
        }
    }
}
