//! GH-272: a shell-SYNTAX rule must never react to text it finds inside a
//! string literal or a heredoc body.
//!
//! `linter::quoting` already resolves quoting for the whole file and hands the
//! masked copy to the rules named in `QUOTE_SENSITIVE_RULES`. The defect is
//! that the allowlist is hand-maintained: a rule added later reads the raw
//! source by default, so it sees `#!/usr/bin/env bash` inside `cat <<EOF`, or
//! the word `break` inside an English sentence, and reports it at
//! `Severity::Error`. That is the same shape as bashrs#266 — two lists with
//! nothing tying them together.
//!
//! This file is the thing that ties them together. `PAYLOADS` holds one
//! construct per rule that was found reacting to literal text on the rmedia
//! script corpus. Each is asserted twice:
//!
//! - `quoted_payloads_produce_no_errors` — wrapped in a heredoc or a string,
//!   the whole fixture must be error-clean. A rule that starts reading literal
//!   text fails here.
//! - `unquoted_payloads_still_fire` — the SAME payload as bare code must still
//!   produce its finding. Without this the first test could be passed by
//!   deleting the rules, and a fix that trades a false positive for a false
//!   negative would look like success.
//!
//! Every quoted fixture below is clean under `shellcheck -S error` and
//! `bash -n`.

use bashrs::linter::{lint_shell, Severity};

/// A construct that is shell syntax as code and ordinary text inside a quote.
struct Payload {
    /// The rule that reacted to it inside a literal.
    code: &'static str,
    /// The payload as bare code — must still be reported.
    bare: &'static str,
    /// The same payload as literal text — must be reported by nothing.
    quoted: &'static str,
    /// Where it was found reacting to literal text.
    found_at: &'static str,
}

const PAYLOADS: &[Payload] = &[
    Payload {
        code: "SC1128",
        // A shebang genuinely not on line 1.
        bare: "echo hi\n#!/usr/bin/env bash\n",
        // ...and the same bytes written INTO a file by a heredoc. The
        // delimiter is unquoted, which is what the corpus does, so this is not
        // covered by the quoted-heredoc filter.
        quoted: "cat > \"$dir/cmd.sh\" <<EOF\n#!/usr/bin/env bash\necho x\nEOF\n",
        found_at: "rmedia/scripts/falsify-ci-retry-classifier.sh:23",
    },
    Payload {
        code: "SC2188",
        bare: "> out.txt\n",
        // `<svg …>` and `</svg>` in a heredoc are XML, not redirections.
        quoted: "cat > frame.svg << FEOF\n<svg viewBox=\"0 0 10 10\">\n</svg>\nFEOF\n",
        found_at: "rmedia/scripts/demo-advanced.sh:163",
    },
    Payload {
        code: "SC2105",
        bare: "if true; then\n    break\nfi\n",
        // "break" as an English verb in a diagnostic message.
        quoted: "echo \"could not break the matcher — this case discriminates nothing\"\n",
        found_at: "rmedia/scripts/falsify-complexity-count-single-sourced.sh:180",
    },
    Payload {
        code: "SC2111",
        bare: "function greet() { echo hi; }\ngreet\n",
        // awk has a `function` keyword too, and the program is one
        // single-quoted argument spanning lines.
        quoted: "awk '\n    function flush() { print \"x\" }\n    { flush() }\n' \"$file\"\n",
        found_at: "rmedia/scripts/lint-feature-gates.sh:167",
    },
    Payload {
        code: "SC2122",
        bare: "a=1; b=2\nif [ \"$a\" >= \"$b\" ]; then echo x; fi\n",
        // `>=` inside the program handed to another interpreter.
        quoted:
            "cov=90\nif [ \"$(printf '%s' \"int($cov >= 85)\")\" != \"1\" ]; then echo no; fi\n",
        found_at: "rmedia/scripts/prove-course-gates.sh:379",
    },
];

fn error_codes(source: &str) -> Vec<String> {
    let script = format!("#!/usr/bin/env bash\n{source}");
    lint_shell(&script)
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn quoted_payloads_produce_no_errors() {
    for p in PAYLOADS {
        let codes = error_codes(p.quoted);
        assert!(
            codes.is_empty(),
            "{} ({}): literal text produced {codes:?}\n--- script ---\n{}",
            p.code,
            p.found_at,
            p.quoted
        );
    }
}

/// The whole set at once, so a rule that only misfires with several constructs
/// present is caught too — that is how the corpus actually looks.
#[test]
fn the_payloads_together_produce_no_errors() {
    let script: String = PAYLOADS.iter().map(|p| p.quoted).collect();
    let codes = error_codes(&script);
    assert!(
        codes.is_empty(),
        "a file of nothing but literal payloads produced {codes:?}"
    );
}

#[test]
fn unquoted_payloads_still_fire() {
    for p in PAYLOADS {
        let codes = error_codes(p.bare);
        assert!(
            codes.iter().any(|c| c == p.code),
            "{} stopped firing on the real defect it exists for — a false \
             positive traded for a false negative. Got {codes:?}\n--- script ---\n{}",
            p.code,
            p.bare
        );
    }
}
