//! GH-386: `date` given a time to convert (`-d`/`--date`), a file whose mtime
//! to print (`-r`/`--reference`) or a file of dates (`-f`/`--file`) reads no
//! clock. Its output is a pure function of its input, so it is not a
//! timestamp source. Bare `date` and `date +FMT` still read the clock and
//! stay DET002. One fixture per spelling of the operand, because a detector
//! is only as good as the form variants it has been shown.

#![allow(non_snake_case)]

use super::check;

/// Each line would be DET002 if `date` were read as a clock: the value goes
/// into an artifact name.
fn artifact(date_cmd: &str) -> String {
    format!("#!/bin/bash\nTS=$({date_cmd})\ncp build.log \"out/report_$TS.log\"\n")
}

#[test]
fn test_GH386_conversion_forms_are_not_timestamps() {
    for cmd in [
        "date -u -d \"$lm\" +%s",
        "date -d \"$lm\" +%s",
        "date -d\"$lm\" +%s",
        "date --date=\"$lm\" +%s",
        "date --date \"$lm\" +%s",
        "date -ud \"$lm\" +%s",
        "date -r build.log +%s",
        "date --reference=build.log +%s",
        "date -f dates.txt +%s",
        "date +%s -d \"$lm\"",
    ] {
        let n = check(&artifact(cmd)).diagnostics.len();
        assert_eq!(n, 0, "`{cmd}` converts its operand and reads no clock");
    }
}

#[test]
fn test_GH386_bare_date_form_from_the_issue_is_not_reported() {
    // Issue #386 reproducer: a stderr redirect made it look like a sink.
    let script = "#!/bin/bash\nf() { local lm; lm=$(curl -fsIL \"$1\" | sed -n 's/^Last-Modified: //p'); date -u -d \"$lm\" +%s 2>/dev/null || true; }\n";
    assert_eq!(check(script).diagnostics.len(), 0);
}

#[test]
fn test_GH386_clock_reads_are_still_reported() {
    for cmd in ["date", "date +%s", "date -u +%Y%m%d", "date -u", "date -R", "date -Iseconds"] {
        let n = check(&artifact(cmd)).diagnostics.len();
        assert_eq!(n, 1, "`{cmd}` reads the clock into an artifact name");
    }
    // Bare `date` into a file, and a `-u` cluster with no operand flag.
    assert_eq!(check("#!/bin/bash\ndate -u > VERSION\n").diagnostics.len(), 1);
}

#[test]
fn test_GH386_an_operand_flag_on_another_command_does_not_exempt_date() {
    // The `-d` belongs to `cut`, not to `date`.
    let script = "#!/bin/bash\nTS=$(date +%s | cut -d' ' -f1)\ncp build.log \"out/report_$TS.log\"\n";
    assert_eq!(check(script).diagnostics.len(), 1);
}

#[test]
fn test_GH386_operand_flag_in_quoted_text_or_comment_does_not_exempt_date() {
    // Only a flag `date` itself receives counts. A `-f` inside a quoted
    // argument, or `--date` in a trailing comment, is text.
    for cmd in ["date +%s \"use -f flag\"", "date '+%s -r'"] {
        let n = check(&artifact(cmd)).diagnostics.len();
        assert_eq!(n, 1, "`{cmd}` still reads the clock");
    }
    let script = "#!/bin/bash\ndate -u > VERSION # not --date\n";
    assert_eq!(check(script).diagnostics.len(), 1);
}
