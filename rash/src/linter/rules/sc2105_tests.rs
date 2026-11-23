//! Tests for SC2105 (break/continue outside a loop), split out of sc2105.rs (#544).
//! A move only: same tests, no rule change.

use super::*;

#[test]
fn test_sc2105_break_outside_loop() {
    let code = r#"
if [ "$skip" = "true" ]; then
break
fi
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SC2105");
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert!(result.diagnostics[0].message.contains("break"));
}

#[test]
fn test_sc2105_continue_outside_loop() {
    let code = r#"
if [ ! -r "$file" ]; then
continue
fi
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SC2105");
    assert!(result.diagnostics[0].message.contains("continue"));
}

#[test]
fn test_sc2105_break_in_while_loop_ok() {
    let code = r#"
while read -r line; do
if [ "$line" = "STOP" ]; then
    break
fi
done
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2105_continue_in_for_loop_ok() {
    let code = r#"
for file in *.txt; do
if [ ! -r "$file" ]; then
    continue
fi
echo "$file"
done
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2105_nested_loops() {
    let code = r#"
for i in 1 2 3; do
for j in a b c; do
    if [ "$j" = "b" ]; then
        continue
    fi
done
done
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2105_until_loop_ok() {
    let code = r#"
until [ "$count" -eq 10 ]; do
if [ "$count" -eq 5 ]; then
    break
fi
count=$((count + 1))
done
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2105_select_loop_ok() {
    let code = r#"
select option in "Continue" "Exit"; do
if [ "$option" = "Exit" ]; then
    break
fi
done
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2105_break_after_loop() {
    let code = r#"
for i in 1 2 3; do
echo "$i"
done
break
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sc2105_no_loops_or_breaks() {
    let code = r#"
echo "Hello"
ls -la
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2105_function_without_loop() {
    let code = r#"
function process() {
if [ "$1" = "skip" ]; then
    continue
fi
}
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
}

/// PMAT-244 (contracts/linter-lexer-context-v1.yaml F-SC2105-PMAT244-*):
/// a one-line loop must be read as a loop, not as three independent
/// keyword-counts computed for the whole physical line.

#[test]
fn test_PMAT255_pmat244_v1_inline_andand_break_is_clean() {
    let code = "for m in a b; do [[ $m == b ]] && { x=1; break; }; done\n";
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "break is inside the one-line for-loop, got {:?}",
        result.diagnostics
    );
}

#[test]
fn test_PMAT255_pmat244_v2_inline_if_break_is_clean() {
    let code = "for m in a b; do if [[ $m == b ]]; then x=1; break; fi; done\n";
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "break is inside the one-line for-loop, got {:?}",
        result.diagnostics
    );
}

#[test]
fn test_PMAT255_pmat244_v3_multiline_still_clean() {
    let code = r#"
for m in a b; do
if [[ $m == b ]]; then
    x=1
    break
fi
done
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);

    let while_code = "while true; do\n    break\ndone\n";
    let while_result = check(while_code);
    assert_eq!(while_result.diagnostics.len(), 0);

    let while_inline = "while true; do break; done\n";
    let while_inline_result = check(while_inline);
    assert_eq!(while_inline_result.diagnostics.len(), 0);
}

#[test]
fn test_PMAT255_pmat244_v5_toplevel_break_still_fires() {
    let code = "break\n";
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SC2105");
}

// #535 differential table: (input, findings). Every count was measured with
// BOTH the origin/main binary and this tree's binary; a row differs from
// main only where marked `// main: N`: `break`/`continue` is then certainly
// not a command (part of a larger word, a whole-command assignment value, or
// a plain argument of `echo`/`printf`). Every other shape fires, as on main.
const TABLE: &[(&str, usize)] = &[
    ("break\n", 1),
    ("  continue\n", 1),
    ("\tbreak\n", 1),
    ("break 2\n", 1),
    ("echo hi; break\n", 1),
    ("true && break\n", 1),
    ("false || continue\n", 1),
    ("echo x | break\n", 1),
    ("if true; then break; fi\n", 1),
    ("(break)\n", 1),
    ("( continue )\n", 1),
    ("{ break; }\n", 1),
    ("f() {\n  break\n}\n", 1),
    ("break # note\n", 1),
    ("case \"$1\" in q) break ;; esac\n", 1),
    ("case $1 in *) continue;; esac\n", 1),
    ("true; break; continue\n", 2),
    ("break;break\n", 2),
    ("! break\n", 1),
    ("time break\n", 1),
    ("time -p break\n", 1),
    ("command break\n", 1),
    ("builtin break\n", 1),
    ("eval break\n", 1),
    ("eval \"break\"\n", 0),
    ("x=`break`\n", 1),
    ("`break`\n", 1),
    ("$(break)\n", 1),
    ("echo $(break)\n", 1),
    ("echo `break`\n", 1),
    ("FOO=1 break\n", 1),
    ("FOO=1 x=break foo\n", 1),
    ("x=break foo\n", 1),
    ("x=break # c\n", 1),
    ("break>/dev/null\n", 1),
    ("break>&2\n", 1),
    ("break 2>&1\n", 1),
    (">/dev/null break\n", 1),
    ("\"break\"\n", 0),
    ("'break'\n", 0),
    ("\\break\n", 1),
    ("break)\n", 1),
    ("break;\n", 1),
    ("if true; then\n\tbreak\nfi\n", 1),
    ("while true; do :; done; break\n", 1),
    ("echo \"a; break\"\n", 0),
    ("echo 'a; break'\n", 0),
    ("echo \"$x\" break\n", 1),
    ("printf '%s\\n' break\n", 1),
    ("echo a || break\n", 1),
    ("echo then break\n", 0),                                      // main: 1
    ("echo break\n", 0),                                           // main: 1
    ("echo x break\n", 0),                                         // main: 1
    ("printf %s break\n", 0),                                      // main: 1
    ("echo x && echo break\n", 0),                                 // main: 1
    ("expect escaped-line-break-fails    1 \"$tmp/esc-nl\"\n", 0), // main: 1
    ("echo -break- x\n", 0),                                       // main: 1
    ("echo no-break\n", 0),                                        // main: 1
    ("echo --break\n", 0),                                         // main: 1
    ("echo continue-on-error\n", 0),                               // main: 1
    ("cat break.txt\n", 0),                                        // main: 1
    ("ls ./break\n", 0),                                           // main: 1
    ("x=break\n", 0),                                              // main: 1
    ("FOO=1 x=break\n", 0),                                        // main: 1
    ("break=1\n", 0),                                              // main: 1
    ("echo ${break}\n", 0),                                        // main: 1
    ("echo $break\n", 0),                                          // main: 1
    ("echo breakfast\n", 0),
    ("# break\n", 0),
    ("for i in 1; do break; done\n", 0),
    ("while true; do\n  if x; then continue; fi\ndone\n", 0),
];

#[test]
fn test_PMAT535_differential_table() {
    assert!(TABLE.len() >= 70);
    // Through the full lint pipeline, as the binary runs it.
    let bad: Vec<_> = TABLE
        .iter()
        .filter_map(|(code, want)| {
            let got = crate::linter::rules::lint_shell(code)
                .diagnostics
                .iter()
                .filter(|d| d.code == "SC2105")
                .count();
            (got != *want).then(|| format!("{code:?}: got {got}, want {want}"))
        })
        .collect();
    assert!(bad.is_empty(), "wrong counts: {bad:#?}");
}

#[test]
fn test_PMAT535_span_points_at_keyword() {
    let result = check("echo hi; break\n");
    let d = &result.diagnostics[0];
    assert_eq!((d.span.start_col, d.span.end_col), (10, 15));
}
