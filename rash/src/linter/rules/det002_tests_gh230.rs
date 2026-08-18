//! GH-230: DET002 must fire on timestamps that reach a *reproducible sink*, and
//! must stay quiet on log lines, stdout, comparisons and SOURCE_DATE_EPOCH.

#![allow(non_snake_case)]

use super::det002::check;

fn count(src: &str) -> usize {
    check(src).diagnostics.len()
}

// ---------- must KEEP firing (genuine reproducibility defects) ----------

#[test]
fn test_GH230_sink_filename_flagged() {
    let script = "#!/bin/sh\nTIMESTAMP=\"$(date +%Y%m%d_%H%M%S)\"\ncp build.log \"out/report_$TIMESTAMP.log\"\n";
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1, "artifact filename is a sink");
    let d = &result.diagnostics[0];
    assert_eq!(d.span.start_line, 2, "span stays on the date line");
    assert_eq!(d.span.start_col, 12);
    assert!(
        d.message.contains("line 3"),
        "message must name the sink line, got: {}",
        d.message
    );
}

#[test]
fn test_GH230_tee_without_append_flagged() {
    assert_eq!(count("printf '%s' \"$(date +%s)\" | tee build.stamp\n"), 1);
}

#[test]
fn test_GH230_truncating_redirect_flagged() {
    let script = "#!/bin/sh\nTS=$(date +%Y%m%d)\necho hi > \"report_$TS.txt\"\n";
    let r = check(script);
    assert_eq!(r.diagnostics.len(), 1);
    assert!(r.diagnostics[0].message.contains("line 3"));
}

#[test]
fn test_GH230_tar_artifact_flagged() {
    let script = "#!/bin/sh\nTS=$(date +%Y%m%d)\ntar -czf \"backup_$TS.tar.gz\" /data\n";
    let r = check(script);
    assert_eq!(r.diagnostics.len(), 1);
    assert!(r.diagnostics[0].message.contains("line 3"));
}

#[test]
fn test_GH230_build_id_assignment_flagged() {
    let r = check("BUILD_ID=\"1.0.0-$(date +%s)\"\n");
    assert_eq!(r.diagnostics.len(), 1);
    assert!(r.diagnostics[0].message.contains("line 1"));
}

#[test]
fn test_GH230_taint_propagates_two_hops() {
    let script = "#!/bin/sh\nTS=$(date +%Y%m%d)\nNAME=\"report_$TS\"\nDEST=\"/out/$NAME.txt\"\ncp build.log \"$DEST\"\n";
    let r = check(script);
    assert_eq!(r.diagnostics.len(), 1);
    assert_eq!(r.diagnostics[0].span.start_line, 2);
    assert!(
        r.diagnostics[0].message.contains("line 5"),
        "got: {}",
        r.diagnostics[0].message
    );
}

// A one-line compound used to be classified as ONE unit, so the comparison in
// the condition marked the artifact write in the body benign. The sink is in
// the body whichever line it is written on.
#[test]
fn test_GH230_oneline_if_body_sink_not_masked_by_condition() {
    let script = "#!/bin/bash\nTS=$(date +%s)\nif [ -n \"$TS\" ]; then cp build.tar \"out/build-$TS.tar\"; fi\n";
    let r = check(script);
    assert_eq!(
        r.diagnostics.len(),
        1,
        "the `cp` in the body is a reproducible sink"
    );
    assert!(
        r.diagnostics[0].message.contains("line 3"),
        "the body, not the condition, is the sink; got: {}",
        r.diagnostics[0].message
    );
}

#[test]
fn test_GH230_oneline_until_body_sink_not_masked_by_condition() {
    let script = "#!/bin/bash\nTS=$(date +%s)\nuntil [ -z \"$TS\" ]; do cp b.tar \"out/b-$TS.tar\"; break; done\n";
    let r = check(script);
    assert_eq!(r.diagnostics.len(), 1);
    assert!(r.diagnostics[0].message.contains("line 3"), "got: {}", r.diagnostics[0].message);
}

#[test]
fn test_GH230_oneline_if_body_that_only_logs_stays_quiet() {
    // The other half of the same contract: splitting the line must not turn a
    // guarded log line into a finding.
    let script = "#!/bin/bash\nTS=$(date +%s)\nif [ -n \"$TS\" ]; then echo \"started at $TS\"; fi\n";
    assert_eq!(count(script), 0);
}

#[test]
fn test_GH230_oneline_while_body_without_the_value_stays_quiet() {
    let script = "#!/bin/bash\nTS=$(date +%s)\nwhile [ \"$TS\" -lt 5 ]; do sleep 1; done\n";
    assert_eq!(count(script), 0);
}

#[test]
fn test_GH230_guarded_use_agrees_with_the_unguarded_one() {
    // Default-deny does not care whether a use is guarded: an unclassifiable
    // sink is reported the same on one line as on two.
    let guarded = "TS=$(date +%s)\nif [ -n \"$TS\" ]; then send_metric \"$TS\"; fi\n";
    let bare = "TS=$(date +%s)\nsend_metric \"$TS\"\n";
    assert_eq!(count(guarded), count(bare));
    assert_eq!(count(bare), 1);
}

#[test]
fn test_GH230_semicolon_inside_quotes_does_not_split() {
    // The split is quote- and depth-aware: this is one `echo`, still benign.
    assert_eq!(count("TS=$(date +%s)\necho \"a;b $TS\"\n"), 0);
}

#[test]
fn test_GH230_unused_capture_still_flagged() {
    // Contract F-DET002-SOUND: default-deny. An unused capture stays reported.
    assert_eq!(count("ts=$(date +%s)\n"), 1);
}

#[test]
fn test_GH230_marker_must_be_a_comment() {
    // A `telemetry` substring in *code* used to silence the rule for the whole
    // following assignment block.
    let script = "#!/bin/sh\ncurl -s https://telemetry.example.com/ping\nSTAMP=$(date +%s)\ncp build.log \"out/report_$STAMP.log\"\n";
    assert_eq!(count(script), 1, "code-line markers must not suppress");
}

#[test]
fn test_GH230_reassignment_kills_taint() {
    let script = "#!/bin/sh\nTS=$(date +%s)\nTS=\"static\"\ncp a \"b_$TS\"\n";
    let r = check(script);
    assert_eq!(r.diagnostics.len(), 1);
    assert!(
        !r.diagnostics[0].message.contains("line 4"),
        "reassignment kills the taint, so line 4 is not the sink"
    );
}

#[test]
fn test_GH230_append_with_timestamped_target_flagged() {
    assert_eq!(count("echo x >> \"log_$(date +%Y%m%d).txt\"\n"), 1);
}

#[test]
fn test_GH230_sudo_prefix_recognised() {
    assert_eq!(count("sudo cp a \"b_$(date +%s)\"\n"), 1);
}

#[test]
fn test_GH230_checksum_sink_flagged() {
    let script = "#!/bin/sh\nTS=$(date +%s)\necho \"$TS\" | sha256sum > sums.txt\n";
    assert_eq!(count(script), 1);
}

#[test]
fn test_GH230_env_prefix_assignment_not_a_sink() {
    assert_eq!(count("TZ=UTC date +%s\n"), 1);
}

#[test]
fn test_GH230_backtick_in_artifact_arg_flagged() {
    assert_eq!(count("mv a \"b_`date +%Y`\"\n"), 1);
}

// ---------- must go to ZERO (false positives) ----------

#[test]
fn test_GH230_log_line_not_flagged() {
    let script = "#!/bin/sh\nLOG_FILE=/var/log/app.log\necho \"[$(date '+%Y-%m-%d %H:%M:%S')] started\" | tee -a \"$LOG_FILE\"\n";
    assert_eq!(count(script), 0, "an append-only log line is not a defect");
}

#[test]
fn test_GH230_source_date_epoch_clears() {
    let script = "#!/bin/sh\nTIMESTAMP=\"$(date -u -d \"@${SOURCE_DATE_EPOCH:-$(date +%s)}\" +%Y%m%d 2>/dev/null || date +%Y%m%d)\"\ncp build.log \"out/report_$TIMESTAMP.log\"\n";
    assert_eq!(count(script), 0, "DET002 must not fire on its own remedy");
}

#[test]
fn test_GH230_source_date_epoch_export_then_use() {
    let script = "#!/bin/sh\nexport SOURCE_DATE_EPOCH=\"${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}\"\nBUILD_DATE=\"$(date -u -d \"@$SOURCE_DATE_EPOCH\" +%Y-%m-%d)\"\necho \"$BUILD_DATE\" > version.txt\n";
    assert_eq!(count(script), 0);
}

#[test]
fn test_GH230_append_redirect_benign() {
    assert_eq!(count("echo \"start $(date +%s)\" >> /var/log/app.log\n"), 0);
}

#[test]
fn test_GH230_logger_benign() {
    assert_eq!(count("logger \"started at $(date +%s)\"\n"), 0);
}

#[test]
fn test_GH230_stdout_benign() {
    assert_eq!(count("echo \"now: $(date +%s)\"\n"), 0);
}

#[test]
fn test_GH230_tee_append_benign() {
    assert_eq!(
        count("printf '[%s] ok\\n' \"$(date +%s)\" | tee -a /var/log/x.log\n"),
        0
    );
}

#[test]
fn test_GH230_comment_not_flagged() {
    assert_eq!(count("#!/bin/sh\n# never use $(date +%s) in a build\n"), 0);
}

#[test]
fn test_GH230_single_quoted_literal_not_flagged() {
    assert_eq!(count("echo 'literal $(date +%s) not executed'\n"), 0);
}

#[test]
fn test_GH230_quoted_heredoc_body_not_flagged() {
    let script = "#!/bin/sh\ncat > /tmp/gen.sh <<'INNER'\nTS=$(date +%s)\nINNER\n";
    assert_eq!(count(script), 0);
}

#[test]
fn test_GH230_two_sources_one_benign() {
    let script = "#!/bin/sh\nA=$(date +%s)\nB=$(date +%s)\necho \"$A\" >> log\ncp x \"out_$B\"\n";
    let r = check(script);
    assert_eq!(r.diagnostics.len(), 1);
    assert_eq!(r.diagnostics[0].span.start_line, 3);
}

#[test]
fn test_GH230_comparison_sink_not_flagged() {
    let script = "#!/bin/sh\nNOW=$(date +%s)\nif [ \"$NOW\" -gt 100 ]; then echo yes; fi\n";
    assert_eq!(count(script), 0);
}

#[test]
fn test_GH230_dev_null_redirect_benign() {
    assert_eq!(count("echo \"$(date +%s)\" > /dev/null\n"), 0);
}

#[test]
fn test_GH230_stderr_redirect_benign() {
    assert_eq!(count("echo \"[$(date +%s)] warn\" >&2\n"), 0);
}

// ---------- message / remedy ----------

#[test]
fn test_GH230_message_names_source_date_epoch() {
    let r = check("ts=$(date +%s)\n");
    assert!(
        r.diagnostics[0].message.contains("SOURCE_DATE_EPOCH"),
        "got: {}",
        r.diagnostics[0].message
    );
}

#[test]
fn test_GH230_remedy_text_does_not_self_flag() {
    // The core contradiction: every remedy we hand the user must itself be
    // DET002-clean when pasted into a script.
    let r = check("ts=$(date +%s)\n");
    let fix = r.diagnostics[0].fix.as_ref().expect("DET002 carries a fix");
    for alt in &fix.suggested_alternatives {
        assert_eq!(
            count(alt),
            0,
            "suggested remedy self-flags under DET002: {alt}"
        );
    }
    assert_eq!(
        count(&r.diagnostics[0].message),
        0,
        "diagnostic message self-flags under DET002: {}",
        r.diagnostics[0].message
    );
}

#[test]
fn test_GH230_both_dispatchers_agree() {
    use std::path::Path;
    let script = "#!/bin/sh\nLOG_FILE=/var/log/app.log\necho \"[$(date '+%Y-%m-%d %H:%M:%S')] started\" | tee -a \"$LOG_FILE\"\n";
    let a = crate::linter::lint_shell(script)
        .diagnostics
        .iter()
        .filter(|d| d.code == "DET002")
        .count();
    let b = crate::linter::lint_shell_with_path(Path::new("x.sh"), script)
        .diagnostics
        .iter()
        .filter(|d| d.code == "DET002")
        .count();
    assert_eq!(a, 0);
    assert_eq!(a, b, "both dispatchers must agree on DET002");
}

// ---------- property tests ----------

mod props {
    use super::{check, count};
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// Any read of SOURCE_DATE_EPOCH clears DET002, whatever the format
        /// string. The rule must never fire on its own remedy.
        #[test]
        fn prop_GH230_source_date_epoch_always_clears(fmt in "[a-zA-Z%_:-]{1,12}") {
            let src = format!(
                "D=\"$(date -u -d \"@${{SOURCE_DATE_EPOCH:-$(date +%s)}}\" +{fmt})\"\ncp a \"b_$D\"\n"
            );
            prop_assert_eq!(count(&src), 0);
        }

        /// Appending to a log is never a DET002 defect.
        #[test]
        fn prop_GH230_append_log_never_flagged(name in "[a-z]{1,8}") {
            let src = format!("echo \"[$(date +%s)] {name}\" >> /var/log/{name}.log\n");
            prop_assert_eq!(count(&src), 0);
        }

        /// A timestamped artifact name is always a DET002 defect.
        #[test]
        fn prop_GH230_artifact_name_always_flagged(name in "[a-z]{1,8}") {
            let src = format!("cp build.log \"out/{name}_$(date +%s).log\"\n");
            prop_assert_eq!(count(&src), 1);
        }

        /// Every span lands, in range, on a line that really contains `date`.
        #[test]
        fn prop_GH230_span_anchors_on_a_date_line(
            lines in proptest::collection::vec(
                prop_oneof![
                    Just("TS=$(date +%s)"),
                    Just("echo \"$TS\" >> app.log"),
                    Just("cp a \"b_$TS\""),
                    Just("# just a $(date +%s) comment"),
                    Just("echo hello"),
                    Just("D=`date`"),
                ],
                1..8usize,
            )
        ) {
            let script = lines.join("\n");
            let src: Vec<&str> = script.lines().collect();
            for d in check(&script).diagnostics {
                prop_assert!(d.span.start_line >= 1 && d.span.start_line <= src.len());
                prop_assert!(src[d.span.start_line - 1].contains("date"));
            }
        }

        /// Same input, byte-identical diagnostics - no map-iteration order leaks.
        #[test]
        fn prop_GH230_analysis_is_deterministic(
            lines in proptest::collection::vec(
                prop_oneof![
                    Just("A=$(date +%s)"),
                    Just("B=$(date +%Y)"),
                    Just("cp x \"o_$A$B\""),
                    Just("echo \"$A\" >> log"),
                ],
                1..8usize,
            )
        ) {
            let script = lines.join("\n");
            let render = |s: &str| -> Vec<String> {
                check(s)
                    .diagnostics
                    .iter()
                    .map(|d| format!("{}:{}:{}", d.span.start_line, d.span.start_col, d.message))
                    .collect()
            };
            let once = render(&script);
            for _ in 0..5 {
                prop_assert_eq!(&once, &render(&script));
            }
        }
    }
}

#[test]
fn test_GH230_append_to_an_artifact_is_not_a_log() {
    // Adversarial review: the append test looked at the OPERATOR, so the rule
    // was defeated by changing one character (`>` -> `>>`).
    let script = "#!/bin/sh\nTS=$(date +%s)\necho \"$TS\" >> dist/checksums.txt\n";
    assert_eq!(check(script).diagnostics.len(), 1, "a checksum file is an artifact");

    let script = "#!/bin/sh\nTS=$(date +%s)\necho \"build=$TS\" >> release.env\n";
    assert_eq!(check(script).diagnostics.len(), 1, "release.env is an artifact");
}

#[test]
fn test_GH230_append_to_a_real_log_stays_benign() {
    for script in [
        "#!/bin/sh\nTS=$(date +%s)\necho \"[$TS] started\" >> /var/log/app.log\n",
        "#!/bin/sh\nTS=$(date +%s)\necho \"[$TS] started\" >> \"$LOG_FILE\"\n",
        "#!/bin/sh\nTS=$(date +%s)\necho \"[$TS]\" | tee -a \"$LOG_FILE\"\n",
    ] {
        assert_eq!(
            check(script).diagnostics.len(),
            0,
            "a timestamp on a log line is the point of a log line: {script}"
        );
    }
}
