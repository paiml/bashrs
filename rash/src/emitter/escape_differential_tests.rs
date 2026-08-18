//! Differential and witness tests for the shell-escaping boundary (GH-225, GH-220).
//!
//! Two jobs, both load-bearing:
//!
//! 1. **Witness tests.** `rash/src/verifier/kani_harnesses.rs` asserted two things about
//!    `escape_shell_string` that are simply not true of it. Nobody found out because the
//!    harnesses never converged far enough to report a verdict (GH-220). The
//!    `..._legacy_...` tests below pin those falsehoods in ordinary Rust so they cannot
//!    quietly come back with a "proof" attached.
//!
//! 2. **Frozen-oracle differential.** `reference()` is a verbatim copy of the body
//!    `escape_shell_string` had before GH-225 split it onto an allocation-free core. It is
//!    the oracle and it never changes: every future edit to the core is diffed against
//!    pre-GH-225 behaviour on every `cargo test`. This is the shell-injection boundary —
//!    a silent behaviour change here is a security defect, not a bug.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::escape::{
    escape_bytes_into, escape_bytes_len, escape_shell_into, escape_shell_len, escape_shell_string,
    escape_variable_bytes_into, escape_variable_name, is_safe_unquoted_bytes,
    is_valid_shell_identifier_bytes,
};

// ---------------------------------------------------------------------------
// Frozen reference — the pre-GH-225 implementation, verbatim. DO NOT EDIT.
// ---------------------------------------------------------------------------

/// The body of `escape_shell_string` as it stood at 6.66.3, minus the
/// `contract_pre_roundtrip!` debug-assert (which is a precondition on the caller,
/// not part of the transformation).
fn reference(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if reference_is_safe_unquoted(s) {
        return s.to_string();
    }
    if !s.contains('\'') {
        return format!("'{s}'");
    }
    let escaped = s.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

/// The pre-GH-225 `is_safe_unquoted`, verbatim. DO NOT EDIT.
fn reference_is_safe_unquoted(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first_char = match s.chars().next() {
        Some(c) => c,
        None => return false,
    };
    if !first_char.is_ascii_alphanumeric()
        && first_char != '_'
        && first_char != '.'
        && first_char != '/'
    {
        return false;
    }
    s.chars().all(|c| {
        if !c.is_ascii() {
            return false;
        }
        if c.is_ascii_control() {
            return false;
        }
        c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-' | '+' | '=' | ':' | '@')
    })
}

// ---------------------------------------------------------------------------
// The legacy Kani oracle, kept as a witness. DO NOT EDIT.
// ---------------------------------------------------------------------------

/// `contains_unescaped_metachar` as it stood in `kani_harnesses.rs` before GH-225.
///
/// It tracks `'` and `\` but not `"`, so after the `'"'"'` requote idiom it believes it
/// is outside quotes. Kept only so `test_GH225_legacy_metachar_oracle_false_positive`
/// can pin the defect.
fn legacy_contains_unescaped_metachar(s: &str) -> bool {
    let mut in_quotes = false;
    let mut escaped = false;

    for ch in s.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '\'' => in_quotes = !in_quotes,
            ';' | '&' | '|' | '`' | '$' | '(' | ')' | '<' | '>' | '\n' => {
                if !in_quotes {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// The exact domain `any_bounded_string::<2>()` generates: ASCII alphanumerics,
/// length 0..=2. 3907 strings.
fn legacy_harness_domain() -> Vec<String> {
    let alnum: Vec<char> = (0u8..=127)
        .filter(|b| b.is_ascii_alphanumeric())
        .map(|b| b as char)
        .collect();
    let mut out = vec![String::new()];
    for &a in &alnum {
        out.push(a.to_string());
        for &b in &alnum {
            out.push(format!("{a}{b}"));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// WITNESS TESTS — these describe defects in the pre-GH-225 harness, not in the
// escaper. They pass against the old escaper and must keep passing against the new
// one, because the escaper's behaviour is correct and the harness was wrong.
// ---------------------------------------------------------------------------

/// GH-220/GH-225: `verify_escape_safety` Property 1 —
/// `escaped.starts_with('\'') && escaped.ends_with('\'')` — is FALSE on 3906 of the
/// 3907 inputs its own generator produces. Only `""` satisfies it.
#[test]
fn test_GH225_legacy_kani_property1_is_false_on_its_own_domain() {
    let domain = legacy_harness_domain();
    assert_eq!(domain.len(), 3907, "generator domain size changed");

    let violations: Vec<&String> = domain
        .iter()
        .filter(|s| {
            let e = escape_shell_string(s);
            !(e.starts_with('\'') && e.ends_with('\''))
        })
        .collect();

    assert_eq!(
        violations.len(),
        3906,
        "old Property 1 should be refuted by all but the empty string"
    );
    // The smallest counterexample, pinned.
    assert_eq!(escape_shell_string("0"), "0");
    assert_eq!(escape_shell_string(""), "''");
}

/// GH-225: the legacy generator alphabet (ASCII alphanumerics only) cannot reach the
/// security-critical requote branch, so a converging proof over it would have proved the
/// passthrough branch and nothing else. Guard against silently narrowing it again.
#[test]
fn test_GH225_legacy_harness_domain_never_reaches_requote_branch() {
    let reached = legacy_harness_domain()
        .iter()
        .filter(|s| escape_shell_string(s).contains("'\"'\"'"))
        .count();
    assert_eq!(reached, 0, "alphanumeric alphabet cannot contain a quote");

    // Whereas a quote-bearing input does reach it:
    assert_eq!(escape_shell_string("'"), "''\"'\"''");
}

/// GH-225: the legacy `contains_unescaped_metachar` oracle reports a false positive on
/// correct, safe output — because it does not track double quotes. Widening the harness
/// alphabet without replacing the oracle would have produced a spurious refutation.
///
/// The second half of the test shows the same strings are genuinely safe by running them
/// through a real `sh`.
#[test]
fn test_GH225_legacy_metachar_oracle_false_positive() {
    for input in ["';", "a'b;c", "';rm -rf /"] {
        let escaped = escape_shell_string(input);
        assert!(
            legacy_contains_unescaped_metachar(&escaped),
            "legacy oracle was expected to false-positive on {escaped:?}"
        );
        assert_eq!(
            sh_printf(&escaped),
            Some(input.to_string()),
            "but a real shell says {escaped:?} is exactly {input:?}"
        );
    }
}

/// Run `sh -c "printf '%s' <word>"` and return what the shell produced, or `None` if
/// `sh` is unavailable or the command failed.
fn sh_printf(word: &str) -> Option<String> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("printf '%s' {word}"))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

// ---------------------------------------------------------------------------
// Corpus construction (shared by the differential and shell-roundtrip tests)
// ---------------------------------------------------------------------------

/// Every ASCII string of length 0..=2: `""` + 128 singles + 16384 pairs = 16513.
fn corpus_ascii_pairs() -> Vec<String> {
    let mut v = Vec::with_capacity(16_513);
    v.push(String::new());
    for a in 0u8..=127 {
        v.push((a as char).to_string());
        for b in 0u8..=127 {
            v.push(format!("{}{}", a as char, b as char));
        }
    }
    v
}

/// The adversarial alphabet: every character class that can change the escaper's mind.
const ADVERSARIAL: [char; 30] = [
    '\'', '"', '\\', '$', '`', ';', '|', '&', '(', ')', '<', '>', ' ', '\n', '\t', '\0', 'a', '0',
    '_', '-', '/', '@', '=', ':', '.', '+', 'é', '\u{202E}', '🚀', '\u{7f}',
];

/// Every string of length 0..=3 over `ADVERSARIAL`: 1 + 30 + 900 + 27000 = 27931.
fn corpus_adversarial() -> Vec<String> {
    fn rec(depth: usize, cur: &mut String, out: &mut Vec<String>) {
        out.push(cur.clone());
        if depth == 0 {
            return;
        }
        for &c in ADVERSARIAL.iter() {
            cur.push(c);
            rec(depth - 1, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::with_capacity(27_931);
    rec(3, &mut String::new(), &mut out);
    out
}

/// Deterministic xorshift64 Unicode fuzz — same seed every run, so a failure reproduces.
fn corpus_unicode_fuzz(count: usize) -> Vec<String> {
    let mut state: u64 = 0x243F_6A88_85A3_08D3;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    (0..count)
        .map(|_| {
            let len = (next() % 12) as usize;
            (0..len)
                .map(|_| char::from_u32((next() % 0x11000) as u32).unwrap_or('?'))
                .collect()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// GH-225 unit tests — expected values, not round-trips
// ---------------------------------------------------------------------------

#[test]
fn test_GH225_escape_empty_yields_two_quotes() {
    assert_eq!(escape_shell_string(""), "''");
    assert_eq!(escape_shell_len(""), 2);
    assert_eq!(escape_bytes_len(b""), 2);
}

#[test]
fn test_GH225_escape_safe_passthrough() {
    for s in [
        "a",
        "ab",
        "/usr/bin/env",
        "file.txt",
        "a@b:c=d+e",
        "v1.0-rc1",
    ] {
        assert_eq!(
            escape_shell_string(s),
            s,
            "{s:?} must pass through unquoted"
        );
        assert_eq!(escape_shell_len(s), s.len());
        assert!(is_safe_unquoted_bytes(s.as_bytes()));
    }
}

#[test]
fn test_GH225_escape_space_is_quoted() {
    assert_eq!(escape_shell_string("a b"), "'a b'");
    assert_eq!(escape_shell_len("a b"), 5);
}

#[test]
fn test_GH225_escape_single_quote_requoted() {
    assert_eq!(escape_shell_string("don't"), "'don'\"'\"'t'");
    assert_eq!(escape_shell_len("don't"), 11);
}

#[test]
fn test_GH225_escape_all_quotes_hits_worst_case() {
    assert_eq!(escape_shell_string("'"), "''\"'\"''");
    assert_eq!(escape_shell_len("'"), 7);
    assert_eq!(escape_shell_len("'"), 5 + 2);

    assert_eq!(escape_shell_string("''"), "''\"'\"''\"'\"''");
    assert_eq!(escape_shell_len("''"), 12);
    assert_eq!(escape_shell_len("''"), 5 * 2 + 2);
}

#[test]
fn test_GH225_escape_metachars_quoted() {
    assert_eq!(escape_shell_string("$(id)"), "'$(id)'");
    assert_eq!(escape_shell_string("a;b"), "'a;b'");
    assert_eq!(escape_shell_string("`x`"), "'`x`'");
    assert_eq!(escape_shell_string("a|b"), "'a|b'");
    assert_eq!(escape_shell_string("a&b"), "'a&b'");
}

#[test]
fn test_GH225_escape_leading_dash_is_quoted() {
    assert_eq!(escape_shell_string("-x"), "'-x'");
    assert_eq!(escape_shell_len("-x"), 4);
    assert_eq!(escape_shell_string("--flag"), "'--flag'");
    assert_eq!(escape_shell_len("--flag"), 8);
}

#[test]
fn test_GH225_escape_control_and_nul() {
    assert_eq!(escape_shell_string("x\ny"), "'x\ny'");
    assert_eq!(escape_shell_len("x\ny"), 5);
    assert_eq!(escape_shell_string("\0"), "'\0'");
    assert_eq!(escape_shell_len("\0"), 3);
}

#[test]
fn test_GH225_escape_non_ascii_quoted() {
    assert_eq!(escape_shell_string("é"), "'é'");
    assert_eq!(escape_shell_len("é"), 4);
    assert_eq!(escape_shell_string("🚀"), "'🚀'");
    assert_eq!(escape_shell_len("🚀"), 6);
    assert_eq!(escape_shell_string("test\u{202E}exe"), "'test\u{202E}exe'");
    assert_eq!(escape_shell_len("test\u{202E}exe"), 12);
}

#[test]
fn test_GH225_escape_quote_plus_metachar() {
    assert_eq!(escape_shell_string("a'b;c"), "'a'\"'\"'b;c'");
    assert_eq!(escape_shell_len("a'b;c"), 11);
    assert_eq!(
        escape_shell_string("'; rm -rf / #"),
        "''\"'\"'; rm -rf / #'"
    );
    assert_eq!(escape_shell_len("'; rm -rf / #"), 19);
}

// ---------------------------------------------------------------------------
// Buffer contract of the allocation-free core
// ---------------------------------------------------------------------------

#[test]
fn test_GH225_buffer_exactly_one_short_returns_none() {
    for s in corpus_adversarial() {
        let b = s.as_bytes();
        let need = escape_bytes_len(b);
        let mut buf = vec![0xAAu8; need.saturating_sub(1)];
        let before = buf.clone();
        assert!(
            escape_bytes_into(b, &mut buf).is_none(),
            "{s:?} needs {need} bytes; a shorter buffer must be refused"
        );
        assert_eq!(buf, before, "a refused call must not write into the buffer");
    }
}

#[test]
fn test_GH225_buffer_oversized_leaves_tail_untouched() {
    for s in corpus_adversarial() {
        let b = s.as_bytes();
        let need = escape_bytes_len(b);
        let mut buf = vec![0xAAu8; need * 4 + 8];
        let n = escape_bytes_into(b, &mut buf).expect("oversized buffer must succeed");
        assert_eq!(n, need);
        assert!(
            buf[n..].iter().all(|&x| x == 0xAA),
            "escape wrote past the {n} bytes it reported for {s:?}"
        );
    }
}

#[test]
fn test_GH225_escape_len_respects_worst_case_bound() {
    for s in corpus_adversarial()
        .iter()
        .chain(corpus_ascii_pairs().iter())
    {
        let need = escape_bytes_len(s.as_bytes());
        assert!(
            need <= 5 * s.len() + 2,
            "{s:?} escapes to {need} bytes, above the 5n+2 bound"
        );
        assert!(need >= 1, "{s:?} escapes to nothing");
    }
    // The bound is tight exactly when every byte is a single quote.
    assert_eq!(escape_bytes_len(b"'''"), 5 * 3 + 2);
}

// ---------------------------------------------------------------------------
// Differential: the delegating wrapper vs the frozen pre-GH-225 oracle
// ---------------------------------------------------------------------------

/// Assert `escape_shell_string` still agrees byte-for-byte with the pre-GH-225
/// implementation, and that the core's length and buffer contracts hold, for one input.
fn assert_agrees(s: &str) {
    let expected = reference(s);
    let actual = escape_shell_string(s);
    assert_eq!(
        actual, expected,
        "escaper diverged from the frozen oracle on {s:?}"
    );

    let need = escape_shell_len(s);
    assert_eq!(
        need,
        expected.len(),
        "escape_shell_len disagrees with the output on {s:?}"
    );

    let mut buf = vec![0u8; need];
    let n = escape_shell_into(s, &mut buf).expect("buffer sized by escape_shell_len");
    assert_eq!(n, need);
    assert_eq!(
        &buf[..n],
        expected.as_bytes(),
        "escape_shell_into diverged on {s:?}"
    );
}

#[test]
fn test_GH225_delegation_matches_frozen_reference_ascii_pairs() {
    for s in corpus_ascii_pairs() {
        assert_agrees(&s);
    }
}

#[test]
fn test_GH225_delegation_matches_frozen_reference_adversarial() {
    for s in corpus_adversarial() {
        assert_agrees(&s);
    }
}

#[test]
fn test_GH225_delegation_matches_frozen_reference_unicode_fuzz() {
    for s in corpus_unicode_fuzz(100_000) {
        assert_agrees(&s);
    }
}

#[test]
fn test_GH225_is_safe_unquoted_byte_and_char_agree() {
    for s in corpus_adversarial()
        .iter()
        .chain(corpus_ascii_pairs().iter())
        .chain(corpus_unicode_fuzz(50_000).iter())
    {
        assert_eq!(
            is_safe_unquoted_bytes(s.as_bytes()),
            reference_is_safe_unquoted(s),
            "byte and char safety predicates disagree on {s:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// escape_variable_name must stay CHARACTER-based (one `_` per char, not per byte)
// ---------------------------------------------------------------------------

#[test]
fn test_GH225_escape_variable_name_char_semantics_preserved() {
    assert_eq!(escape_variable_name("hello_世界"), "hello___");
    assert_eq!(escape_variable_name("café_var"), "caf__var");
    assert_eq!(escape_variable_name("123invalid"), "_23invalid");
    assert_eq!(escape_variable_name("valid_name"), "valid_name");

    for s in corpus_unicode_fuzz(20_000) {
        if s.is_empty() {
            continue;
        }
        let out = escape_variable_name(&s);
        assert_eq!(
            out.len(),
            s.chars().count(),
            "variable escaping must emit one byte per input CHAR, not per byte ({s:?})"
        );
    }
}

/// The pre-GH-225 `escape_variable_name`, verbatim (minus the precondition macro).
/// DO NOT EDIT — this is the oracle for the identifier core.
fn reference_escape_variable_name(name: &str) -> String {
    if reference_is_valid_shell_identifier(name) {
        name.to_string()
    } else {
        let mut result = String::new();
        for (i, c) in name.chars().enumerate() {
            if i == 0 {
                if c.is_ascii_alphabetic() || c == '_' {
                    result.push(c);
                } else {
                    result.push('_');
                }
            } else if c.is_ascii_alphanumeric() || c == '_' {
                result.push(c);
            } else {
                result.push('_');
            }
        }
        result
    }
}

/// The pre-GH-225 `is_valid_shell_identifier`, verbatim. DO NOT EDIT.
fn reference_is_valid_shell_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first_char = match name.chars().next() {
        Some(c) => c,
        None => return false,
    };
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }
    name.chars()
        .skip(1)
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The byte identifier core must agree with the frozen char oracle on every ASCII input,
/// and must refuse every non-ASCII one rather than sanitise it per byte.
#[test]
fn test_GH225_identifier_core_matches_frozen_reference() {
    let ascii: Vec<String> = corpus_ascii_pairs()
        .into_iter()
        .chain(["_", "a_b", "A1", "9x", "my-var", "my var", "_23invalid"].map(String::from))
        .collect();

    for s in &ascii {
        let b = s.as_bytes();
        let mut buf = vec![0u8; b.len()];
        let n = escape_variable_bytes_into(b, &mut buf).expect("ASCII input, exact buffer");
        assert_eq!(n, b.len());
        assert_eq!(
            std::str::from_utf8(&buf[..n]).expect("ASCII out"),
            reference_escape_variable_name(s),
            "identifier core diverged from the frozen oracle on {s:?}"
        );
        assert_eq!(
            is_valid_shell_identifier_bytes(b),
            reference_is_valid_shell_identifier(s),
            "identifier predicates disagree on {s:?}"
        );
    }

    // Non-ASCII must be refused, never sanitised per byte.
    for s in ["hello_世界", "café_var", "🚀", "a\u{202E}b"] {
        let b = s.as_bytes();
        let mut buf = vec![0u8; b.len()];
        assert!(
            escape_variable_bytes_into(b, &mut buf).is_none(),
            "{s:?} is non-ASCII; the byte core must refuse it"
        );
        assert!(!is_valid_shell_identifier_bytes(b));
    }

    // Buffer too small is refused.
    let mut small = [0u8; 2];
    assert!(escape_variable_bytes_into(b"abc", &mut small).is_none());
}

#[test]
fn test_GH225_escape_variable_name_matches_frozen_reference_fuzz() {
    for s in corpus_unicode_fuzz(50_000) {
        if s.is_empty() {
            continue; // outside escape_variable_name's precondition
        }
        assert_eq!(
            escape_variable_name(&s),
            reference_escape_variable_name(&s),
            "variable escaping diverged from the frozen oracle on {s:?}"
        );
    }
}

/// `escape_variable_name` is NOT total: `contract_pre_roundtrip!(name)` sits ahead of
/// any emptiness handling, so the empty string is outside its precondition and trips the
/// debug assert. Pinned here so a future refactor that "helpfully" reorders the macro is
/// caught as the behaviour change it is.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "precondition violated")]
fn test_GH225_escape_variable_name_empty_violates_precondition() {
    let _ = escape_variable_name("");
}

// ---------------------------------------------------------------------------
// Against a real shell, not a model
// ---------------------------------------------------------------------------

/// Every escaped word, fed to a real `sh`, must print back exactly the input.
///
/// Batched: one `sh -c` per 2000 words, NUL-delimited output. NUL bytes are excluded
/// because a shell word cannot carry one — that case is covered by the model tests.
#[test]
fn test_GH225_shell_roundtrip_executes() {
    let corpus: Vec<String> = corpus_ascii_pairs()
        .into_iter()
        .chain(corpus_adversarial())
        .filter(|s| !s.contains('\0'))
        .collect();

    for chunk in corpus.chunks(2000) {
        let script: String = chunk
            .iter()
            .map(|s| format!("printf '%s\\0' {}\n", escape_shell_string(s)))
            .collect();
        let out = match std::process::Command::new("sh")
            .arg("-c")
            .arg(&script)
            .output()
        {
            Ok(o) => o,
            Err(_) => return, // no /bin/sh here — the model tests still cover the property
        };
        assert!(
            out.status.success(),
            "sh rejected a script built from escaped words"
        );
        let mut parts = out.stdout.split(|&b| b == 0);
        for s in chunk {
            let got = parts.next().expect("one NUL-terminated field per word");
            assert_eq!(
                got,
                s.as_bytes(),
                "sh printed {:?} for input {s:?} escaped as {:?}",
                String::from_utf8_lossy(got),
                escape_shell_string(s)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4096))]

        #[test]
        fn test_GH225_prop_delegation_matches_frozen_reference(s in any::<String>()) {
            assert_agrees(&s);
        }

        #[test]
        fn test_GH225_prop_char_vec_matches_frozen_reference(
            cs in prop::collection::vec(any::<char>(), 0..16)
        ) {
            let s: String = cs.into_iter().collect();
            assert_agrees(&s);
        }

        #[test]
        fn test_GH225_prop_output_is_valid_utf8_for_all_utf8_input(s in any::<String>()) {
            let mut buf = vec![0u8; escape_shell_len(&s)];
            let n = escape_shell_into(&s, &mut buf).expect("sized buffer");
            prop_assert!(std::str::from_utf8(&buf[..n]).is_ok());
        }

        #[test]
        fn test_GH225_prop_byte_core_matches_str_wrapper(
            bs in prop::collection::vec(any::<u8>(), 0..24)
        ) {
            if let Ok(s) = std::str::from_utf8(&bs) {
                prop_assert_eq!(escape_bytes_len(&bs), escape_shell_len(s));
                let mut a = vec![0u8; escape_bytes_len(&bs)];
                let mut b = vec![0u8; escape_shell_len(s)];
                let na = escape_bytes_into(&bs, &mut a).expect("sized");
                let nb = escape_shell_into(s, &mut b).expect("sized");
                prop_assert_eq!(na, nb);
                prop_assert_eq!(&a[..na], &b[..nb]);
            }
        }
    }
}
