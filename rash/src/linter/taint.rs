//! Intra-file taint analysis for the path-safety rules (SEC010, SEC014).
//!
//! # Why this exists (GH-227)
//!
//! SEC010 used to fire whenever a line contained a `$` and any of a dozen
//! generic substrings (`DIR`, `FILE`, `PATH`, `NAME`, …) appeared **anywhere**
//! on that line. `mkdir -p "$OUT_DIR"` was an `Error` even when `OUT_DIR` had
//! been assigned a string literal three lines above — the substring `DIR`
//! matched, and there was no dataflow of any kind. SEC014 had no dataflow
//! either. Meanwhile a *real* inline `case` guard did not clear the finding,
//! and a function *named* `validate_path` whose body was `:` did.
//!
//! This module supplies the missing precondition: a path finding is reported
//! only when the path expression can actually be influenced by input from
//! outside the script.
//!
//! # Scope
//!
//! Deliberately bounded: ONE file, ONE ordered pass over physical lines, no
//! fixpoint, no worklist, no path sensitivity. Guard and validator recognition
//! are two bounded pre-passes. See `KNOWN LIMITATIONS` at the bottom.

use std::collections::{HashMap, HashSet};

/// Maximum number of lines scanned looking for a guard block's closer.
const GUARD_BLOCK_CAP: usize = 60;

/// Maximum number of lines scanned for a function body.
const FUNCTION_BODY_CAP: usize = 200;

/// Environment variables whose value the script owner controls, or that cannot
/// carry a traversal payload in practice. Superset of SEC010's old
/// `SAFE_VAR_PATTERNS` (issue #73).
const SAFE_ENV_VARS: &[&str] = &[
    "PWD",
    "OLDPWD",
    "HOME",
    "TMPDIR",
    "PATH",
    "SHELL",
    "TERM",
    "LANG",
    "LC_ALL",
    "USER",
    "LOGNAME",
    "HOSTNAME",
    "UID",
    "EUID",
    "PPID",
    "SHLVL",
    "LINENO",
    "RANDOM",
    "SECONDS",
    "IFS",
    "BASH_SOURCE",
    "BASH_VERSION",
    "FUNCNAME",
    "PS1",
];

/// Shell metadata expansions that carry no attacker-controlled payload.
const SHELL_META: &[&str] = &["#", "?", "$", "!", "-"];

/// Expansions that are, by construction, filled from outside the script.
const EXTERNAL_SPECIALS: &[&str] = &["OPTARG", "REPLY"];

/// Single-character expansions we are willing to lex as a name.
const SPECIAL_CHARS: &[u8] = b"@*#?$!-";

/// Commands whose stdout is data fetched from outside the machine.
const EXTERNAL_DATA_CMDS: &[&str] = &[
    "curl", "wget", "nc", "ncat", "ssh", "scp", "git", "aws", "gcloud", "kubectl", "jq",
];

/// Commands that write external input straight into shell variables.
const READ_COMMANDS: &[&str] = &["read", "readarray", "mapfile"];

/// `read` options that consume the following token as their argument.
const READ_OPTS_WITH_ARG: &[&str] = &["-p", "-d", "-n", "-N", "-t", "-u", "-i"];

/// Bytes that may precede the name of a shell assignment.
const ASSIGN_DELIMS: &[u8] = b" \t;&|(){}";

/// Characters that end one statement and begin the next.
const STATEMENT_SEPARATORS: &[char] = &[';', '&', '|', '\n', '(', ')', '{', '}'];

/// Words that may introduce a statement without being its command.
const STATEMENT_KEYWORDS: &[&str] = &["if", "then", "do", "while", "until", "time", "!"];

/// Keywords that open a conditional or loop body.
const BLOCK_OPENERS: &[&str] = &["if", "case", "while", "until", "for"];

/// Keywords that close one.
const BLOCK_CLOSERS: &[&str] = &["fi", "esac", "done"];

/// How much external influence a value may carry.
///
/// Ordered: `Clean < Ambient < External`. Propagation takes the maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TaintKind {
    /// Transitively derived from string literals only, or sanitised by a
    /// dominating guard.
    #[default]
    Clean,
    /// Never assigned in this file: an environment variable, or one set by a
    /// `source`d file. Real but *unproven* external influence.
    Ambient,
    /// Definitely external: positional parameters, `$@`/`$*`, `read`,
    /// `getopts`/`$OPTARG`, `$REPLY`, network command substitution.
    External,
}

/// Result of the taint pass: for every variable, the ordered list of line
/// indices at which its taint changed.
#[derive(Debug, Clone, Default)]
pub struct TaintMap {
    /// `var -> [(effective_from_line_idx, kind)]`, sorted by line index.
    changes: HashMap<String, Vec<(usize, TaintKind)>>,
    /// 0-based indices of lines that belong to a function definition.
    in_function: HashSet<usize>,
}

impl TaintMap {
    /// Taint of `var` as line `line_idx` (0-based) executes.
    pub fn var_taint(&self, line_idx: usize, var: &str) -> TaintKind {
        match self.recorded(line_idx, var) {
            Some(kind) => kind,
            None => contextual_intrinsic(var, self.in_function.contains(&line_idx)),
        }
    }

    /// Highest taint among every variable expansion appearing in `line`.
    ///
    /// `Clean` means nothing on this line can be externally influenced.
    pub fn line_taint(&self, line_idx: usize, line: &str) -> TaintKind {
        var_names(line)
            .into_iter()
            .map(|name| self.var_taint(line_idx, &name))
            .max()
            .unwrap_or_default()
    }

    /// Highest taint among the variables interpolated into a slash-separated
    /// path fragment — SEC014's shape, e.g. `"/data/$X/$Y"`.
    pub fn path_taint(&self, line_idx: usize, args: &str) -> TaintKind {
        args.split('/')
            .filter(|part| part.contains('$'))
            .flat_map(var_names)
            .map(|name| self.var_taint(line_idx, &name))
            .max()
            .unwrap_or_default()
    }

    fn recorded(&self, line_idx: usize, var: &str) -> Option<TaintKind> {
        let history = self.changes.get(var)?;
        let pos = history.partition_point(|(from, _)| *from <= line_idx);
        history.get(pos.checked_sub(1)?).map(|(_, kind)| *kind)
    }
}

/// Mutable state carried down the file by [`analyze`].
#[derive(Debug, Default)]
struct Ctx {
    current: HashMap<String, TaintKind>,
    changes: HashMap<String, Vec<(usize, TaintKind)>>,
    /// `var -> conditional construct its last assignment sat in`. Two
    /// assignments merge only when they are branches of the *same* construct.
    blocks: HashMap<String, Option<usize>>,
    /// Whether the line currently being applied sits in a function body.
    in_function: bool,
}

impl Ctx {
    /// Record that `name` has taint `kind` from line `from` onward.
    fn set(&mut self, name: String, kind: TaintKind, from: usize) {
        if self.current.get(&name) == Some(&kind) {
            return;
        }
        self.current.insert(name.clone(), kind);
        self.changes.entry(name).or_default().push((from, kind));
    }

    fn get(&self, name: &str) -> TaintKind {
        match self.current.get(name) {
            Some(kind) => *kind,
            None => contextual_intrinsic(name, self.in_function),
        }
    }
}

/// Run the taint pass over a whole script.
///
/// One ordered pass over the physical lines, plus three bounded pre-passes
/// (heredoc regions, function bodies, guards). Total and panic-free.
pub fn analyze(source: &str) -> TaintMap {
    let lines: Vec<&str> = source.lines().collect();
    let heredoc_body = crate::linter::heredoc::quoted_heredoc_lines(source);
    let in_function = function_body_lines(&lines);
    let validators = collect_validator_functions(&lines);
    let untaints = guard_untaints(&lines, &in_function);
    let blocks = block_ids(&lines, &heredoc_body);

    let mut ctx = Ctx::default();
    for (idx, line) in lines.iter().enumerate() {
        if heredoc_body.contains(&(idx + 1)) || line.trim_start().starts_with('#') {
            continue;
        }
        ctx.in_function = in_function.contains(&idx);
        apply_line(&mut ctx, line, idx + 1, blocks.get(idx).copied().flatten());
        apply_validator_call(&mut ctx, line, &validators, idx + 1);
        apply_untaints(&mut ctx, untaints.get(&idx), idx + 1);
    }

    TaintMap {
        changes: ctx.changes,
        in_function,
    }
}

fn apply_validator_call(
    ctx: &mut Ctx,
    line: &str,
    validators: &HashSet<String>,
    from: usize,
) -> bool {
    match validator_call_var(line, validators) {
        Some(var) => {
            ctx.set(var, TaintKind::Clean, from);
            true
        }
        None => false,
    }
}

fn apply_untaints(ctx: &mut Ctx, vars: Option<&Vec<String>>, from: usize) {
    for var in vars.into_iter().flatten() {
        ctx.set(var.clone(), TaintKind::Clean, from);
    }
}

// ---------------------------------------------------------------------------
// taint of a single reference
// ---------------------------------------------------------------------------

/// Taint of a name the file never assigns, at a point that may be inside a
/// function body.
///
/// A function's `$1` comes from its *caller*, which this pass does not resolve.
/// Calling it `External` would be a claim we cannot support — and, with graded
/// severity, would break builds on a guess. It is downgraded to `Ambient`.
fn contextual_intrinsic(name: &str, in_function: bool) -> TaintKind {
    let kind = intrinsic_taint(name);
    if in_function && kind == TaintKind::External && positional_taint(name).is_some() {
        return TaintKind::Ambient;
    }
    kind
}

/// Taint of a name that the file never assigns.
fn intrinsic_taint(name: &str) -> TaintKind {
    if let Some(kind) = positional_taint(name) {
        return kind;
    }
    if SHELL_META.contains(&name) {
        return TaintKind::Clean;
    }
    if EXTERNAL_SPECIALS.contains(&name) || name.starts_with('!') {
        // `${!x}` is indirect expansion: unresolvable, so assume the worst.
        return TaintKind::External;
    }
    if is_safe_env_var(name) {
        return TaintKind::Clean;
    }
    TaintKind::Ambient
}

fn positional_taint(name: &str) -> Option<TaintKind> {
    if name == "@" || name == "*" {
        return Some(TaintKind::External);
    }
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if name == "0" {
        return Some(TaintKind::Clean);
    }
    Some(TaintKind::External)
}

fn is_safe_env_var(name: &str) -> bool {
    name.starts_with("XDG_") || SAFE_ENV_VARS.contains(&name)
}

// ---------------------------------------------------------------------------
// lexing variable expansions
// ---------------------------------------------------------------------------

/// Every variable name expanded in `text`, in order of appearance.
fn var_names(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'$' {
            i += 1;
            continue;
        }
        match read_var_at(bytes, i) {
            Some((name, next)) => {
                out.push(name);
                i = next;
            }
            None => i += 1,
        }
    }
    out
}

/// Read the expansion starting at `i` (which must be a `$`).
fn read_var_at(bytes: &[u8], i: usize) -> Option<(String, usize)> {
    let start = i + 1;
    match bytes.get(start) {
        None => None,
        Some(b'{') => read_braced(bytes, start + 1),
        Some(b'(') => None,
        Some(_) => read_plain(bytes, start),
    }
}

fn read_plain(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let first = *bytes.get(start)?;
    if first.is_ascii_digit() {
        let end = scan_while(bytes, start, |b| b.is_ascii_digit());
        return Some((slice_string(bytes, start, end), end));
    }
    if first.is_ascii_alphabetic() || first == b'_' {
        let end = scan_while(bytes, start, is_name_byte);
        return Some((slice_string(bytes, start, end), end));
    }
    if SPECIAL_CHARS.contains(&first) {
        return Some((slice_string(bytes, start, start + 1), start + 1));
    }
    None
}

/// Read a `${…}` expansion; `start` is the byte after the `{`.
fn read_braced(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let close = scan_while(bytes, start, |b| b != b'}');
    if close >= bytes.len() {
        return None;
    }
    let name = brace_name(&slice_string(bytes, start, close))?;
    Some((name, close + 1))
}

/// Extract the variable name from the inside of a `${…}` expansion.
fn brace_name(inner: &str) -> Option<String> {
    let indirect = inner.starts_with('!');
    let body = inner.trim_start_matches(['!', '#']);
    let name: String = body
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return None;
    }
    if indirect {
        return Some(format!("!{name}"));
    }
    Some(name)
}

fn scan_while(bytes: &[u8], start: usize, pred: impl Fn(u8) -> bool) -> usize {
    let mut end = start;
    while end < bytes.len() && pred(bytes[end]) {
        end += 1;
    }
    end
}

fn slice_string(bytes: &[u8], start: usize, end: usize) -> String {
    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_valid_name(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ---------------------------------------------------------------------------
// code / literal / comment separation
//
// GH-227 regression: matching a control keyword against the raw text of a line
// makes the *mention* of that keyword — in an `echo`, in a comment, in a `trap`
// string — indistinguishable from running it. Everything that decides whether a
// guard rejects its input therefore looks at code only.
// ---------------------------------------------------------------------------

/// `text` with comments dropped and every string literal blanked.
fn code_only(text: &str) -> String {
    scan_text(text, true)
}

/// `text` with comments dropped, literals intact — for pattern matching, where
/// the *contents* of `*".."*` are the thing being looked for.
fn bare_code(text: &str) -> String {
    scan_text(text, false)
}

/// Quoting is resolved one physical line at a time, matching the rest of this
/// pass; an unbalanced quote cannot then swallow the remainder of a block.
fn scan_text(text: &str, blank_literals: bool) -> String {
    text.split('\n')
        .map(|line| scan_line(line, blank_literals))
        .collect::<Vec<_>>()
        .join("\n")
}

fn scan_line(line: &str, blank_literals: bool) -> String {
    let mut out = String::with_capacity(line.len());
    let mut quote: Option<char> = None;
    for c in line.chars() {
        match quote {
            Some(q) => quote = scan_in_quote(&mut out, c, q, blank_literals),
            None => {
                if !scan_in_code(&mut out, c, &mut quote, blank_literals) {
                    break;
                }
            }
        }
    }
    out
}

/// Consume one character inside a literal; returns the quote state after it.
fn scan_in_quote(out: &mut String, c: char, quote: char, blank: bool) -> Option<char> {
    if c == quote {
        out.push(if blank { '\u{1}' } else { c });
        return None;
    }
    if !blank {
        out.push(c);
    }
    Some(quote)
}

/// Consume one character outside a literal; returns false when a comment starts
/// and the rest of the line is dropped.
fn scan_in_code(out: &mut String, c: char, quote: &mut Option<char>, blank: bool) -> bool {
    if c == '#' && starts_word(out) {
        return false;
    }
    if c == '"' || c == '\'' {
        *quote = Some(c);
        if !blank {
            out.push(c);
        }
        return true;
    }
    out.push(c);
    true
}

fn starts_word(out: &str) -> bool {
    match out.chars().last() {
        None => true,
        Some(c) => c.is_whitespace(),
    }
}

/// The statements of a chunk of code, in order.
fn statements(code: &str) -> std::str::Split<'_, &'static [char]> {
    code.split(STATEMENT_SEPARATORS)
}

/// The command word of a statement and its first argument.
fn command_word(stmt: &str) -> Option<(&str, Option<&str>)> {
    let mut words = stmt
        .split_whitespace()
        .skip_while(|w| STATEMENT_KEYWORDS.contains(w));
    Some((words.next()?, words.next()))
}

/// The first word of every statement in `code` that is one of `words`.
fn count_keywords(code: &str, words: &[&str]) -> usize {
    statements(code)
        .filter_map(|stmt| stmt.split_whitespace().next())
        .filter(|word| words.contains(word))
        .count()
}

// ---------------------------------------------------------------------------
// statement recognisers
// ---------------------------------------------------------------------------

fn apply_line(ctx: &mut Ctx, line: &str, from: usize, block: Option<usize>) {
    if apply_read(ctx, line, from) {
        return;
    }
    if apply_for_loop(ctx, line, from) {
        return;
    }
    apply_assignment(ctx, line, from, block);
}

fn apply_assignment(ctx: &mut Ctx, line: &str, from: usize, block: Option<usize>) -> bool {
    let Some((name, rhs)) = split_assignment(line) else {
        return false;
    };
    let fresh = if is_sanitizer_rhs(rhs) {
        TaintKind::Clean
    } else {
        rhs_taint(ctx, rhs)
    };
    // A *straight-line* reassignment from a literal really does clean a
    // variable, and one from input really does re-taint it. A conditional one
    // cannot lower the taint: the other branch may have run instead.
    let kind = if block.is_some() || assignment_is_guarded(line, &name) {
        merged_kind(ctx, &name, fresh, block)
    } else {
        fresh
    };
    ctx.blocks.insert(name.clone(), block);
    ctx.set(name, kind, from);
    true
}

/// Merge a conditional assignment with the taint the variable already carries,
/// so that the verdict no longer depends on which branch was written last
/// (GH-227: `if …; then P="$1"; else P=/default; fi` lost the taint entirely,
/// and swapping the branches brought it back).
///
/// Two conditions keep the merge honest:
///
/// * the file must have assigned the variable before — a *first* assignment
///   inside an `if` is judged on its own value, or `if …; then D="build"; fi`
///   would become `Ambient` and re-create the false positives GH-227 removed;
/// * the earlier assignment must belong to the same construct, so a reset at
///   the top of a loop body does not inherit a stale value from an unrelated
///   function.
fn merged_kind(ctx: &Ctx, name: &str, fresh: TaintKind, block: Option<usize>) -> TaintKind {
    match ctx.current.get(name) {
        Some(prior) if ctx.blocks.get(name) == Some(&block) => fresh.max(*prior),
        _ => fresh,
    }
}

/// An assignment reached only through `&&`/`||` runs conditionally too.
fn assignment_is_guarded(line: &str, name: &str) -> bool {
    let code = code_only(line);
    let head = match word_pos(&code, name) {
        Some(pos) => code.get(..pos).unwrap_or(""),
        None => code.as_str(),
    };
    head.contains("&&") || head.contains("||")
}

/// For every line, the index of the outermost conditional or loop it sits
/// inside — `None` at top level.
///
/// Heredoc bodies are data, and are not allowed to unbalance the nesting count.
fn block_ids(lines: &[&str], heredoc_body: &HashSet<usize>) -> Vec<Option<usize>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut open: Option<usize> = None;
    let mut depth: usize = 0;
    for (idx, line) in lines.iter().enumerate() {
        if heredoc_body.contains(&(idx + 1)) {
            out.push(open);
            continue;
        }
        let code = code_only(line);
        let opens = count_keywords(&code, BLOCK_OPENERS);
        if open.is_none() && opens > 0 {
            open = Some(idx);
        }
        out.push(open);
        depth = (depth + opens).saturating_sub(count_keywords(&code, BLOCK_CLOSERS));
        if depth == 0 {
            open = None;
        }
    }
    out
}

/// Split `NAME=value`, tolerating `export`/`local`/`readonly` prefixes and
/// case-arm prefixes, and rejecting comparisons (`==`, `!=`), `+=`, and
/// array-element assignment.
fn split_assignment(line: &str) -> Option<(String, &str)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'=' {
            i += 1;
            continue;
        }
        if bytes.get(i + 1) == Some(&b'=') {
            i += 2;
            continue;
        }
        if let Some(name) = name_before(bytes, i) {
            return Some((name, line.get(i + 1..)?));
        }
        i += 1;
    }
    None
}

fn name_before(bytes: &[u8], eq: usize) -> Option<String> {
    let mut start = eq;
    while start > 0 && is_name_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start == eq || bytes[start].is_ascii_digit() {
        return None;
    }
    if start > 0 && !ASSIGN_DELIMS.contains(&bytes[start - 1]) {
        return None;
    }
    Some(slice_string(bytes, start, eq))
}

fn rhs_taint(ctx: &Ctx, rhs: &str) -> TaintKind {
    let from_vars = var_names(rhs)
        .into_iter()
        .map(|name| ctx.get(&name))
        .max()
        .unwrap_or_default();
    from_vars.max(cmd_sub_taint(rhs))
}

/// A command substitution is external when it runs a command that pulls data
/// from off-box.
fn cmd_sub_taint(rhs: &str) -> TaintKind {
    if !rhs.contains("$(") && !rhs.contains('`') {
        return TaintKind::Clean;
    }
    if EXTERNAL_DATA_CMDS
        .iter()
        .any(|cmd| word_pos(rhs, cmd).is_some())
    {
        return TaintKind::External;
    }
    TaintKind::Clean
}

/// Canonicalisation that makes a path safe regardless of where it came from
/// (issue #104).
fn is_sanitizer_rhs(rhs: &str) -> bool {
    rhs.contains("realpath")
        || rhs.contains("readlink -f")
        || rhs.contains("readlink --canonicalize")
}

fn apply_read(ctx: &mut Ctx, line: &str, from: usize) -> bool {
    let Some(args) = read_command_args(line) else {
        return false;
    };
    let masked = mask_quoted(args);
    let head = masked.split([';', '<', '|', '&']).next().unwrap_or("");
    let mut assigned = false;
    let mut skip_next = false;
    for token in head.split_whitespace() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if token.starts_with('-') {
            skip_next = READ_OPTS_WITH_ARG.contains(&token);
            continue;
        }
        if !is_valid_name(token) {
            break;
        }
        ctx.set(token.to_string(), TaintKind::External, from);
        assigned = true;
    }
    assigned
}

/// Replace every quoted region with a single opaque token, so that
/// `read -p "enter a name: " answer` still tokenises as three words.
fn mask_quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut quote: Option<char> = None;
    for c in text.chars() {
        match quote {
            Some(q) if c == q => {
                quote = None;
                out.push('\u{1}');
            }
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None => out.push(c),
        }
    }
    out
}

fn read_command_args(line: &str) -> Option<&str> {
    for cmd in READ_COMMANDS {
        if let Some(pos) = word_pos(line, cmd) {
            return line.get(pos + cmd.len()..);
        }
    }
    None
}

fn apply_for_loop(ctx: &mut Ctx, line: &str, from: usize) -> bool {
    let Some((name, words)) = split_for(line) else {
        return false;
    };
    let kind = rhs_taint(ctx, words);
    ctx.set(name, kind, from);
    true
}

fn split_for(line: &str) -> Option<(String, &str)> {
    let rest = line.trim().strip_prefix("for ")?;
    let mut parts = rest.splitn(2, " in ");
    let name = parts.next()?.trim();
    let words = parts.next()?;
    if !is_valid_name(name) {
        return None;
    }
    Some((name.to_string(), words))
}

// ---------------------------------------------------------------------------
// guard recognition (pre-pass)
// ---------------------------------------------------------------------------

/// `line_index -> variables cleared from the NEXT line onward`.
///
/// Guards inside a function body are ignored: they constrain the function's
/// own parameters, not the whole script.
fn guard_untaints(lines: &[&str], in_function: &HashSet<usize>) -> HashMap<usize, Vec<String>> {
    let mut out: HashMap<usize, Vec<String>> = HashMap::new();
    for idx in 0..lines.len() {
        if in_function.contains(&idx) {
            continue;
        }
        if let Some(var) = inline_guard(lines[idx]) {
            out.entry(idx).or_default().push(var);
            continue;
        }
        if let Some((end, var)) = block_guard(lines, idx) {
            out.entry(end).or_default().push(var);
        }
    }
    out
}

/// A guard written on a single line: `[[ "$V" == *..* ]] && exit 1`, or a
/// one-line `case … esac`.
fn inline_guard(line: &str) -> Option<String> {
    let subject = var_names(line).into_iter().next()?;
    if is_inline_case(line) {
        return case_rejects_traversal(line).then_some(subject);
    }
    if !line_tests_traversal(line) || !line_hard_fails(line) {
        return None;
    }
    Some(subject)
}

fn is_inline_case(line: &str) -> bool {
    let code = bare_code(line);
    code.contains("case ") && code.contains("esac")
}

/// A multi-line `case … esac` or `if … fi` guard. Returns the index of the
/// closing line (the guard dominates everything after it) and the subject.
fn block_guard(lines: &[&str], idx: usize) -> Option<(usize, String)> {
    let trimmed = lines.get(idx)?.trim();
    if trimmed.starts_with("case ") && trimmed.ends_with(" in") {
        return guard_block(lines, idx, "esac", case_rejects_traversal);
    }
    if is_if_opener(trimmed) && line_tests_traversal(trimmed) {
        return guard_block(lines, idx, "fi", then_branch_rejects);
    }
    None
}

fn is_if_opener(trimmed: &str) -> bool {
    trimmed.starts_with("if ") || trimmed.starts_with("elif ")
}

/// Delimit the block opened at `idx` and hand its text to `rejects`.
fn guard_block(
    lines: &[&str],
    idx: usize,
    closer: &str,
    rejects: impl Fn(&str) -> bool,
) -> Option<(usize, String)> {
    let subject = var_names(lines.get(idx)?).into_iter().next()?;
    let end = find_closer(lines, idx, closer)?;
    let body = lines.get(idx..=end)?.join("\n");
    rejects(&body).then_some((end, subject))
}

/// Does some `case` arm both *match* a traversal pattern and abort?
///
/// GH-227: the previous implementation asked whether the block contained a
/// traversal pattern anywhere and a rejection anywhere — two independent
/// `any()`s. A stock dispatcher (`*/*) echo …;; *) exit 1;;`) satisfied both
/// and cleared the taint on its subject. `;;` terminates an arm, so splitting
/// on it puts the pattern and the rejection that must pair in one chunk.
fn case_rejects_traversal(text: &str) -> bool {
    bare_code(text).split(";;").any(arm_rejects_traversal)
}

fn arm_rejects_traversal(arm: &str) -> bool {
    line_is_traversal_case_arm(arm) && text_hard_fails(&code_only(arm))
}

/// Does the branch the traversal test selects abort?
///
/// GH-227: crediting an `exit` from the `else` branch made the *absence* of a
/// rejection on the guarded path look like a guard, so the scan stops at the
/// first `else`/`elif`.
fn then_branch_rejects(text: &str) -> bool {
    for stmt in statements(&code_only(text)) {
        if opens_other_branch(stmt) {
            return false;
        }
        if statement_hard_fails(stmt) {
            return true;
        }
    }
    false
}

fn opens_other_branch(stmt: &str) -> bool {
    matches!(stmt.split_whitespace().next(), Some("else") | Some("elif"))
}

fn find_closer(lines: &[&str], start: usize, closer: &str) -> Option<usize> {
    let last = lines.len().checked_sub(1)?;
    let cap = (start + GUARD_BLOCK_CAP).min(last);
    (start..=cap).find(|idx| lines.get(*idx).is_some_and(|l| line_closes(l, closer)))
}

fn line_closes(line: &str, closer: &str) -> bool {
    let trimmed = line.trim();
    trimmed == closer
        || trimmed.starts_with(&format!("{closer} "))
        || trimmed.ends_with(&format!(" {closer}"))
        || trimmed.ends_with(&format!(";{closer}"))
}

/// Text that looks like a path-traversal pattern.
fn line_has_traversal_pattern(line: &str) -> bool {
    line.contains("..")
        || line.contains("\\.\\.")
        || line.contains("/*")
        || line.contains("realpath")
        || line.contains("readlink")
}

fn line_has_test(line: &str) -> bool {
    line.contains("==")
        || line.contains("!=")
        || line.contains("=~")
        || line.contains("case ")
        || line.contains("grep")
}

/// The line compares something against a traversal pattern.
fn line_tests_traversal(line: &str) -> bool {
    line_has_traversal_pattern(line) && line_has_test(line)
}

/// A `case` arm whose *pattern* is a traversal pattern, e.g. `*..*|/*) exit 1`.
///
/// In a `case` block the comparison is implicit, so `line_has_test` never
/// matches the arm itself — the pattern has to be read out of the arm label.
fn line_is_traversal_case_arm(line: &str) -> bool {
    match line.split_once(')') {
        Some((pattern, _)) => line_has_traversal_pattern(pattern),
        None => false,
    }
}

/// The line abandons the current path. A guard that only prints a message is
/// not a guard. `continue` counts: inside a loop it skips the body, which is
/// the loop-shaped spelling of "reject this input".
///
/// GH-227: the rejection must be a *command*, not a word. Substring matching
/// made `echo "set STRICT=1 to exit on this"` — and `# should we exit here?` —
/// indistinguishable from `exit 1`, so a warn-only path check silenced
/// SEC010/SEC014 for the rest of the file. A control any mention defeats is not
/// a control.
fn line_hard_fails(line: &str) -> bool {
    text_hard_fails(&code_only(line))
}

fn text_hard_fails(code: &str) -> bool {
    statements(code).any(statement_hard_fails)
}

fn statement_hard_fails(stmt: &str) -> bool {
    match command_word(stmt) {
        Some((cmd, arg)) => command_hard_fails(cmd, arg),
        None => false,
    }
}

/// `exit 0` and `return 0` report success — they are not rejections.
fn command_hard_fails(cmd: &str, arg: Option<&str>) -> bool {
    match cmd {
        "exit" => arg != Some("0"),
        "return" => arg.is_some_and(|status| status != "0"),
        "continue" | "die" | "fatal" | "abort" => true,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// validator functions (pre-pass)
// ---------------------------------------------------------------------------

/// Names of functions defined in this file whose body actually validates a
/// path — i.e. tests for a traversal pattern **and** aborts.
///
/// GH-227: the previous implementation trusted the function's *name*, so
/// `validate_path() { :; }` silenced the rule. A control any rename defeats is
/// not a control.
fn collect_validator_functions(lines: &[&str]) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut idx = 0;
    while idx < lines.len() {
        let Some(name) = lines.get(idx).and_then(|l| function_header(l)) else {
            idx += 1;
            continue;
        };
        let end = function_end(lines, idx);
        if lines.get(idx..=end).is_some_and(body_is_path_validator) {
            out.insert(name);
        }
        idx = end + 1;
    }
    out
}

/// GH-227: the same any-line ∧ any-line flaw one level down. A function that
/// *warns* on traversal and *exits* on something unrelated was registered as a
/// validator — the realistic shape of a permissive one. The body must contain a
/// guard that does both, which is exactly what [`inline_guard`] and
/// [`block_guard`] already decide.
fn body_is_path_validator(body: &[&str]) -> bool {
    (0..body.len()).any(|idx| body_line_guards(body, idx))
}

fn body_line_guards(body: &[&str], idx: usize) -> bool {
    match body.get(idx) {
        Some(line) => inline_guard(line).is_some() || block_guard(body, idx).is_some(),
        None => false,
    }
}

/// 0-based indices of every line belonging to a function definition.
fn function_body_lines(lines: &[&str]) -> HashSet<usize> {
    let mut out = HashSet::new();
    let mut idx = 0;
    while idx < lines.len() {
        if lines.get(idx).and_then(|l| function_header(l)).is_none() {
            idx += 1;
            continue;
        }
        let end = function_end(lines, idx);
        out.extend(idx..=end);
        idx = end + 1;
    }
    out
}

/// Name of the function defined on this line, if any.
fn function_header(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let trimmed = match trimmed.strip_prefix("function ") {
        Some(rest) => rest.trim_start(),
        None => trimmed,
    };
    let paren = trimmed.find("()")?;
    let name = trimmed.get(..paren)?.trim();
    if !is_valid_name(name) {
        return None;
    }
    Some(name.to_string())
}

/// Index of the line closing the function opened at `start`.
fn function_end(lines: &[&str], start: usize) -> usize {
    let last = lines.len().saturating_sub(1);
    let cap = (start + FUNCTION_BODY_CAP).min(last);
    let mut depth: i32 = 0;
    for idx in start..=cap {
        depth += brace_delta(lines.get(idx).copied().unwrap_or(""));
        if depth > 0 {
            continue;
        }
        if idx > start || lines.get(idx).is_some_and(|l| l.contains('}')) {
            return idx;
        }
    }
    cap
}

fn brace_delta(line: &str) -> i32 {
    let opens = line.matches('{').count() as i32;
    let closes = line.matches('}').count() as i32;
    opens - closes
}

/// Variable passed to a call of an in-file validator function.
///
/// A call to a function that is **not defined in this file** returns `None`:
/// we cannot read its body, so it is not evidence of anything.
fn validator_call_var(line: &str, validators: &HashSet<String>) -> Option<String> {
    if validators.is_empty() || function_header(line).is_some() {
        return None;
    }
    let trimmed = line.trim();
    let head = trimmed.split_whitespace().next()?;
    if !validators.contains(head) {
        return None;
    }
    var_names(trimmed).into_iter().next()
}

/// Position of `word` in `line` when it appears as a whole word.
fn word_pos(line: &str, word: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(rel) = line.get(from..)?.find(word) {
        let pos = from + rel;
        if word_boundaries_ok(line.as_bytes(), pos, word.len()) {
            return Some(pos);
        }
        from = pos + 1;
    }
    None
}

fn word_boundaries_ok(bytes: &[u8], pos: usize, len: usize) -> bool {
    let before_ok = pos == 0 || !is_name_byte(bytes[pos - 1]);
    let after_ok = match bytes.get(pos + len) {
        None => true,
        Some(b) => !is_name_byte(*b),
    };
    before_ok && after_ok
}

// ---------------------------------------------------------------------------
// KNOWN LIMITATIONS (accepted false negatives, by design)
//
// * Cross-file: variables set by a `source`d file are `Ambient`, never
//   `External` — so they warn, they never break a build.
// * Path insensitivity: a guard inside an `if` branch untaints unconditionally
//   from the block's end. Favours a false negative over a false positive.
// * Assignments are merged, not joined: a conditional assignment takes the
//   maximum of its own value and the variable's prior taint, so the result no
//   longer depends on the textual order of the branches — but there is still no
//   per-branch state, so a variable tainted in one branch reads as tainted in
//   the other one too (over-tainting — conservative).
// * No back edges: a reassignment textually *after* a use is not seen. Loops
//   that re-taint on the second iteration are missed.
// * Function scope is flattened: `$1` inside a function body is treated as the
//   script's `$1` (over-tainting — conservative).
// * Arrays and namerefs are tracked by base name only.
// * `eval` bodies are not analysed; `${!x}` is pessimistically `External`.
// * `X=$(cat /some/file)` is `Clean` — only `EXTERNAL_DATA_CMDS` and nested
//   tainted variables mark a substitution external. Reading a path out of a
//   config file therefore does not taint it; the file's own contents are
//   treated as script-owned data. This is deliberate: the alternative marks
//   every `$(…)` external and reintroduces the noise GH-227 removed.
// * Command substitutions are read one physical line at a time, so a `$(…)`
//   that spans lines — a here-document body inside one, in particular — is
//   invisible to `cmd_sub_taint`: `P=$(cat <<EOF … $1 … EOF)` reads `Clean`.
// * Quoting is not tracked: `'$FOO'` counts as an expansion (over-tainting).
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "taint_tests.rs"]
mod tests;
