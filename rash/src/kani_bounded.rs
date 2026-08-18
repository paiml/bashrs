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

/// A symbolic byte string of length `0..=N` over an **arbitrary** alphabet.
///
/// The caller owns the array, so nothing here allocates. That is the entire point
/// (GH-220 / GH-225): while a generator returns a `String`, `alloc::raw_vec`,
/// `Layout::array` and `handle_alloc_error` are reachable from every harness that uses
/// it, and CBMC models the allocator symbolically — 7.4 GB of RSS and no verdict in
/// 600 s for `verify_escape_safety`.
///
/// The alphabet is unconstrained, which is *stronger* than `any_bounded_string`'s ASCII
/// alphanumerics, not weaker: quotes, metacharacters, control bytes and non-UTF-8 bytes
/// are all in the domain, so a proof over it reaches the escaper's requote branch — the
/// branch an alphanumeric alphabet can never reach.
///
/// Returns the symbolic length; the caller slices `buf[..len]`.
pub fn any_bounded_bytes<const N: usize>(buf: &mut [u8; N]) -> usize {
    *buf = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= N);
    len
}

/// A symbolic byte string of length `1..=N` over the POSIX identifier alphabet:
/// `[A-Za-z_][A-Za-z0-9_]*`. Allocation-free counterpart of `any_bounded_identifier`.
///
/// The constraint is applied to the byte values directly. Do not implement this by
/// indexing an alphabet table with a symbolic `usize` — that makes CBMC reason over
/// 64-bit indices and does not converge.
pub fn any_bounded_identifier_bytes<const N: usize>(buf: &mut [u8; N]) -> usize {
    *buf = kani::any();
    let len: usize = kani::any();
    kani::assume(len >= 1 && len <= N);

    let mut i = 0;
    while i < N {
        if i == 0 {
            kani::assume(buf[i].is_ascii_alphabetic() || buf[i] == b'_');
        } else {
            kani::assume(buf[i].is_ascii_alphanumeric() || buf[i] == b'_');
        }
        i += 1;
    }
    len
}

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
