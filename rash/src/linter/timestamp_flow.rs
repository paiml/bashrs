//! Where does a `date` value go? Sink analysis for DET002 (GH-230).
//!
//! DET002 used to fire on the *call* to `date`. The reproducibility defect is
//! the timestamp **reaching a build artifact** - a filename, an artifact's
//! contents, a build id, a checksummed payload. A timestamp on a log line is
//! the point of a log line, and a timestamp read from `SOURCE_DATE_EPOCH` is
//! DET002's own remedy. Firing on all three identically made the rule
//! unactionable, so users disabled it.
//!
//! Three things are resolved here:
//!
//! 1. **Literal vs code.** A `date` occurrence inside a comment, a single
//!    quoted string, or a quoted heredoc body is text, not a command.
//! 2. **Adoption of `SOURCE_DATE_EPOCH`.** Any line that reads that variable
//!    is exempt, and the variable it assigns is *cleared* rather than tainted,
//!    so the downstream `cp "out_$STAMP"` is clean too.
//! 3. **Destination.** The captured value is tracked across assignments until
//!    it reaches something classifiable.
//!
//! ## Default-deny
//!
//! A capture whose value is never used stays reported (`SinkClass::Unknown`).
//! That is load-bearing: `F-DET002-SOUND` in `linter-det-idem-v1.yaml` and its
//! four test mirrors all use the shape `ts=$(date +%s)` with no use. Silencing
//! unused captures would be a contract amendment, not a bug fix.
//!
//! ## Documented false negatives
//!
//! * The `SOURCE_DATE_EPOCH` gate is *line* granular, so `date -u +%Y%m%d` on a
//!   line that merely mentions `SOURCE_DATE_EPOCH` elsewhere is exempted.
//! * Only `|` splits a pipeline; `&&`, `||` and `;` do not, so a compound line
//!   is classified as a whole and usually lands on `Unknown` (still reported).

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// Where a timestamp value ends up. Ordered: a later use can only raise it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SinkClass {
    /// Every observed use is stdout/stderr, an append-only log, or a comparison.
    Benign,
    /// No use observed, or a use we cannot classify. Reported.
    Unknown,
    /// Reaches an artifact name/content, a build id, or a checksum. Reported.
    Reproducible,
}

/// One reportable `date` invocation plus the verdict on where its value goes.
#[derive(Debug, Clone)]
pub(crate) struct TimestampUse {
    /// 1-indexed line of the `date` occurrence.
    pub(crate) line: usize,
    /// 1-indexed byte column of the occurrence (matches the pre-GH-230 spans).
    pub(crate) col: usize,
    /// Byte length of the matched pattern (8, 6 or 5).
    pub(crate) len: usize,
    /// Variable the value was captured into, if any.
    pub(crate) var: Option<String>,
    /// Strongest sink reached.
    pub(crate) class: SinkClass,
    /// 1-indexed line of the reproducible sink, when `class == Reproducible`.
    pub(crate) sink_line: Option<usize>,
    /// Trimmed text of that line, for the diagnostic message.
    pub(crate) sink_text: Option<String>,
    /// Whether any use of the value was seen at all.
    saw_use: bool,
}

/// Analyse `source`; one entry per `date` occurrence that is real code and does
/// not adopt `SOURCE_DATE_EPOCH`. Entries come back in line order.
pub(crate) fn analyze(source: &str) -> Vec<TimestampUse> {
    let skip = quoted_heredoc_lines(source);
    let mut st = FlowState::default();
    for (idx, line) in source.lines().enumerate() {
        let ln = idx + 1;
        if !skip.contains(&ln) {
            scan_line(&mut st, ln, line);
        }
    }
    finalize(st.uses)
}

/// A quoted heredoc body is literal text by definition, so no rule should read
/// it as shell. Delegated to the shared GH-217 scanner.
fn quoted_heredoc_lines(source: &str) -> HashSet<usize> {
    crate::linter::heredoc::quoted_heredoc_lines(source)
}

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

/// Commands whose arguments name build artifacts.
const ARTIFACT_CMDS: &[&str] = &[
    "cp", "mv", "install", "ln", "mkdir", "rmdir", "touch", "tar", "zip", "unzip", "gzip", "bzip2",
    "xz", "rsync", "scp", "dd", "cpio", "7z",
];
/// Commands that checksum their input.
const HASH_CMDS: &[&str] = &[
    "sha1sum",
    "sha256sum",
    "sha512sum",
    "md5sum",
    "cksum",
    "shasum",
    "b2sum",
];
/// Package/build drivers; a timestamp in one of these is a build id.
const BUILD_CMDS: &[&str] = &[
    "docker", "podman", "buildah", "git", "npm", "cargo", "helm", "gh",
];
/// Sub-commands that make a `BUILD_CMDS` invocation artifact-producing.
const BUILD_SUBCMDS: &[&str] = &[
    "build", "tag", "publish", "version", "package", "release", "push", "commit",
];
/// Commands whose output is a log or the terminal.
const PRINT_CMDS: &[&str] = &["echo", "printf", "print"];
/// Commands that write to the system log.
const LOG_CMDS: &[&str] = &["logger", "syslog"];
/// Commands that consume a timestamp without producing an artifact.
///
/// `date` is deliberately absent: `date` as the command word of the *source*
/// line is the timestamp's origin, not a destination, so it must stay reported.
const NEUTRAL_CMDS: &[&str] = &["sleep", "test", "true", "false", ":"];
/// Prefixes to skip when looking for the command word.
const CMD_PREFIXES: &[&str] = &["sudo", "env", "command", "nohup", "time", "exec", "eval"];
/// Shell keywords that precede a command word.
const KEYWORDS: &[&str] = &[
    "if", "elif", "while", "until", "then", "do", "else", "!", "{",
];
/// Redirect targets that are not artifacts.
const SINKLESS_TARGETS: &[&str] = &["/dev/null", "/dev/stdout", "/dev/stderr", "/dev/tty"];
/// Normalised variable names (separators stripped, uppercased) that ARE the artifact.
const BUILD_ID_NAMES: &[&str] = &[
    "VERSION",
    "RELEASE",
    "BUILDID",
    "BUILDNUMBER",
    "BUILDTAG",
    "REVISION",
    "ARTIFACT",
    "ARTIFACTNAME",
    "IMAGETAG",
    "PKGVERSION",
    "PACKAGEVERSION",
    "CHECKSUM",
    "DIGEST",
    "TAG",
];
/// Declarators that may precede an assignment.
const DECLARATORS: &[&str] = &["export", "local", "readonly", "declare", "typeset"];
/// Keywords that introduce a condition.
const COND_KEYWORDS: &[&str] = &["if ", "elif ", "while ", "until "];

/// The three `date` spellings DET002 has always looked for, in the original
/// priority order so spans stay byte-identical to the pre-GH-230 output.
const DATE_PATTERNS: [(&str, usize); 3] = [("date +%s", 8), ("$(date", 6), ("`date", 5)];

/// Comment markers that declare a timestamp intentional (issues #43 and #58).
/// Preserved verbatim from the original `det002::is_intentional_timestamp_marker`.
const MARKERS: &[&str] = &[
    // Intentional markers (explicit)
    "intentional: timestamp",
    "intentional timestamp",
    // Result tracking markers
    "timestamp for result tracking",
    "timestamp for tracking",
    // Benchmark markers
    "benchmark result",
    "benchmark recording",
    // Logging markers
    "logging timestamp",
    "log timestamp",
    // Metrics markers (Issue #58)
    "metrics recording",
    "record metric",
    "record-metric",
    "metrics timestamp",
    // Telemetry markers
    "telemetry",
    "observability",
];

// ---------------------------------------------------------------------------
// Literal masking: which bytes of a line are text rather than shell code
// ---------------------------------------------------------------------------

/// Lexical context while scanning one physical line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    /// Top level, or inside `$( )` / `${ }` / backticks: shell code.
    Code,
    /// Inside `$( )`.
    CmdSub,
    /// Inside `${ }`.
    ParamExp,
    /// Inside backticks.
    Backtick,
    /// Inside `'...'`: literal, no expansion at all.
    Single,
    /// Inside `"..."`: literal except for expansions.
    Double,
    /// After an unquoted `#`.
    Comment,
}

/// Per-byte facts about one line.
struct LineMask {
    /// `true` where the byte is literal text rather than shell code.
    literal: Vec<bool>,
    /// Nesting depth at that byte; 1 is the top level.
    depth: Vec<usize>,
    /// Byte offset of the `#` that starts a trailing comment.
    comment: Option<usize>,
}

impl LineMask {
    /// Is this byte literal text (comment, single- or double-quoted body)?
    fn is_literal(&self, i: usize) -> bool {
        self.literal.get(i).copied().unwrap_or(false)
    }

    /// Nesting depth at this byte; 0 when unknown.
    fn depth_at(&self, i: usize) -> usize {
        self.depth.get(i).copied().unwrap_or(0)
    }

    /// Everything before a trailing comment.
    fn code_of<'a>(&self, line: &'a str) -> &'a str {
        match self.comment {
            Some(c) => &line[..c],
            None => line,
        }
    }

    /// A top-level unquoted whitespace byte, i.e. a shell word separator.
    fn is_word_break(&self, b: &[u8], i: usize) -> bool {
        b.get(i).is_some_and(u8::is_ascii_whitespace)
            && !self.is_literal(i)
            && self.depth_at(i) == 1
    }
}

/// One-pass lexer producing a [`LineMask`].
struct Scanner<'a> {
    b: &'a [u8],
    i: usize,
    mask: LineMask,
    stack: Vec<Ctx>,
}

impl<'a> Scanner<'a> {
    /// Lex one line.
    fn scan(line: &'a str) -> LineMask {
        let mut s = Scanner {
            b: line.as_bytes(),
            i: 0,
            mask: LineMask {
                literal: vec![false; line.len()],
                depth: vec![0; line.len()],
                comment: None,
            },
            stack: vec![Ctx::Code],
        };
        while s.i < s.b.len() {
            s.step();
        }
        s.mask
    }

    /// Current lexical context.
    fn top(&self) -> Ctx {
        self.stack.last().copied().unwrap_or(Ctx::Code)
    }

    /// Consume at least one byte.
    fn step(&mut self) {
        let ctx = self.top();
        if let Some(d) = self.mask.depth.get_mut(self.i) {
            *d = self.stack.len();
        }
        match ctx {
            Ctx::Comment => self.step_comment(),
            Ctx::Single => self.step_single(),
            Ctx::Double => self.step_double(),
            _ => self.step_code(),
        }
    }

    /// The rest of the line is literal text.
    fn step_comment(&mut self) {
        for m in self.mask.literal.iter_mut().skip(self.i) {
            *m = true;
        }
        self.i = self.b.len();
    }

    /// Single quotes have no escapes; only `'` ends them.
    fn step_single(&mut self) {
        self.mark_literal(self.i);
        if self.b[self.i] == b'\'' {
            self.stack.pop();
        }
        self.i += 1;
    }

    /// Double quotes are literal except for `$(`, `${` and backticks.
    fn step_double(&mut self) {
        let c = self.b[self.i];
        if c == b'\\' && self.i + 1 < self.b.len() {
            self.mark_literal(self.i);
            self.mark_literal(self.i + 1);
            self.i += 2;
            return;
        }
        if c == b'"' {
            self.mark_literal(self.i);
            self.stack.pop();
            self.i += 1;
            return;
        }
        if self.open_expansion() {
            return;
        }
        self.mark_literal(self.i);
        self.i += 1;
    }

    /// Shell code: quotes open literal regions, `#` opens a comment.
    fn step_code(&mut self) {
        let c = self.b[self.i];
        if c == b'\\' && self.i + 1 < self.b.len() {
            self.i += 2;
            return;
        }
        if self.close_code() {
            self.i += 1;
            return;
        }
        if c == b'\'' || c == b'"' {
            self.mark_literal(self.i);
            self.stack
                .push(if c == b'\'' { Ctx::Single } else { Ctx::Double });
            self.i += 1;
            return;
        }
        if self.open_expansion() {
            return;
        }
        if self.is_comment_start() {
            self.mask.comment = Some(self.i);
            self.stack.push(Ctx::Comment);
            return;
        }
        self.i += 1;
    }

    /// `$(`, `${` and a backtick all (re)open *code*.
    fn open_expansion(&mut self) -> bool {
        let (ctx, width) = match self.pending_opener() {
            Some(v) => v,
            None => return false,
        };
        self.stack.push(ctx);
        self.i += width;
        true
    }

    /// Which expansion, if any, starts at the cursor.
    fn pending_opener(&self) -> Option<(Ctx, usize)> {
        if starts_with(self.b, self.i, b"$(") {
            return Some((Ctx::CmdSub, 2));
        }
        if starts_with(self.b, self.i, b"${") {
            return Some((Ctx::ParamExp, 2));
        }
        if self.b[self.i] == b'`' {
            return Some((Ctx::Backtick, 1));
        }
        None
    }

    /// Does the cursor close the innermost expansion?
    fn close_code(&mut self) -> bool {
        let closes = matches!(
            (self.top(), self.b[self.i]),
            (Ctx::CmdSub, b')') | (Ctx::ParamExp, b'}') | (Ctx::Backtick, b'`')
        );
        if closes {
            self.stack.pop();
        }
        closes
    }

    /// `#` starts a comment only at the top level and only at a word boundary.
    fn is_comment_start(&self) -> bool {
        if self.b[self.i] != b'#' || self.stack.len() != 1 {
            return false;
        }
        self.i == 0 || self.b[self.i - 1].is_ascii_whitespace() || self.b[self.i - 1] == b';'
    }

    /// Record one byte as literal text.
    fn mark_literal(&mut self, i: usize) {
        if let Some(m) = self.mask.literal.get_mut(i) {
            *m = true;
        }
    }
}

/// Does `b` contain `pat` starting exactly at `i`?
fn starts_with(b: &[u8], i: usize, pat: &[u8]) -> bool {
    b.len() >= i + pat.len() && &b[i..i + pat.len()] == pat
}

/// First byte offset at or after `from` where `pat` occurs in `hay`.
fn find_from(hay: &[u8], pat: &[u8], from: usize) -> Option<usize> {
    if pat.is_empty() || hay.len() < pat.len() {
        return None;
    }
    (from..=hay.len() - pat.len()).find(|&i| &hay[i..i + pat.len()] == pat)
}

// ---------------------------------------------------------------------------
// Word / segment splitting
// ---------------------------------------------------------------------------

/// Split `s` into shell words at unquoted top-level whitespace.
fn split_words(s: &str) -> Vec<&str> {
    let m = Scanner::scan(s);
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for i in 0..b.len() {
        if m.is_word_break(b, i) {
            if let Some(st) = start.take() {
                out.push(&s[st..i]);
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(st) = start {
        out.push(&s[st..]);
    }
    out
}

/// Split a command line on unquoted top-level `|` that is not part of `||`.
fn pipeline_segments(code: &str) -> Vec<&str> {
    let m = Scanner::scan(code);
    let b = code.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    for i in 0..b.len() {
        if is_pipe_at(b, i, &m) {
            out.push(&code[start..i]);
            start = i + 1;
        }
    }
    out.push(&code[start..]);
    out
}

/// Is byte `i` a pipeline separator?
fn is_pipe_at(b: &[u8], i: usize, m: &LineMask) -> bool {
    b[i] == b'|'
        && !m.is_literal(i)
        && m.depth_at(i) == 1
        && b.get(i + 1) != Some(&b'|')
        && (i == 0 || b[i - 1] != b'|')
}

/// `NAME` and the byte offset just past `=`, for a leading `NAME=` / `NAME+=`.
fn split_name_eq(s: &str) -> Option<(&str, usize)> {
    let b = s.as_bytes();
    if b.is_empty() || !(b[0].is_ascii_alphabetic() || b[0] == b'_') {
        return None;
    }
    let mut i = 0;
    while i < b.len() && (b[i] == b'_' || b[i].is_ascii_alphanumeric()) {
        i += 1;
    }
    let mut j = i;
    if b.get(j) == Some(&b'+') {
        j += 1;
    }
    if b.get(j) != Some(&b'=') {
        return None;
    }
    Some((&s[..i], j + 1))
}

/// The command name of a segment, with env prefixes, keywords and flags skipped.
fn command_word(seg: &str) -> Option<String> {
    for w in split_words(seg) {
        if is_skippable_prefix(w) {
            continue;
        }
        return Some(basename(w).to_string());
    }
    None
}

/// Words that come *before* the command name.
fn is_skippable_prefix(w: &str) -> bool {
    CMD_PREFIXES.contains(&w)
        || KEYWORDS.contains(&w)
        || w.starts_with('-')
        || split_name_eq(w).is_some()
}

/// Last path component of a command word (`/usr/bin/cp` -> `cp`).
fn basename(w: &str) -> &str {
    match w.rsplit('/').next() {
        Some(x) if !x.is_empty() => x,
        _ => w,
    }
}

/// Drop one layer of surrounding quotes.
fn unquote(s: &str) -> &str {
    let t = s.trim();
    for q in ['"', '\''] {
        if t.len() >= 2 && t.starts_with(q) && t.ends_with(q) {
            return &t[1..t.len() - 1];
        }
    }
    t
}

// ---------------------------------------------------------------------------
// Redirections
// ---------------------------------------------------------------------------

/// The last output redirection of a pipeline segment.
enum Redirect<'a> {
    /// `> target` - truncating write, i.e. the file IS the artifact.
    Truncate(&'a str),
    /// `>> target` - append-only, i.e. a log.
    Append(&'a str),
    /// `>&2`, `2>&1` - a file-descriptor dup, not a file.
    Fd,
}

/// Find the last unquoted top-level output redirection.
fn redirect_of(seg: &str) -> Option<Redirect<'_>> {
    let m = Scanner::scan(seg);
    let b = seg.as_bytes();
    let mut found = None;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'>' && !m.is_literal(i) && m.depth_at(i) == 1 {
            let (r, next) = parse_redirect(seg, b, i);
            found = Some(r);
            i = next;
        } else {
            i += 1;
        }
    }
    found
}

/// Parse one redirection starting at the `>` at `i`; return it and the next index.
fn parse_redirect<'a>(seg: &'a str, b: &[u8], i: usize) -> (Redirect<'a>, usize) {
    let append = b.get(i + 1) == Some(&b'>');
    let mut j = i + if append { 2 } else { 1 };
    if b.get(j) == Some(&b'&') {
        return (Redirect::Fd, j + 2);
    }
    while j < b.len() && b[j].is_ascii_whitespace() {
        j += 1;
    }
    let target = split_words(&seg[j..]).first().copied().unwrap_or("");
    let end = (j + target.len()).max(i + 1);
    let r = if append {
        Redirect::Append(target)
    } else {
        Redirect::Truncate(target)
    };
    (r, end)
}

/// Is this redirect target a bit bucket rather than a build artifact?
fn is_sinkless(target: &str) -> bool {
    SINKLESS_TARGETS.contains(&unquote(target))
}

/// Does this segment carry `tee`'s append flag?
fn has_append_flag(seg: &str) -> bool {
    split_words(seg)
        .iter()
        .any(|w| *w == "-a" || *w == "--append")
}

// ---------------------------------------------------------------------------
// Sink taxonomy
// ---------------------------------------------------------------------------

/// What we are tracking through the script.
enum Needle<'a> {
    /// The literal `date` spelling on the source line.
    Text(&'a str),
    /// A shell variable holding the captured value.
    Var(&'a str),
}

impl Needle<'_> {
    /// Does `hay` carry the tracked value?
    fn found_in(&self, hay: &str) -> bool {
        match self {
            Needle::Text(t) => hay.contains(t),
            Needle::Var(v) => references(hay, v),
        }
    }
}

/// Does `hay` expand `$var`, `${var}` or `${var:...}`?
fn references(hay: &str, var: &str) -> bool {
    let b = hay.as_bytes();
    let vb = var.as_bytes();
    let mut i = 0;
    while let Some(p) = find_from(b, b"$", i) {
        i = p + 1;
        let mut j = i;
        if b.get(j) == Some(&b'{') {
            j += 1;
        }
        if !starts_with(b, j, vb) {
            continue;
        }
        let after = b.get(j + vb.len());
        if !after.is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_') {
            return true;
        }
    }
    false
}

/// Strongest sink the value reaches on this command line.
fn classify_sink(code: &str, n: &Needle<'_>) -> SinkClass {
    let segs = pipeline_segments(code);
    let last = segs.len().saturating_sub(1);
    let mut carries = false;
    let mut class = SinkClass::Unknown;
    for (i, seg) in segs.iter().enumerate() {
        carries = carries || n.found_in(seg);
        if !carries {
            continue;
        }
        if is_reproducible_segment(seg, n) {
            return SinkClass::Reproducible;
        }
        if i == last && is_benign_segment(seg, n) {
            class = SinkClass::Benign;
        }
    }
    class
}

/// Does this segment write the value into a build artifact?
fn is_reproducible_segment(seg: &str, n: &Needle<'_>) -> bool {
    reproducible_by_command(seg, n) || reproducible_by_redirect(seg, n)
}

/// Artifact-producing command words.
fn reproducible_by_command(seg: &str, n: &Needle<'_>) -> bool {
    let Some(cw) = command_word(seg) else {
        return false;
    };
    if HASH_CMDS.contains(&cw.as_str()) {
        return true;
    }
    if cw == "tee" {
        return !has_append_flag(seg);
    }
    if ARTIFACT_CMDS.contains(&cw.as_str()) {
        return n.found_in(seg);
    }
    is_build_command(seg, &cw) && n.found_in(seg)
}

/// `docker build`, `git tag`, `npm publish`, ...
fn is_build_command(seg: &str, cw: &str) -> bool {
    BUILD_CMDS.contains(&cw)
        && split_words(seg)
            .iter()
            .any(|w| BUILD_SUBCMDS.contains(&unquote(w)))
}

/// A truncating write is the artifact; an append whose *name* is timestamped is too.
fn reproducible_by_redirect(seg: &str, n: &Needle<'_>) -> bool {
    match redirect_of(seg) {
        Some(Redirect::Truncate(t)) => !is_sinkless(t),
        Some(Redirect::Append(t)) => n.found_in(t),
        _ => false,
    }
}

/// Does this segment merely print, log or compare the value?
fn is_benign_segment(seg: &str, n: &Needle<'_>) -> bool {
    if is_test_context(seg, n) {
        return true;
    }
    if matches!(redirect_of(seg), Some(Redirect::Append(t)) if !n.found_in(t)) {
        return true;
    }
    match command_word(seg) {
        Some(cw) => benign_command(&cw, seg, n),
        None => false,
    }
}

/// Command words whose output is not an artifact.
fn benign_command(cw: &str, seg: &str, n: &Needle<'_>) -> bool {
    if LOG_CMDS.contains(&cw) || NEUTRAL_CMDS.contains(&cw) {
        return true;
    }
    if cw == "tee" {
        return has_append_flag(seg);
    }
    PRINT_CMDS.contains(&cw) && redirect_is_benign(redirect_of(seg), n)
}

/// Is this redirection incapable of producing a build artifact?
fn redirect_is_benign(r: Option<Redirect<'_>>, n: &Needle<'_>) -> bool {
    match r {
        None | Some(Redirect::Fd) => true,
        Some(Redirect::Append(t)) => !n.found_in(t),
        Some(Redirect::Truncate(t)) => is_sinkless(t),
    }
}

/// Is the value only being compared or arithmetically tested?
fn is_test_context(seg: &str, n: &Needle<'_>) -> bool {
    if let Some(cond) = condition_part(seg) {
        return n.found_in(cond);
    }
    let t = seg.trim_start();
    t.starts_with("[ ") || t.starts_with("[[ ") || t.starts_with("((") || t.starts_with("test ")
}

/// The condition of an `if`/`elif`/`while`/`until`, up to the first `;`.
fn condition_part(seg: &str) -> Option<&str> {
    let t = seg.trim_start();
    let kw = COND_KEYWORDS.iter().find(|k| t.starts_with(**k))?;
    let rest = &t[kw.len()..];
    let end = rest.find(';').unwrap_or(rest.len());
    Some(&rest[..end])
}

/// Is this variable name itself the artifact identifier?
fn is_build_id_name(name: &str) -> bool {
    let norm: String = name
        .chars()
        .filter(|c| *c != '_' && *c != '-')
        .flat_map(char::to_uppercase)
        .collect();
    BUILD_ID_NAMES.contains(&norm.as_str())
}

// ---------------------------------------------------------------------------
// Line classification helpers
// ---------------------------------------------------------------------------

/// Reading `SOURCE_DATE_EPOCH` is DET002's own remedy, so the line is exempt.
fn adopts_source_date_epoch(line: &str) -> bool {
    line.contains("SOURCE_DATE_EPOCH")
}

/// Is there an intentional-timestamp marker in this line's *comment*?
///
/// GH-230: the original matched the marker list against the whole line, so a
/// `curl https://telemetry.example.com/...` silently disabled the rule for the
/// following assignment block.
fn marker_in_comment(line: &str, mask: &LineMask) -> bool {
    let Some(c) = mask.comment else {
        return false;
    };
    let tail = line[c..].to_lowercase();
    MARKERS.iter().any(|m| tail.contains(m))
}

/// Check if timestamp is used for file tracking (not program logic).
/// Preserved verbatim from the original `det002::is_timestamp_for_tracking`.
fn is_timestamp_for_tracking(line: &str) -> bool {
    let line_trimmed = line.trim();
    if line_trimmed.starts_with("if ")
        || line_trimmed.starts_with("elif ")
        || line_trimmed.starts_with("while ")
        || line_trimmed.contains("[ $(date")
        || line_trimmed.contains("[[ $(date")
    {
        return false;
    }
    line_trimmed.contains('=') && !line_trimmed.starts_with('[')
}

/// Check if line is a variable assignment.
/// Preserved verbatim from the original `det002::is_variable_assignment`.
fn is_variable_assignment(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && trimmed.contains('=')
        && !trimmed.starts_with('[')
}

/// `(name, RHS offset)` for a *pure* assignment.
///
/// Returns `None` for `NAME=VALUE cmd ...`, which is an environment prefix on a
/// command line, not an assignment.
fn assignment_target(code: &str) -> Option<(String, usize)> {
    let mut off = code.len() - code.trim_start().len();
    while let Some(word) = split_words(&code[off..]).first().copied() {
        if !DECLARATORS.contains(&word) {
            break;
        }
        off += word.len();
        let rest = &code[off..];
        off += rest.len() - rest.trim_start().len();
    }
    let (name, eq) = split_name_eq(&code[off..])?;
    let rhs = off + eq;
    if !is_pure_assignment(&code[rhs..]) {
        return None;
    }
    Some((name.to_string(), rhs))
}

/// A pure assignment's RHS is a single shell word (optionally then a comment).
fn is_pure_assignment(rhs: &str) -> bool {
    let words = split_words(rhs);
    match words.get(1) {
        None => true,
        Some(w) => w.starts_with('#'),
    }
}

/// The `date` occurrence to report on this line, if any is real shell code.
fn find_date(line: &str, mask: &LineMask) -> Option<(usize, &'static str, usize)> {
    let b = line.as_bytes();
    for (pat, len) in DATE_PATTERNS {
        let mut from = 0;
        while let Some(col) = find_from(b, pat.as_bytes(), from) {
            if !mask.is_literal(col) {
                return Some((col, pat, len));
            }
            from = col + 1;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Flow
// ---------------------------------------------------------------------------

/// Taint state while walking the script top to bottom.
///
/// `tainted` is an insertion-ordered `Vec` rather than a `HashMap` on purpose:
/// diagnostics must be byte-identical between runs, and map iteration order is
/// not.
#[derive(Default)]
struct FlowState {
    uses: Vec<TimestampUse>,
    tainted: Vec<(String, usize)>,
    marker_ctx: bool,
}

impl FlowState {
    /// Bind `name` to the timestamp recorded at `idx`.
    fn taint(&mut self, name: &str, idx: usize) {
        self.untaint(name);
        self.tainted.push((name.to_string(), idx));
    }

    /// `name` no longer holds a timestamp.
    fn untaint(&mut self, name: &str) {
        self.tainted.retain(|(v, _)| v != name);
    }

    /// Every tainted variable this code references, in binding order.
    fn referenced(&self, code: &str) -> Vec<(String, usize)> {
        self.tainted
            .iter()
            .filter(|(v, _)| references(code, v))
            .cloned()
            .collect()
    }
}

/// Walk one physical line.
fn scan_line(st: &mut FlowState, ln: usize, line: &str) {
    let mask = Scanner::scan(line);
    if marker_in_comment(line, &mask) {
        st.marker_ctx = true;
        return;
    }
    update_marker_ctx(st, line);
    let code = mask.code_of(line);
    if adopts_source_date_epoch(line) {
        clear_sde_target(st, code);
        return;
    }
    match find_date(line, &mask) {
        Some(hit) => handle_source(st, ln, code, hit),
        None => handle_flow(st, ln, code),
    }
}

/// The marker context survives comments and assignments only.
fn update_marker_ctx(st: &mut FlowState, line: &str) {
    let t = line.trim();
    if !t.is_empty() && !t.starts_with('#') && !is_variable_assignment(line) {
        st.marker_ctx = false;
    }
}

/// A value derived from `SOURCE_DATE_EPOCH` is reproducible, so clear its taint.
fn clear_sde_target(st: &mut FlowState, code: &str) {
    if let Some((name, _)) = assignment_target(code) {
        st.untaint(&name);
    }
}

/// A line that invokes `date`.
fn handle_source(st: &mut FlowState, ln: usize, code: &str, hit: (usize, &'static str, usize)) {
    if st.marker_ctx && is_timestamp_for_tracking(code) {
        return;
    }
    let (col0, pat, len) = hit;
    let var = assignment_target(code).map(|(n, _)| n);
    let idx = st.uses.len();
    st.uses.push(TimestampUse {
        line: ln,
        col: col0 + 1,
        len,
        var: var.clone(),
        class: SinkClass::Benign,
        sink_line: None,
        sink_text: None,
        saw_use: false,
    });
    match var {
        Some(name) => start_taint(st, idx, &name, ln, code),
        None => record(st, idx, classify_sink(code, &Needle::Text(pat)), ln, code),
    }
}

/// Capture into a variable; a build-id name is itself the artifact.
fn start_taint(st: &mut FlowState, idx: usize, name: &str, ln: usize, code: &str) {
    st.taint(name, idx);
    if is_build_id_name(name) {
        record(st, idx, SinkClass::Reproducible, ln, code);
    }
}

/// A line that does not invoke `date`: it may propagate or consume a taint.
fn handle_flow(st: &mut FlowState, ln: usize, code: &str) {
    match assignment_target(code) {
        Some((name, _)) => handle_propagation(st, ln, code, &name),
        None => handle_uses(st, ln, code),
    }
}

/// `NAME="...$TS..."` carries the taint; any other RHS kills it.
fn handle_propagation(st: &mut FlowState, ln: usize, code: &str, name: &str) {
    match st.referenced(code).first() {
        Some(&(_, idx)) => {
            st.taint(name, idx);
            if is_build_id_name(name) {
                record(st, idx, SinkClass::Reproducible, ln, code);
            }
        }
        None => st.untaint(name),
    }
}

/// Classify every tainted value this command line consumes.
fn handle_uses(st: &mut FlowState, ln: usize, code: &str) {
    for (name, idx) in st.referenced(code) {
        let class = classify_sink(code, &Needle::Var(&name));
        record(st, idx, class, ln, code);
    }
}

/// Fold one observed use into a timestamp's verdict.
fn record(st: &mut FlowState, idx: usize, class: SinkClass, ln: usize, code: &str) {
    let Some(u) = st.uses.get_mut(idx) else {
        return;
    };
    u.saw_use = true;
    if class > u.class {
        u.class = class;
    }
    if class == SinkClass::Reproducible && u.sink_line.is_none() {
        u.sink_line = Some(ln);
        u.sink_text = Some(code.trim().to_string());
    }
}

/// Default-deny: a capture with no observed use stays reported.
fn finalize(mut uses: Vec<TimestampUse>) -> Vec<TimestampUse> {
    for u in &mut uses {
        if !u.saw_use {
            u.class = SinkClass::Unknown;
        }
    }
    uses
}

#[cfg(test)]
#[path = "timestamp_flow_tests.rs"]
mod timestamp_flow_tests;
