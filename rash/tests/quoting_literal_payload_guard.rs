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
    // bashrs#388. `$h=` inside a double-quoted word is text being built, not
    // the left side of an assignment; SC1066 is a line regex and reported it at
    // Severity::Error, failing a correct fleet watch script at the lint gate.
    Payload {
        code: "SC1066",
        bare: "$VAR=hello\n",
        quoted: "h=x\ns=\"\"\ns=\"$s $h=UP\"\necho \"$s\"\n",
        found_at: "infra/machines/lambda-labs/fleet-resource-watch.sh:106 (bashrs#388)",
    },
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
        code: "SC2242",
        // `break` inside a `case` that is not inside a loop — the real defect.
        bare: "case \"$x\" in\n    a) break ;;\nesac\n",
        // The same two keywords as English. #332 made SC2242 count case depth
        // per line, so "this case" opens one and "break the matcher" is read
        // as the break inside it.
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
    // PMAT-248
    Payload {
        code: "SC1109",
        bare: "echo a &lt; b\n",
        // An HTML entity inside an unquoted heredoc body is text being
        // written out, not a typo in this script's own shell syntax.
        quoted: "cat <<EOF\n<li>x &lt; 10</li>\nEOF\n",
        found_at: "#242",
    },
    Payload {
        code: "SC2006",
        bare: "echo `date`\n",
        // A backslash-escaped backtick inside "..." is literal text (POSIX
        // 2.2.3 escapes $, `, " and \ in a double-quoted string); it is not a
        // command substitution.
        quoted: "echo \"Parse markdown links: \\`[text](url)\\`\"\n",
        found_at: "#252",
    },
    Payload {
        code: "SC2099",
        bare: "id=`id -u`\n",
        quoted: "echo \"Parse markdown links: \\`[text](url)\\`\"\n",
        found_at: "#252",
    },
    Payload {
        code: "SC2276",
        // A pipe to another command — the heredoc really is useless here.
        bare: "cat <<EOF | grep x\nfoo\nEOF\n",
        // The `|` is a literal character inside a quoted redirect target,
        // not a pipe: this cat's output goes to a file, not a command.
        quoted: "cat <<EOF > \"file|name\"\nfoo\nEOF\n",
        found_at: "#242 review",
    },
    // bashrs#362. SC2086 and SC2154 are line rules keyed on `$name`; neither
    // knew it was inside `'...'` or after a trailing `#`, so a GraphQL query, a
    // jq filter or an awk program in single quotes, and a comment naming a
    // variable, each read as an unquoted expansion of an unset variable.
    Payload {
        code: "SC2086",
        bare: "echo $x\n",
        quoted: "g() {\n    local q='query($org:String!){ x }'\n    printf '%s\\n' \"$q\"\n}\ng\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh (bashrs#362)",
    },
    Payload {
        code: "SC2086",
        bare: "echo $x\n",
        quoted: "f() { # $1 is the file, written to $t/w.yml\n    :\n}\nf\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh (bashrs#362)",
    },
    // SC2154 must still see a reference wherever one EXPANDS. `"$org"` is one
    // (the literal mask keeps `$name` visible inside "..." as well), and an
    // UNQUOTED heredoc body is the case that tells the two masks apart:
    // `mask_literals` blanks every heredoc body whole, so routing SC2154
    // through it would lose `$t` below — a real read of an unset variable.
    Payload {
        code: "SC2154",
        bare: "echo \"$org\"\n",
        quoted: "g() {\n    local q='query($org:String!){ x }'\n    printf '%s\\n' \"$q\"\n}\ng\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh (bashrs#362)",
    },
    Payload {
        code: "SC2154",
        bare: "cat <<EOF\n$t\nEOF\n",
        quoted: "f() { # $1 is the file, written to $t/w.yml\n    :\n}\nf\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh (bashrs#362)",
    },
    // bashrs#375. SC1087 read `$s[0]` inside a multi-line single-quoted jq
    // program as an unbraced array expansion, and SEC012 read the substring
    // `eval` in the jq field `.eval_count` as the eval builtin. The bare cases
    // keep each rule honest: `"$arr[0]"` in DOUBLE quotes does expand (so
    // SC1087 must not be routed through the literal mask), and a real
    // `eval "$(jq ...)"` must still fire SEC012.
    Payload {
        code: "SC1087",
        bare: "arr=(a b)\necho \"$arr[0]\"\n",
        quoted: "S=x\njq -nc --argjson s \"$S\" '{\n  min:($s[0]), max:($s[-1])}'\n",
        found_at: "infra/machines/lambda-labs/lqw/lqw-bench.sh:42 (bashrs#375)",
    },
    Payload {
        code: "SEC012",
        bare: "f=x\neval \"$(jq -r '.a' \"$f\")\"\n",
        quoted: "line=x\nct=$(jq -r '.eval_count // 0' <<<\"$line\"); pt=$(jq -r '.prompt_eval_count // 0' <<<\"$line\")\necho \"$ct $pt\"\n",
        found_at: "infra/machines/lambda-labs/lqw/lqw-bench.sh:73 (bashrs#375)",
    },
    // bashrs#364. A POSIX awk program in '...' is another language's grammar:
    // `if (…)`, `else if`, `a || b <= c`, a regex `[ \t]+` and a `#` inside a
    // regex are awk, not shell. 77 of 129 warnings on one infra guard were
    // these five rules reading its 100-line awk program. The construct sits on
    // a CONTINUATION line of the string, as in the census — a line rule sees no
    // quote there at all, which is why a one-line `awk '…'` does not reproduce
    // it. Each bare case is the finding the rule's own tests assert.
    Payload {
        code: "SC2204",
        bare: "if ( true ); then\n    :\nfi\n",
        quoted: "f=x\nawk '\n    { if (s ~ /^y/ || s ~ /z$/) s = substr(s, 2) }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:60 (bashrs#364)",
    },
    Payload {
        code: "SC1075",
        bare:
            "x=1\nif [ \"$x\" -eq 1 ]; then\n    :\nelse if [ \"$x\" -eq 2 ]; then\n    :\nfi\nfi\n",
        quoted: "f=x\nawk '\n    { if (a) b++\n    else if (c != \"\") { d++ } }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:69 (bashrs#364)",
    },
    Payload {
        code: "SC2297",
        bare: "cat file | sort > output\n",
        quoted: "f=x\nawk '\n    { if (oi < 0 || n <= oi) next }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:93 (bashrs#364)",
    },
    Payload {
        code: "SC2102",
        bare: "v=1\n[[ $v = [0-9]+ ]] && echo n\n",
        quoted: "f=x\nawk '\n    { sub(/^[ \\t]+/, \"\", s) }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:58 (bashrs#364)",
    },
    Payload {
        code: "SC1099",
        bare: "echo hello#world\n",
        quoted: "f=x\nawk '\n    { sub(/[ \\t]+#.*$/, \"\", s) }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:59 (bashrs#364)",
    },
    // Three info-level rules in the same program, same cause: a `\t` in an awk
    // regex is not a shell escape the shell drops (SC1012, SC2025), and awk
    // has a `function` keyword too (SC2112 — SC2111, its ksh twin, is already
    // listed for exactly that).
    Payload {
        code: "SC1012",
        bare: "echo hello\\tworld\n",
        quoted: "f=x\nawk '\n    { sub(/^[ \\t]+/, \"\", s) }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:58 (bashrs#364)",
    },
    Payload {
        code: "SC2025",
        bare: "echo Hello\\nWorld\n",
        quoted: "f=x\nawk '\n    { sub(/^[ \\t]+/, \"\", s) }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:58 (bashrs#364)",
    },
    Payload {
        code: "SC2112",
        bare: "function foo { echo \"bar\"; }\nfoo\n",
        quoted: "f=x\nawk '\n    function trim(s) { return s }\n    { print trim($0) }\n' \"$f\"\n",
        found_at: "infra/machines/clean-room/coverage-on-tag-census.sh:58 (bashrs#364)",
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

/// Every diagnostic code, regardless of severity — PMAT-248's SC2006 and
/// SC2099 are `Severity::Info`, so `error_codes` alone would never see them
/// fire and "must still fire on the bare payload" would pass vacuously.
fn all_codes(source: &str) -> Vec<String> {
    let script = format!("#!/usr/bin/env bash\n{source}");
    lint_shell(&script)
        .diagnostics
        .iter()
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
        let codes = all_codes(p.bare);
        assert!(
            codes.iter().any(|c| c == p.code),
            "{} stopped firing on the real defect it exists for — a false \
             positive traded for a false negative. Got {codes:?}\n--- script ---\n{}",
            p.code,
            p.bare
        );
    }
}

/// Stronger than `quoted_payloads_produce_no_errors` for `Severity::Info`
/// rules (SC2006, SC2099): checks the rule's OWN code is absent from the
/// quoted fixture regardless of severity, rather than only that no
/// `Severity::Error` diagnostic appears anywhere.
#[test]
fn quoted_payloads_produce_none_of_their_own_code() {
    for p in PAYLOADS {
        let codes = all_codes(p.quoted);
        assert!(
            !codes.iter().any(|c| c == p.code),
            "{} ({}): fired on literal text — {codes:?}\n--- script ---\n{}",
            p.code,
            p.found_at,
            p.quoted
        );
    }
}
