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
pub fn openers(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len())
        .filter(|&i| bytes[i] == b'[' && starts_word(bytes, i) && ends_word(bytes, i))
        .collect()
}

/// `[` must begin a word: start of line, or after a blank or a control operator.
fn starts_word(bytes: &[u8], i: usize) -> bool {
    match i.checked_sub(1).map(|p| bytes[p]) {
        None => true,
        Some(b) => b.is_ascii_whitespace() || matches!(b, b';' | b'&' | b'|' | b'(' | b'`'),
    }
}

/// `[` must end a word: a blank must follow. This is what excludes `[[`, glob
/// character classes and array subscripts.
fn ends_word(bytes: &[u8], i: usize) -> bool {
    bytes.get(i + 1).is_some_and(u8::is_ascii_whitespace)
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
    fn test_GH226_bracket_unclosed_test_returns_none() {
        let line = "[ -f file.txt";
        assert_eq!(close_of(line, openers(line)[0]), None);
    }
}
