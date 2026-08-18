#![cfg(kani)]
//! Kani verification harnesses for RASH
//!
//! These harnesses verify critical safety properties using bounded model
//! checking.
//!
//! ## GH-212: this file did not compile
//!
//! Every harness here was written without ever being built — the crate has not
//! compiled under `cfg(kani)` since these were added, and `make verify-kani`
//! swallowed the failure with `|| true`. Repairing it surfaced three separate
//! problems, only the first of which was a typo:
//!
//! 1. `kani::assert!` is not a macro Kani exports. Kani intercepts the standard
//!    `assert!`/`assert_eq!`, which is what the harnesses should have used.
//! 2. `escape_shell_value`, `is_valid_rust0` and `validate_rust0_ast` do not
//!    exist in this crate and there is no evidence they ever did.
//! 3. `String`/`&str` are not `kani::Arbitrary` and cannot be — see
//!    `crate::kani_bounded`.
//!
//! Two harnesses were also removed rather than repaired, because compiling them
//! would have produced proofs of nothing:
//!
//! - `verify_array_bounds_safety` asserted `true` on one branch and, on the
//!   other, that `format!("if [ {} -lt {} ]; then", ..)` contains `"-lt"`. That
//!   is a property of the format string literal. No code under test was reached.
//! - `verify_parser_soundness` called the real parser on an arbitrary string and
//!   checked it against the two functions that do not exist. Even with those
//!   supplied, a full Rust parser over symbolic input is not tractable under
//!   BMC; a passing version of it would only mean the bound was too small.
//!
//! A harness that cannot fail is worse than a missing one: it consumes the
//! verification budget and reports success.
//!
//! ## GH-220 / GH-225: they compiled, and then never finished — and two of them
//! were asserting the wrong thing
//!
//! Compiling was not enough. Measured with kani 0.67.0: `verify_escape_safety`
//! ran 600 s without emitting a single `VERIFICATION:` line, with cbmc at 7.4 GB
//! RSS and a trace dominated by `alloc::raw_vec::handle_error`,
//! `std::alloc::handle_alloc_error` and `Layout::repeat`. The bottleneck was
//! never the input length — 2, 4 and 8 characters all timed out — it was that
//! every harness called an escaper returning a heap `String`, so CBMC had to
//! model Rust's allocator. GH-225 split the escapers onto caller-provided
//! `&mut [u8]` buffers (`emitter::escape::escape_bytes_into`), and
//! `kani_bounded::any_bounded_bytes` removed the last allocation, the one inside
//! the generator itself.
//!
//! Fixing convergence exposed that two of the three properties were wrong:
//!
//! - **Property 1 was false.** `verify_escape_safety` asserted the result is
//!   single-quoted. `escape_shell_string` returns safe input verbatim and always
//!   has, so the assertion is refuted by 3906 of the 3907 inputs the old
//!   generator produced — only `""` satisfied it. Nobody saw a red harness
//!   because it never converged far enough to report. The honest property is the
//!   disjunction now asserted below, which is also the wording
//!   `contracts/property-invariants-v1.yaml` P-ESC-002 already used.
//! - **The metacharacter oracle was a false-positive oracle.**
//!   `contains_unescaped_metachar` tracked `'` and `\` but not `"`, so after the
//!   `'"'"'` requote idiom it believed it was outside quotes and reported
//!   correct output as unsafe — e.g. `escape("';")` = `''"'"';'`, which a real
//!   `sh` prints back as exactly `';`. Widening the alphabet (which convergence
//!   required, because the old alphanumeric-only alphabet could not produce a
//!   quote and so never reached the requote branch at all) would have turned that
//!   into a spurious refutation of correct code. It is replaced below by
//!   `scan_word`, a POSIX-faithful three-state scanner.
//!
//! Both defects are pinned in ordinary Rust by
//! `emitter::escape_differential_tests`, so they cannot come back with a proof
//! attached.

use crate::emitter::escape::{
    escape_bytes_into, escape_bytes_len, escape_variable_bytes_into, is_safe_unquoted_bytes,
    is_valid_shell_identifier_bytes,
};
use crate::kani_bounded::{any_bounded_bytes, any_bounded_identifier_bytes};

/// Input bound for the escaping harnesses. Raising it is the cost knob; see the
/// measured convergence table in the commit that introduced these.
const N: usize = 4;

/// Always at least `escape_bytes_len(b)` for any `b` of length `N`, because the
/// escape is bounded by `5 * b.len() + 2` (tight when every byte is `'`).
const OUT: usize = 5 * N + 2;

/// Verify shell string escaping prevents injection.
///
/// The alphabet is unconstrained bytes, so quotes, metacharacters and control
/// bytes are all in the domain and the escaper's requote branch is genuinely
/// reachable — unlike the ASCII-alphanumeric domain this harness used before,
/// which could only ever exercise the passthrough branch.
#[kani::proof]
#[kani::unwind(24)]
fn verify_escape_safety() {
    let mut raw = [0u8; N];
    let len = any_bounded_bytes::<N>(&mut raw);
    let input = &raw[..len];

    let mut out = [0u8; OUT];
    let n = match escape_bytes_into(input, &mut out) {
        Some(n) => n,
        None => {
            assert!(false, "5N+2 is always a large enough buffer");
            return;
        }
    };
    let escaped = &out[..n];

    // P0: the escape of anything, including the empty string, is a non-empty word.
    assert!(n >= 1);

    // P0': the length the caller was promised is the length that was written.
    assert!(n == escape_bytes_len(input));

    // P1: either the word needs no quoting at all, or it is fully single-quoted.
    // (The old harness asserted only the right disjunct — see the module docs.)
    assert!(
        is_safe_unquoted_bytes(escaped)
            || (n >= 2 && escaped[0] == b'\'' && escaped[n - 1] == b'\'')
    );

    // P2: no metacharacter is reachable by the shell, and quoting is balanced.
    assert!(scan_word(escaped) == Some(QState::Unquoted));

    // P4: an already-safe word is not gratuitously rewritten.
    if is_safe_unquoted_bytes(input) {
        assert!(escaped == input);
    }
}

/// Verify escaping is lossless: `unescape(escape(s)) == s` for every `s`.
///
/// Split from `verify_escape_safety` so the round-trip's own unwind cost is
/// visible and budgeted separately, not so it is optional.
#[kani::proof]
#[kani::unwind(24)]
fn verify_escape_roundtrip() {
    let mut raw = [0u8; N];
    let len = any_bounded_bytes::<N>(&mut raw);
    let input = &raw[..len];

    let mut out = [0u8; OUT];
    let n = match escape_bytes_into(input, &mut out) {
        Some(n) => n,
        None => {
            assert!(false, "5N+2 is always a large enough buffer");
            return;
        }
    };

    let mut back = [0u8; OUT];
    let m = unescape_into(&out[..n], &mut back);
    assert!(m == input.len());
    assert!(&back[..m] == input);
}

/// Verify the buffer contract: one byte short is refused, and a refused call
/// leaves the caller's buffer untouched.
///
/// This is what makes the allocating wrapper safe to write as
/// `escape_shell_into(..).expect(..)`.
#[kani::proof]
#[kani::unwind(24)]
fn verify_escape_buffer_contract() {
    let mut raw = [0u8; N];
    let len = any_bounded_bytes::<N>(&mut raw);
    let input = &raw[..len];

    let need = escape_bytes_len(input);
    let mut buf = [0xAAu8; OUT];
    assert!(escape_bytes_into(input, &mut buf[..need - 1]).is_none());

    let mut i = 0;
    while i < OUT {
        assert!(buf[i] == 0xAA);
        i += 1;
    }
}

/// Verify that variable-name escaping accepts every valid identifier.
///
/// The original asserted that `format!("\"${{{}}}\"", name)` starts with `"` and
/// contains `"${"` — true of the literal regardless of `name`, so it held even
/// if `escape_variable_name` were the identity function. Those two lines are
/// gone; what remains calls the real sanitiser the emitter uses.
#[kani::proof]
#[kani::unwind(6)]
fn verify_variable_expansion_safety() {
    let mut raw = [0u8; N];
    let len = any_bounded_identifier_bytes::<N>(&mut raw);
    let name = &raw[..len];

    // The generator's constraint and the crate's predicate must be the same rule.
    assert!(is_valid_shell_identifier_bytes(name));

    let mut out = [0u8; N];
    let n = match escape_variable_bytes_into(name, &mut out) {
        Some(n) => n,
        None => {
            assert!(false, "ASCII identifier, buffer of exactly name.len()");
            return;
        }
    };

    // A valid identifier must survive escaping unchanged — if it does not, the
    // emitter is rewriting names the user chose.
    assert!(n == len);
    assert!(&out[..n] == name);
}

/// Verify no injection is possible through an escaped argument.
///
/// This is a corollary of P2 in `verify_escape_safety`, not an independent
/// result: it re-asserts the same `scan_word` property with the escaped word
/// placed in real command position. It is kept because it is the property a
/// reader actually cares about, and because it also pins that escaping writes
/// nothing outside the region it was given.
#[kani::proof]
#[kani::unwind(29)]
fn verify_injection_safety() {
    let mut raw = [0u8; N];
    let len = any_bounded_bytes::<N>(&mut raw);
    let input = &raw[..len];

    let mut ctx = [0u8; 5 + OUT];
    ctx[..5].copy_from_slice(b"echo ");
    let n = match escape_bytes_into(input, &mut ctx[5..]) {
        Some(n) => n,
        None => {
            assert!(false, "5N+2 is always a large enough buffer");
            return;
        }
    };

    // The command word was not clobbered — escaping wrote only into the region it
    // was handed ...
    assert!(&ctx[..5] == b"echo ");
    // ... and everything after the separator is ONE balanced word containing no
    // reachable metacharacter, so nothing in the user's input can start a second
    // word, let alone reach command position.
    //
    // The scan starts at 5, not 0: the space in "echo " is a real word separator and
    // `scan_word` would (correctly) reject it. Scanning the whole line instead is how
    // the first version of this harness was written, and Kani refuted it in 12 s —
    // which is the harness working, not the escaper failing.
    assert!(scan_word(&ctx[5..5 + n]) == Some(QState::Unquoted));
}

/// Quoting state of a POSIX shell word scanner.
#[derive(PartialEq, Eq, Clone, Copy)]
enum QState {
    Unquoted,
    Single,
    Double,
}

/// Is `b` a byte the shell acts on when it is not quoted?
fn is_metachar(b: u8) -> bool {
    matches!(
        b,
        b';' | b'&'
            | b'|'
            | b'`'
            | b'$'
            | b'('
            | b')'
            | b'<'
            | b'>'
            | b'\n'
            | b' '
            | b'\t'
            | b'*'
            | b'?'
            | b'['
            | b'{'
            | b'~'
            | b'#'
            | b'!'
    )
}

/// Scan `s` as the shell would, returning the quoting state it ends in, or `None`
/// if a metacharacter is reachable.
///
/// Replaces `contains_unescaped_metachar`, which tracked `'` and `\` but not `"`
/// and therefore reported correct `'"'"'`-requoted output as unsafe. Asserting
/// `Some(QState::Unquoted)` is strictly stronger than "no reachable metachar": it
/// also requires every quote to be closed.
fn scan_word(s: &[u8]) -> Option<QState> {
    let mut st = QState::Unquoted;
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        i += 1;
        // A backslash escapes the next byte everywhere except inside '...',
        // where it is an ordinary character.
        if st != QState::Single && c == b'\\' {
            i += 1;
            continue;
        }
        st = match st {
            QState::Single => step_single(c),
            QState::Double => step_double(c)?,
            QState::Unquoted => step_unquoted(c)?,
        };
    }
    Some(st)
}

/// Inside `'...'` nothing is special except the closing quote.
fn step_single(c: u8) -> QState {
    if c == b'\'' {
        QState::Unquoted
    } else {
        QState::Single
    }
}

/// Inside `"..."` expansion still happens, so `$` and `` ` `` remain live.
fn step_double(c: u8) -> Option<QState> {
    if c == b'"' {
        Some(QState::Unquoted)
    } else if c == b'$' || c == b'`' {
        None
    } else {
        Some(QState::Double)
    }
}

/// Outside quotes every metacharacter is live.
fn step_unquoted(c: u8) -> Option<QState> {
    if c == b'\'' {
        Some(QState::Single)
    } else if c == b'"' {
        Some(QState::Double)
    } else if is_metachar(c) {
        None
    } else {
        Some(QState::Unquoted)
    }
}

/// Invert `escape_bytes_into`, for the round-trip property. Allocation-free.
fn unescape_into(inp: &[u8], out: &mut [u8]) -> usize {
    if !(inp.len() >= 2 && inp[0] == b'\'' && inp[inp.len() - 1] == b'\'') {
        out[..inp.len()].copy_from_slice(inp);
        return inp.len();
    }
    let inner = &inp[1..inp.len() - 1];
    let mut w = 0;
    let mut i = 0;
    while i < inner.len() {
        if is_requote_at(inner, i) {
            out[w] = b'\'';
            w += 1;
            i += 5;
        } else {
            out[w] = inner[i];
            w += 1;
            i += 1;
        }
    }
    w
}

/// Does the `'"'"'` requote idiom start at `inner[i]`?
fn is_requote_at(inner: &[u8], i: usize) -> bool {
    i + 5 <= inner.len()
        && inner[i] == b'\''
        && inner[i + 1] == b'"'
        && inner[i + 2] == b'\''
        && inner[i + 3] == b'"'
        && inner[i + 4] == b'\''
}
