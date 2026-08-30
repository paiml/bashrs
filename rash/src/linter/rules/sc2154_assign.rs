//! Which names does one physical line assign?
//!
//! GH-275. SC2154 used to answer this with
//!
//! ```text
//! ^\s*(?:(?:local|readonly|export|declare|typeset)(?:\s+-[a-zA-Z]+)?\s+)?([A-Za-z_][A-Za-z0-9_]*)=
//! ```
//!
//! anchored at `^`, so `captures_iter` could match at most once per line. A
//! `;`-, `&&`- or `||`-separated list registered only its first assignment,
//! `local a=1 b=2` registered only `a`, and a one-line function body registered
//! nothing at all — the anchor never got past `f() {`. Everything it missed came
//! back as "referenced but not assigned": 143 findings on the rmedia corpus,
//! where `shellcheck` reports none.
//!
//! Deleting the anchor is the wrong fix. An unanchored `([A-Za-z_]\w*)=` reads
//! `grep --color=auto` as an assignment to `color` and `echo "x=1"` as one to
//! `x`, so SC2154 would go blind on those two variables — a false negative
//! bought with a false positive, which for a linter is the worse direction.
//!
//! What makes `NAME=` an assignment is its POSITION: the shell honours it only
//! at the start of a simple command, or in the assignment-prefix run that leads
//! one. So this scans the line the way the shell reads it — tracking quoting and
//! expansions, and tracking where each command begins — and reports a name only
//! from a word the shell would itself treat as an assignment.

/// Commands after which the remaining words are still declarations, so `NAME=`
/// keeps its meaning: `local a=1 b=2`, `declare -i n=1 m=2`.
const DECLARATION_BUILTINS: &[&str] = &["local", "readonly", "export", "declare", "typeset"];

/// Reserved words that introduce a command rather than being one, so the word
/// after them still begins a simple command: `if true; then c=1; fi`.
const COMMAND_INTRODUCERS: &[&str] = &[
    "then", "else", "elif", "do", "if", "while", "until", "!", "time", "{", "}",
];

/// One lexed unit of a line.
enum Token {
    /// A word, with quoted stretches replaced by NUL so their contents cannot
    /// be mistaken for syntax.
    Word(String),
    /// `;`, `&`, `|`, `(`, `)`, `` ` `` — the next word starts a new command.
    Separator,
}

/// The name assigned by `word`, if the shell would read it as an assignment.
///
/// Accepts `NAME=…`, `NAME+=…` and `NAME[subscript]=…`. Rejects a word whose `=`
/// is not preceded by a whole valid name — `--color=auto`, `-o=x`, `=1`, `2x=1`.
pub fn assigned_name(word: &str) -> Option<&str> {
    let bytes = word.as_bytes();

    if !bytes
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
    {
        return None;
    }
    let mut i = 0;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    let name = &word[..i];

    // An optional array subscript: `q[0]=1`, `map[$k]=v`.
    if bytes.get(i) == Some(&b'[') {
        match word[i..].find(']') {
            Some(close) => i += close + 1,
            None => return None,
        }
    }
    // An optional append: `p+=2`.
    if bytes.get(i) == Some(&b'+') {
        i += 1;
    }

    (bytes.get(i) == Some(&b'=')).then_some(name)
}

/// Consume a `${…}` / `$(…)` expansion starting at `bytes[i]` (which is `$`),
/// returning the index just past it. Expansions are word text, not separators:
/// the `}` in `repo=${1:-}` does not end a command.
fn skip_expansion(bytes: &[u8], i: usize) -> usize {
    let (open, close) = match bytes.get(i + 1) {
        Some(b'{') => (b'{', b'}'),
        Some(b'(') => (b'(', b')'),
        _ => return i + 1,
    };
    let mut depth = 0usize;
    let mut j = i + 1;
    while j < bytes.len() {
        if bytes[j] == open {
            depth += 1;
        } else if bytes[j] == close {
            depth -= 1;
            if depth == 0 {
                return j + 1;
            }
        }
        j += 1;
    }
    bytes.len()
}

/// Consume a quoted region starting at `bytes[i]` (the opening quote),
/// returning the index just past the closing quote.
fn skip_quoted(bytes: &[u8], i: usize) -> usize {
    let quote = bytes[i];
    let mut j = i + 1;
    while j < bytes.len() {
        if bytes[j] == b'\\' && quote == b'"' {
            j += 2;
            continue;
        }
        if bytes[j] == quote {
            return j + 1;
        }
        j += 1;
    }
    bytes.len()
}

/// Lex one physical line into words and command separators.
///
/// Quoted stretches are collapsed to a NUL placeholder rather than scanned: the
/// `=` in `echo "x=1"` is text, and reading it as an assignment is precisely the
/// false negative this module exists to prevent. The placeholder is kept (rather
/// than dropped) so `x"=1"` stays a single word that does not look like a bare
/// `x=` assignment.
fn lex(line: &str) -> Vec<Token> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut i = 0;

    macro_rules! flush {
        () => {
            if !cur.is_empty() {
                out.push(Token::Word(std::mem::take(&mut cur)));
            }
        };
    }

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                cur.push('\\');
                i += 2;
            }
            b'\'' | b'"' => {
                let end = skip_quoted(bytes, i);
                cur.push('\u{0}');
                i = end;
            }
            b'$' => {
                let end = skip_expansion(bytes, i);
                cur.push_str(&line[i..end]);
                i = end;
            }
            // A trailing comment, but only where a word could have started:
            // the `#` in `curl http://x#y` is word text.
            b'#' if cur.is_empty() => break,
            b' ' | b'\t' => {
                flush!();
                i += 1;
            }
            b';' | b'&' | b'|' | b'(' | b')' | b'`' => {
                flush!();
                out.push(Token::Separator);
                i += 1;
            }
            // `{` and `}` are reserved WORDS, not metacharacters: they separate
            // commands only when they stand alone. Inside a word they are brace
            // expansion (`cp f{,.bak}`) and must not split it.
            b'{' | b'}' if cur.is_empty() => {
                let next = bytes.get(i + 1);
                if next.is_none() || matches!(next, Some(b' ' | b'\t' | b';')) {
                    out.push(Token::Separator);
                } else {
                    cur.push(bytes[i] as char);
                }
                i += 1;
            }
            // Redirections end a word without starting a new command.
            b'<' | b'>' => {
                flush!();
                i += 1;
            }
            b => {
                cur.push(b as char);
                i += 1;
            }
        }
    }
    flush!();
    out
}

/// `mapfile`/`readarray` options that consume the word after them, so the array
/// name is not mistaken for an option argument. `-t` takes none.
const MAPFILE_OPTS_WITH_ARG: &[&str] = &["-d", "-n", "-O", "-s", "-u", "-C", "-c"];

/// The array `mapfile`/`readarray` fills, given the words that follow it.
///
/// `mapfile -t names < <(…)` assigns `names`; with no name given it assigns
/// `MAPFILE`, which is already a known builtin, so `None` is the right answer
/// there too.
fn mapfile_target(rest: &[&str]) -> Option<String> {
    let mut i = 0;
    while i < rest.len() {
        let w = rest[i];
        if MAPFILE_OPTS_WITH_ARG.contains(&w) {
            i += 2;
        } else if w.starts_with('-') && w.len() > 1 {
            i += 1;
        } else {
            return is_name(w).then(|| w.to_string());
        }
    }
    None
}

/// Is `w` a bare, valid variable name?
fn is_name(w: &str) -> bool {
    w.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        && w.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// What one word in command position does to the scan.
enum Step {
    /// It assigns this name, and the command position stays open.
    Assigns(String),
    /// It keeps the command position open but assigns nothing
    /// (`local`, `then`, a flag to a declaration builtin).
    KeepsOpen,
    /// It is the command word: the position closes until the next separator.
    Closes,
}

/// The words of the command `tokens[from..]` begins, up to its separator.
fn command_words(tokens: &[Token], from: usize) -> Vec<&str> {
    tokens[from..]
        .iter()
        .map_while(|t| match t {
            Token::Word(w) => Some(w.as_str()),
            Token::Separator => None,
        })
        .collect()
}

/// Classify `word`, which is known to sit in command position.
fn classify(word: &str, in_declaration: bool, rest: &[&str]) -> Step {
    if let Some(name) = assigned_name(word) {
        // An assignment prefix does not consume the command position:
        // `A=1 B=2 cmd` assigns twice.
        return Step::Assigns(name.to_string());
    }
    if DECLARATION_BUILTINS.contains(&word) || COMMAND_INTRODUCERS.contains(&word) {
        return Step::KeepsOpen;
    }
    // `declare -i n=1` — a flag to a declaration builtin keeps it open.
    if in_declaration && word.starts_with('-') {
        return Step::KeepsOpen;
    }
    // `mapfile`/`readarray` name their array as a plain argument rather than
    // with `=`, and only when it is the command word: the `mapfile` inside
    // `echo 'use mapfile -t out'` is a quoted message the lexer has already
    // collapsed to a placeholder.
    if matches!(word, "mapfile" | "readarray") {
        if let Some(name) = mapfile_target(rest) {
            return Step::Assigns(name);
        }
    }
    Step::Closes
}

/// Every variable name `line` assigns.
///
/// Known and accepted imprecision: an array literal whose ELEMENTS look like
/// assignments (`arr=(x=1 y=2)`) also registers `x` and `y`. It is rare, and it
/// can only make SC2154 quieter on those two names — never on anything else.
pub fn line_assignments(line: &str) -> Vec<String> {
    let tokens = lex(line);
    let mut assigned = Vec::new();
    let mut at_command_start = true;
    let mut in_declaration = false;

    for (i, token) in tokens.iter().enumerate() {
        let word = match token {
            Token::Separator => {
                at_command_start = true;
                in_declaration = false;
                continue;
            }
            Token::Word(w) => w.as_str(),
        };
        if !at_command_start {
            continue;
        }

        match classify(word, in_declaration, &command_words(&tokens, i + 1)) {
            Step::Assigns(name) => assigned.push(name),
            Step::KeepsOpen => in_declaration |= DECLARATION_BUILTINS.contains(&word),
            Step::Closes => {
                at_command_start = false;
                in_declaration = false;
            }
        }
    }
    assigned
}

#[cfg(test)]
#[path = "sc2154_assign_tests.rs"]
mod tests;
