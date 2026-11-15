//! bashrs#524: SC2086 in unquoted heredoc bodies.
//! Plain body text is not word-split; command substitutions inside it are.
use super::*;

fn n(src: &str) -> usize {
    check(src).diagnostics.len()
}

#[test]
fn test_524_plain_text_in_unquoted_body_is_quiet() {
    assert_eq!(n("cat <<EOF\nhello $name and ${other}\nEOF\n"), 0);
}

#[test]
fn test_524_command_substitution_in_body_is_flagged() {
    assert_eq!(n("cat <<EOF\nfiles: $(ls $dir)\nEOF\n"), 1);
}

#[test]
fn test_524_backtick_substitution_in_body_is_flagged() {
    assert_eq!(n("cat <<EOF\nfiles: `ls $dir`\nEOF\n"), 1);
}

#[test]
fn test_524_multiline_command_substitution_is_flagged() {
    let src = "cat <<EOF\n$(\n  ls $dir\n)\nafter $plain\nEOF\n";
    let d = check(src).diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].span.start_line, 3);
}

#[test]
fn test_524_arithmetic_in_body_is_quiet() {
    assert_eq!(n("cat <<EOF\nsum: $(( $a + $b ))\nEOF\n"), 0);
}

#[test]
fn test_524_multiline_arithmetic_in_body_is_quiet() {
    assert_eq!(n("cat <<EOF\n$((\n  $a + $b\n))\nEOF\n"), 0);
}

#[test]
fn test_524_escaped_openers_are_quiet() {
    assert_eq!(n("cat <<EOF\n\\$(ls $dir) \\`ls $dir\\`\nEOF\n"), 0);
}

#[test]
fn test_524_code_outside_body_still_flagged() {
    assert_eq!(n("cat <<EOF\ntext\nEOF\nrm $f\n"), 1);
}

#[test]
fn test_524_quoted_delimiter_body_is_quiet() {
    assert_eq!(n("cat <<'EOF'\n$(ls $dir)\nEOF\n"), 0);
}
