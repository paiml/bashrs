//! Shell escaping — the injection boundary every emitted script depends on.
//!
//! # Structure (GH-225)
//!
//! The escaping *rules* live in an allocation-free core that writes into a
//! caller-provided `&mut [u8]`:
//!
//! - [`escape_bytes_len`] — exactly how many bytes the escape will take.
//! - [`escape_bytes_into`] — perform the escape, no heap involved.
//!
//! [`escape_shell_string`] is a thin wrapper that sizes a `Vec` with the former and
//! fills it with the latter. Splitting it this way was not a style preference: the Kani
//! harnesses in `crate::verifier::kani_harnesses` could not converge while `String` was
//! in the call graph, because CBMC then has to model `alloc::raw_vec` / `Layout` /
//! `handle_alloc_error` symbolically (GH-220). The byte entry point also keeps
//! `core::str::validations::run_utf8_validation` out of the proof.
//!
//! Equivalence with the pre-GH-225 implementation is not assumed — it is re-checked on
//! every `cargo test` by `super::escape_differential_tests`, which keeps a frozen copy
//! of the old body as its oracle.

/// A single quote. The byte the whole escaping strategy is organised around.
const SQ: u8 = b'\'';

/// The POSIX idiom for a literal `'` inside a single-quoted word: close, quote a quote
/// with double quotes, reopen. `'` -> `'"'"'`.
const REQUOTE: [u8; 5] = [b'\'', b'"', b'\'', b'"', b'\''];

/// Exactly how many bytes [`escape_bytes_into`] will write for `b`.
///
/// Bounded by `5 * b.len() + 2`, tight exactly when every byte is `'`.
pub fn escape_bytes_len(b: &[u8]) -> usize {
    if b.is_empty() {
        return 2;
    }
    if is_safe_unquoted_bytes(b) {
        return b.len();
    }
    let mut n = 2;
    let mut i = 0;
    while i < b.len() {
        n += if b[i] == SQ { REQUOTE.len() } else { 1 };
        i += 1;
    }
    n
}

/// Write the shell-escaped form of `b` into `out`, allocating nothing.
///
/// Returns the number of bytes written, or `None` if `out.len() < escape_bytes_len(b)`.
/// Capacity is checked up front, so on `None` `out` is not written at all, and on
/// `Some(n)` everything from `out[n..]` is left untouched.
pub fn escape_bytes_into(b: &[u8], out: &mut [u8]) -> Option<usize> {
    if out.len() < escape_bytes_len(b) {
        return None;
    }
    if b.is_empty() {
        out[0] = SQ;
        out[1] = SQ;
        return Some(2);
    }
    if is_safe_unquoted_bytes(b) {
        out[..b.len()].copy_from_slice(b);
        return Some(b.len());
    }
    Some(write_single_quoted(b, out))
}

/// Emit `b` as one single-quoted word, requoting embedded `'`. Caller guarantees
/// `out.len() >= escape_bytes_len(b)`.
///
/// When `b` contains no `'` this is a straight copy inside a quote pair, i.e. exactly
/// the `'{s}'` form — which is why the core needs only one quoting branch where the
/// pre-GH-225 code had two (and a `s.contains('\'')` scan to choose between them).
fn write_single_quoted(b: &[u8], out: &mut [u8]) -> usize {
    let mut w = 0;
    out[w] = SQ;
    w += 1;
    let mut i = 0;
    while i < b.len() {
        if b[i] == SQ {
            out[w..w + REQUOTE.len()].copy_from_slice(&REQUOTE);
            w += REQUOTE.len();
        } else {
            out[w] = b[i];
            w += 1;
        }
        i += 1;
    }
    out[w] = SQ;
    w + 1
}

/// Byte-level form of [`is_safe_unquoted`].
///
/// Exactly equivalent, not approximately: every non-ASCII byte is `>= 0x80`, so it fails
/// `is_ascii_alphanumeric()` and matches none of the eight allowed punctuation bytes —
/// the same `false` the char version reaches via its `!c.is_ascii()` early return. The
/// two are re-checked against each other on every test run.
pub fn is_safe_unquoted_bytes(b: &[u8]) -> bool {
    match b.first() {
        None => false,
        Some(&f) if !is_safe_unquoted_lead(f) => false,
        Some(_) => all_bytes_safe(b),
    }
}

fn all_bytes_safe(b: &[u8]) -> bool {
    let mut i = 0;
    while i < b.len() {
        if !is_safe_unquoted_byte(b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// A word may start unquoted only with these — notably NOT `-`, which would turn the
/// word into an option, and not `~`, which would tilde-expand.
fn is_safe_unquoted_lead(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'/'
}

fn is_safe_unquoted_byte(b: u8) -> bool {
    if !b.is_ascii() || b.is_ascii_control() {
        return false;
    }
    b.is_ascii_alphanumeric() || is_safe_unquoted_punct(b)
}

fn is_safe_unquoted_punct(b: u8) -> bool {
    matches!(b, b'_' | b'.' | b'/' | b'-' | b'+' | b'=' | b':' | b'@')
}

/// Exactly how many bytes escaping `s` will take. `&str` view of [`escape_bytes_len`].
pub fn escape_shell_len(s: &str) -> usize {
    escape_bytes_len(s.as_bytes())
}

/// Escape `s` into a caller-provided buffer. `&str` view of [`escape_bytes_into`].
///
/// The written bytes are valid UTF-8 whenever the input is: multi-byte sequences are
/// copied whole (`0x27` never appears as a continuation byte) and everything inserted is
/// ASCII.
pub fn escape_shell_into(s: &str, out: &mut [u8]) -> Option<usize> {
    escape_bytes_into(s.as_bytes(), out)
}

/// Byte-level form of [`is_valid_shell_identifier`]: `[A-Za-z_][A-Za-z0-9_]*`, non-empty.
///
/// Equivalent to the char version — the predicate is ASCII-only, so any non-ASCII byte
/// fails it exactly as any non-ASCII char fails the char version.
pub fn is_valid_shell_identifier_bytes(b: &[u8]) -> bool {
    match b.first() {
        None => false,
        Some(&f) if !(f.is_ascii_alphabetic() || f == b'_') => false,
        Some(_) => all_ident_tail_bytes_valid(b),
    }
}

fn all_ident_tail_bytes_valid(b: &[u8]) -> bool {
    let mut i = 1;
    while i < b.len() {
        if !(b[i].is_ascii_alphanumeric() || b[i] == b'_') {
            return false;
        }
        i += 1;
    }
    true
}

/// The byte a sanitised identifier carries at position `i`, given `first = (i == 0)`.
fn sanitized_ident_byte(b: u8, first: bool) -> u8 {
    let keep = if first {
        b.is_ascii_alphabetic() || b == b'_'
    } else {
        b.is_ascii_alphanumeric() || b == b'_'
    };
    if keep {
        b
    } else {
        b'_'
    }
}

/// Allocation-free core of [`escape_variable_name`], **for ASCII input only**.
///
/// Writes `name.len()` bytes and returns that count, or `None` if `out` is too small or
/// `name` contains a non-ASCII byte.
///
/// The non-ASCII refusal is deliberate and load-bearing: `escape_variable_name` sanitises
/// per *character*, so `"hello_世界"` becomes `"hello___"` (two CJK chars, two
/// underscores). A byte loop would emit one underscore per byte and produce
/// `"hello______"`. Those cases route to the character path in the wrapper.
///
/// Unlike `escape_variable_name`, this accepts the empty slice (returning `Some(0)`);
/// the caller's `contract_pre_roundtrip!` precondition lives in the wrapper.
pub fn escape_variable_bytes_into(name: &[u8], out: &mut [u8]) -> Option<usize> {
    if out.len() < name.len() || !name.is_ascii() {
        return None;
    }
    if is_valid_shell_identifier_bytes(name) {
        out[..name.len()].copy_from_slice(name);
        return Some(name.len());
    }
    let mut i = 0;
    while i < name.len() {
        out[i] = sanitized_ident_byte(name[i], i == 0);
        i += 1;
    }
    Some(name.len())
}

/// Escape a string for safe use in shell scripts (public alias)
pub fn shell_escape(s: &str) -> String {
    escape_shell_string(s)
}

/// Escape a string for safe use in shell scripts.
///
/// A thin allocating wrapper over [`escape_shell_into`]; the escaping rules themselves
/// live there. Byte-for-byte equivalent to the pre-GH-225 implementation — see
/// `super::escape_differential_tests`, which diffs this against a frozen copy of that
/// implementation on every test run.
pub fn escape_shell_string(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }

    // Contract: encoder-roundtrip-v1.yaml precondition (pv codegen).
    // MUST stay below the is_empty() early return: this macro expands to
    // debug_assert!(!s.is_empty()), so hoisting it would panic on escape_shell_string("")
    // in every debug and test build.
    contract_pre_roundtrip!(s);

    let mut buf = vec![0u8; escape_shell_len(s)];
    let n = escape_shell_into(s, &mut buf).expect("buffer was sized by escape_shell_len");
    buf.truncate(n);
    String::from_utf8(buf)
        .expect("escape_bytes_into copies whole UTF-8 sequences and inserts only ASCII")
}

/// Escape a variable name for shell.
///
/// ASCII input goes through the allocation-free [`escape_variable_bytes_into`]. Non-ASCII
/// input takes the character path below, which emits one byte per *character* — see
/// [`escape_variable_bytes_into`] for why that distinction cannot be flattened.
pub fn escape_variable_name(name: &str) -> String {
    // Contract: encoder-roundtrip-v1.yaml precondition (pv codegen).
    // Note this sits ahead of any emptiness handling and always has: the empty string is
    // outside this function's precondition. Do not move it.
    contract_pre_roundtrip!(name);

    let bytes = name.as_bytes();
    if bytes.is_ascii() {
        let mut buf = vec![0u8; bytes.len()];
        let n = escape_variable_bytes_into(bytes, &mut buf)
            .expect("input is ASCII and the buffer is exactly name.len()");
        buf.truncate(n);
        return String::from_utf8(buf).expect("sanitised identifier bytes are ASCII");
    }

    // Non-ASCII can never be a valid identifier, so this is the sanitising path only.
    sanitize_identifier_chars(name)
}

/// One output byte per input character: keep the character if it is legal at its
/// position, otherwise substitute `_`.
fn sanitize_identifier_chars(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        let keep = if i == 0 {
            // First character must be ASCII letter or underscore (POSIX shell requirement)
            c.is_ascii_alphabetic() || c == '_'
        } else {
            c.is_ascii_alphanumeric() || c == '_'
        };
        result.push(if keep { c } else { '_' });
    }
    result
}

/// Escape a command name for shell execution
pub fn escape_command_name(cmd: &str) -> String {
    // Contract: encoder-roundtrip-v1.yaml precondition (pv codegen)
    contract_pre_roundtrip!(cmd);
    // Commands should not contain special characters
    if is_safe_command_name(cmd) {
        cmd.to_string()
    } else {
        escape_shell_string(cmd)
    }
}

/// Check if a string is safe to use unquoted in shell.
///
/// Rejects anything non-ASCII (bidi overrides, emoji), any control character, and any
/// leading character that a shell would reinterpret (`-` as an option, `~` as a home
/// expansion). Delegates to [`is_safe_unquoted_bytes`] — the two are equivalent, and
/// `test_GH225_is_safe_unquoted_byte_and_char_agree` re-checks that on every run.
///
/// Test-only since GH-225: production now reaches the predicate through the byte core.
/// Kept so the pre-existing `tests::test_safe_unquoted` still exercises the `&str` view.
#[cfg(test)]
fn is_safe_unquoted(s: &str) -> bool {
    is_safe_unquoted_bytes(s.as_bytes())
}

/// Check if a string is a valid POSIX shell identifier (ASCII only).
///
/// Delegates to [`is_valid_shell_identifier_bytes`]; the two are equivalent because the
/// predicate admits ASCII only.
///
/// Test-only since GH-225, for the same reason as [`is_safe_unquoted`].
#[cfg(test)]
fn is_valid_shell_identifier(name: &str) -> bool {
    is_valid_shell_identifier_bytes(name.as_bytes())
}

/// Check if a command name is safe
fn is_safe_command_name(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }

    // Command names should be simple identifiers or paths
    cmd.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
        && !cmd.starts_with('-') // Commands shouldn't start with dash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_simple_string() {
        assert_eq!(escape_shell_string("hello"), "hello");
        assert_eq!(escape_shell_string("hello world"), "'hello world'");
        assert_eq!(escape_shell_string(""), "''");
    }

    #[test]
    fn test_escape_string_with_quotes() {
        assert_eq!(escape_shell_string("don't"), "'don'\"'\"'t'");
    }

    #[test]
    fn test_variable_name_escaping() {
        assert_eq!(escape_variable_name("valid_name"), "valid_name");
        assert_eq!(escape_variable_name("invalid-name"), "invalid_name");
        assert_eq!(escape_variable_name("123invalid"), "_23invalid");
    }

    #[test]
    fn test_command_name_escaping() {
        assert_eq!(escape_command_name("ls"), "ls");
        assert_eq!(escape_command_name("/bin/ls"), "/bin/ls");
        assert_eq!(escape_command_name("my command"), "'my command'");
    }

    #[test]
    fn test_safe_unquoted() {
        assert!(is_safe_unquoted("simple"));
        assert!(is_safe_unquoted("path/to/file"));
        assert!(is_safe_unquoted("version-1.0"));
        assert!(!is_safe_unquoted("has spaces"));
        assert!(!is_safe_unquoted("has$dollar"));
        assert!(!is_safe_unquoted(""));
    }
}
