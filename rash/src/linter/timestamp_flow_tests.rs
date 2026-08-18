//! Unit tests for the GH-230 timestamp sink analysis.
//!
//! These test the *helpers* directly; the rule-level behaviour lives in
//! `rules/det002_tests_gh230.rs`.

#![allow(clippy::unwrap_used)]

use super::*;

// ---------------------------------------------------------------------------
// literal masking
// ---------------------------------------------------------------------------

fn literal_at(line: &str, needle: &str) -> bool {
    let m = Scanner::scan(line);
    let idx = line.find(needle).expect("needle present in line");
    m.is_literal(idx)
}

#[test]
fn test_GH230_mask_single_quoted_body_is_literal() {
    assert!(literal_at("echo 'literal $(date +%s) here'", "$(date"));
}

#[test]
fn test_GH230_mask_command_sub_inside_double_quotes_is_code() {
    assert!(!literal_at("echo \"now $(date +%s)\"", "$(date"));
}

#[test]
fn test_GH230_mask_plain_double_quoted_text_is_literal() {
    assert!(literal_at("echo \"a date +%s b\"", "date +%s"));
}

#[test]
fn test_GH230_mask_comment_body_is_literal() {
    assert!(literal_at("# never use $(date +%s)", "$(date"));
}

#[test]
fn test_GH230_mask_hash_inside_word_is_not_a_comment() {
    let m = Scanner::scan("echo ${x#pre} $(date)");
    assert_eq!(m.comment, None);
    assert!(!literal_at("echo ${x#pre} $(date)", "$(date"));
}

#[test]
fn test_GH230_mask_param_expansion_default_keeps_code() {
    // `${A:-$(date +%s)}` - the fallback is a real command substitution.
    assert!(!literal_at("X=\"${A:-$(date +%s)}\"", "$(date"));
}

#[test]
fn test_GH230_mask_backticks_close_rather_than_nest() {
    let line = "X=`date` Y=`id`";
    let m = Scanner::scan(line);
    // The trailing text after the first backtick pair is still top level.
    assert_eq!(m.depth_at(line.find(" Y=").unwrap()), 1);
}

#[test]
fn test_GH230_mask_trailing_comment_offset() {
    let line = "TS=$(date +%s)  # note";
    let m = Scanner::scan(line);
    assert_eq!(m.comment, Some(line.find("# note").unwrap()));
    assert_eq!(m.code_of(line).trim(), "TS=$(date +%s)");
}

// ---------------------------------------------------------------------------
// words, segments, redirects
// ---------------------------------------------------------------------------

#[test]
fn test_GH230_split_words_respects_quotes_and_subs() {
    assert_eq!(
        split_words("cp build.log \"out/a b.log\" $(date +%s)"),
        vec!["cp", "build.log", "\"out/a b.log\"", "$(date +%s)"]
    );
}

#[test]
fn test_GH230_pipeline_splits_on_single_pipe_only() {
    assert_eq!(pipeline_segments("a | b"), vec!["a ", " b"]);
    assert_eq!(pipeline_segments("a || b"), vec!["a || b"]);
    assert_eq!(pipeline_segments("echo \"a|b\""), vec!["echo \"a|b\""]);
}

#[test]
fn test_GH230_command_word_skips_prefixes() {
    assert_eq!(command_word("sudo cp a b").as_deref(), Some("cp"));
    assert_eq!(command_word("TZ=UTC date +%s").as_deref(), Some("date"));
    assert_eq!(command_word("/usr/bin/tar -cf x").as_deref(), Some("tar"));
    assert_eq!(command_word("if cp a b; then").as_deref(), Some("cp"));
    assert_eq!(command_word("   ").as_deref(), None);
}

#[test]
fn test_GH230_redirect_kinds() {
    assert!(matches!(
        redirect_of("echo x > out.txt"),
        Some(Redirect::Truncate("out.txt"))
    ));
    assert!(matches!(
        redirect_of("echo x >> out.txt"),
        Some(Redirect::Append("out.txt"))
    ));
    assert!(matches!(redirect_of("echo x >&2"), Some(Redirect::Fd)));
    assert!(redirect_of("echo \"a > b\"").is_none());
}

#[test]
fn test_GH230_sinkless_targets_are_recognised() {
    assert!(is_sinkless("/dev/null"));
    assert!(is_sinkless("\"/dev/null\""));
    assert!(!is_sinkless("build.stamp"));
}

// ---------------------------------------------------------------------------
// assignment vs environment prefix
// ---------------------------------------------------------------------------

#[test]
fn test_GH230_assignment_target_basic() {
    assert_eq!(
        assignment_target("TS=$(date +%s)").map(|(n, _)| n),
        Some("TS".to_string())
    );
    assert_eq!(
        assignment_target("export FOO=\"a b\"").map(|(n, _)| n),
        Some("FOO".to_string())
    );
    assert_eq!(
        assignment_target("readonly local X=1").map(|(n, _)| n),
        Some("X".to_string())
    );
}

#[test]
fn test_GH230_env_prefix_is_not_an_assignment() {
    // `TZ=UTC date +%s` runs `date`; it does not assign the timestamp anywhere.
    assert_eq!(assignment_target("TZ=UTC date +%s"), None);
    assert_eq!(assignment_target("cp a b"), None);
}

#[test]
fn test_GH230_assignment_tolerates_trailing_comment() {
    assert_eq!(
        assignment_target("TS=$(date +%s)  # why").map(|(n, _)| n),
        Some("TS".to_string())
    );
}

// ---------------------------------------------------------------------------
// variable references
// ---------------------------------------------------------------------------

#[test]
fn test_GH230_references_forms() {
    assert!(references("cp a \"b_$TS.log\"", "TS"));
    assert!(references("cp a \"b_${TS}.log\"", "TS"));
    assert!(references("echo \"${TS:-x}\"", "TS"));
    assert!(!references("cp a \"b_$TSX.log\"", "TS"));
    assert!(!references("echo TS", "TS"));
}

// ---------------------------------------------------------------------------
// build-id names
// ---------------------------------------------------------------------------

#[test]
fn test_GH230_build_id_names_are_normalised() {
    assert!(is_build_id_name("BUILD_ID"));
    assert!(is_build_id_name("release"));
    assert!(is_build_id_name("image_tag"));
    assert!(!is_build_id_name("TIMESTAMP"));
    assert!(!is_build_id_name("LOG_FILE"));
}

// ---------------------------------------------------------------------------
// classify_sink
// ---------------------------------------------------------------------------

fn class_of(code: &str, var: &str) -> SinkClass {
    classify_sink(code, &Needle::Var(var))
}

#[test]
fn test_GH230_classify_artifact_commands() {
    assert_eq!(class_of("cp a \"b_$TS\"", "TS"), SinkClass::Reproducible);
    assert_eq!(
        class_of("tar -czf \"x_$TS.tgz\" d", "TS"),
        SinkClass::Reproducible
    );
    assert_eq!(
        class_of("docker build -t \"img:$TS\" .", "TS"),
        SinkClass::Reproducible
    );
}

#[test]
fn test_GH230_classify_logging() {
    assert_eq!(class_of("echo \"$TS\" >> app.log", "TS"), SinkClass::Benign);
    assert_eq!(class_of("logger \"$TS\"", "TS"), SinkClass::Benign);
    assert_eq!(class_of("echo \"$TS\"", "TS"), SinkClass::Benign);
    assert_eq!(
        class_of("printf '%s' \"$TS\" | tee -a app.log", "TS"),
        SinkClass::Benign
    );
}

#[test]
fn test_GH230_classify_pipeline_head_is_not_the_sink() {
    // `echo` feeding a pipe does NOT reach the terminal, so it must not be
    // treated as benign on the strength of being an `echo`.
    assert_eq!(
        class_of("echo \"$TS\" | sha256sum > sums.txt", "TS"),
        SinkClass::Reproducible
    );
    assert_eq!(
        class_of("echo \"$TS\" | some_helper", "TS"),
        SinkClass::Unknown
    );
}

#[test]
fn test_GH230_classify_comparisons_are_benign() {
    assert_eq!(
        class_of("if [ \"$TS\" -gt 100 ]; then", "TS"),
        SinkClass::Benign
    );
    assert_eq!(class_of("[ \"$TS\" -gt 100 ]", "TS"), SinkClass::Benign);
}

#[test]
fn test_GH230_classify_unknown_is_the_default() {
    assert_eq!(class_of("send_metric \"$TS\"", "TS"), SinkClass::Unknown);
    assert_eq!(
        class_of("curl -d \"$TS\" http://x", "TS"),
        SinkClass::Unknown
    );
}

#[test]
fn test_GH230_classify_append_to_timestamped_name_is_reproducible() {
    // The *file name* is timestamped, so a fresh artifact appears every run.
    assert_eq!(
        class_of("echo x >> \"log_$TS.txt\"", "TS"),
        SinkClass::Reproducible
    );
}

// ---------------------------------------------------------------------------
// analyze
// ---------------------------------------------------------------------------

#[test]
fn test_GH230_analyze_default_deny_on_unused_capture() {
    let uses = analyze("ts=$(date +%s)\n");
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].class, SinkClass::Unknown);
    assert_eq!(uses[0].var.as_deref(), Some("ts"));
}

#[test]
fn test_GH230_analyze_is_deterministic_with_many_taints() {
    // Insertion-ordered taints: repeated runs must agree byte for byte.
    let src = "A=$(date +%s)\nB=$(date +%s)\nC=$(date +%s)\ncp x \"o_$A$B$C\"\n";
    let first = format!("{:?}", analyze(src));
    for _ in 0..25 {
        assert_eq!(format!("{:?}", analyze(src)), first);
    }
}

#[test]
fn test_GH230_analyze_skips_quoted_heredoc_bodies() {
    let src = "cat > g.sh <<'EOF'\nTS=$(date +%s)\nEOF\n";
    assert!(analyze(src).is_empty());
}

#[test]
fn test_GH230_analyze_source_date_epoch_clears_downstream() {
    let src = "D=\"$(date -u -d \"@${SOURCE_DATE_EPOCH:-0}\" +%Y%m%d)\"\ncp a \"b_$D\"\n";
    assert!(analyze(src).is_empty());
}

#[test]
fn test_GH230_analyze_span_matches_legacy_columns() {
    // The pattern priority (`date +%s` before `$(date`) is preserved so spans
    // are byte-identical to the pre-GH-230 output.
    let uses = analyze("RELEASE=\"release-$(date +%s)\"\n");
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].col, 20);
    assert_eq!(uses[0].len, 8);
}
