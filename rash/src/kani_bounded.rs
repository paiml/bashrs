#![cfg(kani)]
//! Bounded symbolic values for Kani harnesses.
//!
//! Kani cannot generate a `String` or `&str`: neither implements
//! `kani::Arbitrary`, and neither can — they are unbounded heap types, and
//! bounded model checking needs a finite state space. Every `let s: String =
//! kani::any();` in this repo was therefore an `E0277`, which is most of the 19
//! errors that made the crate uncompilable under `cfg(kani)` (GH-212). Those
//! harnesses had never been compiled, so nobody found out.
//!
//! The standard model is a fixed-size byte array (which IS `Arbitrary`) plus a
//! symbolic length and an alphabet constraint. `N` is the real cost knob: the
//! solver explores `|alphabet|^N` strings, so keep it small and raise it only
//! with a measured runtime.

/// A symbolic `String` of length `0..=N` over ASCII alphanumerics.
///
/// Pair with `#[kani::unwind(N + 1)]` on the harness — the loop below is what
/// needs the unwind bound, and an unwind that is too low is reported by Kani as
/// an unwinding-assertion failure rather than silently under-approximating.
pub fn any_bounded_string<const N: usize>() -> String {
    let bytes: [u8; N] = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= N);

    let mut s = String::with_capacity(len);
    for &b in bytes.iter().take(len) {
        kani::assume(b.is_ascii_alphanumeric());
        s.push(b as char);
    }
    s
}

/// A symbolic POSIX-ish identifier of length `1..=N`: `[A-Za-z_][A-Za-z0-9_]*`.
///
/// Separate from `any_bounded_string` because the leading character carries a
/// different constraint, and folding that into one function would either
/// over-constrain ordinary strings or under-constrain identifiers.
pub fn any_bounded_identifier<const N: usize>() -> String {
    let bytes: [u8; N] = kani::any();
    let len: usize = kani::any();
    kani::assume(len >= 1 && len <= N);

    let mut s = String::with_capacity(len);
    for (i, &b) in bytes.iter().take(len).enumerate() {
        if i == 0 {
            kani::assume(b.is_ascii_alphabetic() || b == b'_');
        } else {
            kani::assume(b.is_ascii_alphanumeric() || b == b'_');
        }
        s.push(b as char);
    }
    s
}
