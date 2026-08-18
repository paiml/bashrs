//! Locating the POSIX `[ … ]` test command, shared by SC1020 and SC1140.
//!
//! GH-226: both rules treated *any* `[` as the start of a test command, so they
//! reported on things that are not tests at all and cannot be:
//!
//! ```sh
//! if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then   # array subscript
//! if [[ "$line" =~ ^[[:space:]]*fn ]]; then      # regex character class
//! case "$mode" in [0-7][0-7][0-7]) ;; esac       # glob character class
//! M["vld1q_f32|vst1q_f32"]="neon:NEON"           # associative array key
//! ```
//!
//! All four produced `Severity::Error` findings, and none has a fix, because
//! there is nothing wrong with them.
//!
//! The discriminator is in POSIX itself: `[` is an ordinary command, so it must
//! be **its own word** — preceded by a command separator and followed by a
//! blank. `[0-7]`, `arr[0]` and `[[` all fail that test, while `[ -f x ]` — and
//! the defective `[ -f x]` these rules exist to catch — pass it.

/// Byte offsets of every `[` on `line` that opens a `[ … ]` test command.
///
/// Three conditions, all necessary:
///
/// 1. `[` begins a word — excludes `arr[0]` and `M["k"]`;
/// 2. `[` sits in COMMAND position — excludes `grep [a b] file`, where the
///    bracket is a glob in an argument;
/// 3. the bracket encloses a blank — excludes `case` glob patterns such as
///    `[0-7][0-7][0-7])`, while still admitting `[-z "$1"]` and `[$x = y]`,
///    which are the two most common novice `test` bugs and fail at runtime
///    with `[-z: command not found`.
pub fn openers(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    command_positions(line)
        .into_iter()
        .filter(|&i| bytes[i] == b'[' && bytes.get(i + 1) != Some(&b'['))
        .filter(|&i| encloses_blank(line, i))
        .collect()
}

/// `[` must enclose a blank to be a test rather than a glob character class.
fn encloses_blank(line: &str, open: usize) -> bool {
    let bytes = line.as_bytes();
    if bytes.get(open + 1).is_some_and(u8::is_ascii_whitespace) {
        return true;
    }
    close_of(line, open)
        .is_some_and(|close| bytes[open + 1..close].iter().any(u8::is_ascii_whitespace))
}

/// Byte offsets at which a word begins a command, on this line.
///
/// A command begins at the start of the line, after a control operator, or
/// after a reserved word that is followed by another command.
fn command_positions(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut expect_command = true;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
        } else if operator_at(bytes, i) {
            expect_command = true;
            i += 1;
        } else {
            let end = word_end(bytes, i);
            if expect_command {
                out.push(i);
                expect_command = leads_a_command(&line[i..end]);
            }
            i = end;
        }
    }
    out
}

/// Is the byte at `i` a control operator here?
///
/// `;`, `&` and `|` always are. `(` and a backtick only when they begin a word:
/// in `[[ $x =~ ^(a|[0-9]+)$ ]]` the parenthesis is part of a regex, and reading
/// it as a subshell made the following `[0-9]` look like a test in command
/// position.
fn operator_at(bytes: &[u8], i: usize) -> bool {
    match bytes[i] {
        b';' | b'&' | b'|' => true,
        b'(' | b'`' => match i.checked_sub(1).map(|p| bytes[p]) {
            None => true,
            Some(b) => b.is_ascii_whitespace() || matches!(b, b';' | b'&' | b'|' | b'('),
        },
        _ => false,
    }
}

fn word_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start + 1;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() && !operator_at(bytes, end) {
        end += 1;
    }
    end
}

/// Reserved words after which another command still follows.
fn leads_a_command(word: &str) -> bool {
    matches!(
        word,
        "if" | "elif" | "while" | "until" | "then" | "do" | "else" | "!" | "{"
    )
}

/// Byte offsets of every `[[` on `line` that opens a bash `[[ … ]]` test.
///
/// `[[ -f x]]` is a genuine syntax error (`bash -n` rejects it), so excluding
/// double brackets from SC1020 entirely lost real coverage.
pub fn double_openers(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    command_positions(line)
        .into_iter()
        .filter(|&i| bytes[i] == b'[' && bytes.get(i + 1) == Some(&b'['))
        .collect()
}

/// Byte offset of the `]]` closing the `[[` opened at `open`, if any.
///
/// The closing bracket must end a word, which is how the `]]` inside a
/// character class — `[[ "$x" =~ ^[[:space:]]*fn ]]` — is skipped: it is
/// followed by `*`, not by a blank.
pub fn double_close_of(line: &str, open: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    (open + 2..bytes.len().saturating_sub(1))
        .find(|&j| bytes[j] == b']' && bytes[j + 1] == b']' && closes_word(bytes, j + 1))
}

/// Byte offset of the `]` closing the test opened at `open`, if any.
///
/// The closing bracket must itself end a word, which is how `${arr[0]}` inside
/// a test is skipped while `[ -f x]` — the missing-space defect — is still
/// found: there the `]` is followed by end of line.
pub fn close_of(line: &str, open: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    (open + 1..bytes.len()).find(|&j| bytes[j] == b']' && closes_word(bytes, j))
}

fn closes_word(bytes: &[u8], j: usize) -> bool {
    match bytes.get(j + 1) {
        None => true,
        Some(b) => b.is_ascii_whitespace() || matches!(b, b';' | b'&' | b'|' | b')'),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_GH226_bracket_plain_test_is_an_opener() {
        assert_eq!(openers("[ -f file.txt ]"), vec![0]);
        assert_eq!(openers("if [ -f x ]; then"), vec![3]);
        assert_eq!(openers("true && [ -f x ]"), vec![8]);
    }

    #[test]
    fn test_GH226_bracket_double_bracket_is_not_an_opener() {
        assert!(openers("[[ -f file.txt ]]").is_empty());
        assert!(openers(r#"if [[ "$line" =~ ^[[:space:]]*fn ]]; then"#).is_empty());
    }

    #[test]
    fn test_GH226_bracket_glob_class_is_not_an_opener() {
        assert!(openers("  [0-7][0-7][0-7]) echo octal ;;").is_empty());
    }

    #[test]
    fn test_GH226_bracket_array_subscript_is_not_an_opener() {
        assert!(openers("echo ${arr[0]}").is_empty());
        assert!(openers(r#"M["vld1q_f32|vst1q_f32"]="neon:NEON""#).is_empty());
    }

    #[test]
    fn test_GH226_bracket_close_skips_subscripts() {
        let line = "[ -n ${arr[0]} ]";
        let open = openers(line)[0];
        assert_eq!(close_of(line, open), Some(line.len() - 1));
    }

    #[test]
    fn test_GH226_bracket_close_finds_the_missing_space_defect() {
        let line = "[ -f file.txt]";
        let open = openers(line)[0];
        assert_eq!(close_of(line, open), Some(13));
    }

    #[test]
    fn test_GH226_bracket_missing_space_after_open_is_still_a_test() {
        // Adversarial review: requiring a blank AFTER `[` excluded the two most
        // common novice test bugs, which used to be reported at Error severity.
        assert_eq!(openers(r#"if [-z "$1"]; then echo usage; fi"#), vec![3]);
        assert_eq!(openers("if [$x = y]; then exit 1; fi"), vec![3]);
    }

    #[test]
    fn test_GH226_bracket_glob_in_argument_position_is_not_a_test() {
        // A bracket carrying a blank is only a test in COMMAND position.
        assert!(openers("grep [a b] file").is_empty());
        assert!(openers("echo [a b]").is_empty());
    }

    #[test]
    fn test_GH226_bracket_double_bracket_defect_is_found() {
        // `[[ -f x]]` is a real bash syntax error; excluding `[[` wholesale
        // from SC1020 lost that coverage.
        let line = "[[ -f x]]";
        assert_eq!(double_openers(line), vec![0]);
        assert_eq!(double_close_of(line, 0), Some(7));
    }

    #[test]
    fn test_GH226_bracket_double_close_skips_character_classes() {
        let line = r#"if [[ "$x" =~ ^[[:space:]]*fn ]]; then :; fi"#;
        let open = double_openers(line)[0];
        let close = double_close_of(line, open).expect("the real ]] must be found");
        assert_eq!(&line[close..close + 2], "]]");
        assert!(
            line.as_bytes()[close - 1].is_ascii_whitespace(),
            "no defect here"
        );
    }

    #[test]
    fn test_GH226_bracket_regex_group_is_not_a_command_separator() {
        // Adversarial review: `(` inside a regex made the following character
        // class look like a test command in command position.
        assert!(openers(r#"    if [[ $line =~ ^\*\*([0-9]+)\..+$ ]]; then :; fi"#).is_empty());
        assert!(openers(
            r#"if [[ "$l" =~ ^[[:space:]]*fn[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*) ]]; then"#
        )
        .is_empty());
    }

    #[test]
    fn test_GH226_bracket_real_subshell_still_opens_a_command() {
        assert_eq!(openers("( [ -f x] )"), vec![2]);
    }

    #[test]
    fn test_GH226_bracket_unclosed_test_returns_none() {
        let line = "[ -f file.txt";
        assert_eq!(close_of(line, openers(line)[0]), None);
    }
}
