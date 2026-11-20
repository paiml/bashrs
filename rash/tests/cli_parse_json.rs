//! `bashrs parse --format json`: one fixture per shell form, fail-closed cases,
//! and a property test that `commands` is exactly the Simple nodes of `ast`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use assert_cmd::Command;
use serde_json::Value;

fn run(src: &str) -> (i32, Value) {
    let out = Command::cargo_bin("bashrs")
        .unwrap()
        .args(["parse", "--format", "json", "-"])
        .write_stdin(src.to_string())
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("not JSON ({e}): {text}"));
    (out.status.code().unwrap_or(-1), v)
}

fn commands(src: &str) -> Vec<Value> {
    let (code, v) = run(src);
    assert_eq!(code, 0, "expected exit 0 for {src:?}, got {v}");
    v["commands"].as_array().expect("commands array").clone()
}

fn argv(c: &Value) -> Vec<String> {
    c["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| match w {
            Value::String(s) => s.clone(),
            other => format!("<dyn:{}>", other["text"].as_str().unwrap_or("")),
        })
        .collect()
}

fn find<'a>(cmds: &'a [Value], words: &[&str]) -> &'a Value {
    cmds.iter()
        .find(|c| argv(c).iter().map(String::as_str).collect::<Vec<_>>() == words)
        .unwrap_or_else(|| panic!("no command {words:?} in {cmds:#?}"))
}

fn ctx(c: &Value) -> Vec<String> {
    c["context"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn wraps(c: &Value) -> Vec<String> {
    c["wrappers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn assert_unparsed(src: &str) {
    let (code, v) = run(src);
    assert_eq!(code, 2, "expected exit 2 for {src:?}, got {v}");
    assert!(
        v.get("commands").is_none(),
        "partial commands for {src:?}: {v}"
    );
    let u = v["unparsed"].as_array().expect("unparsed array");
    assert!(!u.is_empty());
    assert!(u[0]["line"].as_u64().unwrap() >= 1);
    assert!(!u[0]["reason"].as_str().unwrap().is_empty());
}

const RM: [&str; 4] = ["docker", "rm", "-f", "x"];

// ---- one fixture per form -----------------------------------------------

#[test]
fn form_plain_command_with_line() {
    let c = commands("echo hi\n\ndocker rm -f x\n");
    let d = find(&c, &RM);
    assert_eq!(d["line"], 3);
    assert!(ctx(d).is_empty() && wraps(d).is_empty());
}

#[test]
fn form_pipeline() {
    let c = commands("docker ps | xargs docker rm -f x\n");
    assert!(ctx(find(&c, &["docker", "ps"])).contains(&"pipeline".to_string()));
    let x = find(&c, &["docker", "rm", "-f", "x"]);
    assert_eq!(wraps(x), vec!["xargs"]);
}

#[test]
fn form_and_or_and_background() {
    let c = commands("true && docker rm -f x || true &\n");
    assert!(ctx(find(&c, &RM)).contains(&"background".to_string()));
}

#[test]
fn form_subshell_and_group() {
    let c = commands("( docker rm -f x )\n{ docker rm -f x; }\n");
    let both: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(both.len(), 2);
    assert!(ctx(both[0]).contains(&"subshell".to_string()));
    assert!(ctx(both[1]).contains(&"group".to_string()));
}

#[test]
fn form_if_elif_else() {
    let c = commands("if a; then b; elif c; then docker rm -f x; else d; fi\n");
    assert!(ctx(find(&c, &RM)).contains(&"if".to_string()));
    find(&c, &["a"]);
    find(&c, &["d"]);
}

#[test]
fn form_while_until_for() {
    let c = commands("while read l; do docker rm -f x; done\nuntil false; do docker rm -f x; done\nfor i in 1 2; do docker rm -f x; done\n");
    let ctxs: Vec<_> = c.iter().filter(|c| argv(c) == RM).map(ctx).collect();
    assert!(ctxs[0].contains(&"while".to_string()));
    assert!(ctxs[1].contains(&"until".to_string()));
    assert!(ctxs[2].contains(&"for".to_string()));
}

#[test]
fn form_select() {
    let c = commands("select v in a b; do docker rm -f x; done\n");
    assert!(ctx(find(&c, &RM)).contains(&"select".to_string()));
}

#[test]
fn form_coproc_plain_and_named() {
    let c = commands("coproc docker rm -f x\ncoproc NAME { docker rm -f x; }\n");
    let all: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(all.len(), 2);
    assert!(all.iter().all(|c| ctx(c).contains(&"coproc".to_string())));
}

#[test]
fn form_case_arms_including_done_paren_pattern() {
    let src =
        "case $1 in\n  done) docker rm -f x ;;\n  a|b) echo hi ;;\n  *) docker rm -f x\nesac\n";
    let c = commands(src);
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 2);
    assert!(ctx(hits[0]).contains(&"case".to_string()));
}

#[test]
fn form_case_arm_with_esac_pattern_and_one_line() {
    let c = commands("case x in esac) docker rm -f x;; esac\n");
    find(&c, &RM);
}

#[test]
fn form_function_both_syntaxes() {
    let c = commands(
        "f() { docker rm -f x; }\nfunction g { docker rm -f x; }\nh ()\n{\n docker rm -f x\n}\n",
    );
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 3);
    assert!(hits
        .iter()
        .all(|c| ctx(c).contains(&"function".to_string())));
}

#[test]
fn form_command_substitution_and_in_double_quotes() {
    let c =
        commands("x=$(docker rm -f x)\necho \"$(docker rm -f x)\" \"a ${B:-$(docker rm -f x)}\"\n");
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 3);
    assert!(hits
        .iter()
        .all(|c| ctx(c).contains(&"cmdsubst".to_string())));
}

#[test]
fn form_nested_command_substitution_with_parens_and_case() {
    let c = commands("echo $(case a in a) docker rm -f x;; esac) $(echo $(docker rm -f x))\n");
    assert_eq!(c.iter().filter(|c| argv(c) == RM).count(), 2);
}

#[test]
fn form_backticks_multiline_and_nested_escape() {
    let c = commands("echo `echo a\ndocker rm -f x`\necho \"`docker rm -f x`\"\necho `echo \\`docker rm -f x\\``\n");
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0]["line"], 2, "multiline backtick keeps source line");
    assert!(ctx(hits[0]).contains(&"backtick".to_string()));
}

#[test]
fn form_process_substitution() {
    let c = commands("diff <(docker rm -f x) >(docker rm -f x)\n");
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|c| ctx(c).contains(&"procsub".to_string())));
}

#[test]
fn form_heredoc_fed_while_read_loop() {
    let src = "while read l; do\n  docker rm -f x\ndone <<EOF\none\ntwo\nEOF\ndocker rm -f x\n";
    let c = commands(src);
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 2);
    assert_eq!(
        hits[1]["line"], 7,
        "command after the heredoc body keeps its line"
    );
}

#[test]
fn form_heredoc_body_substitution_runs_but_quoted_does_not() {
    let c = commands("cat <<EOF\n$(docker rm -f x)\nEOF\ncat <<'EOF'\n$(docker rm -f y)\nEOF\n");
    assert!(ctx(find(&c, &RM)).contains(&"heredoc".to_string()));
    assert!(!c.iter().any(|c| argv(c) == ["docker", "rm", "-f", "y"]));
}

#[test]
fn form_two_heredocs_on_one_line_and_tab_strip() {
    let c = commands("cat <<A <<-B\na\nA\n\tb\n\tB\ndocker rm -f x\n");
    assert_eq!(find(&c, &RM)["line"], 6);
}

#[test]
fn form_herestring_and_redirect_targets() {
    let c = commands("cat <<< \"$(docker rm -f x)\" > \"$(echo out)\" 2>&1\n");
    find(&c, &RM);
    find(&c, &["echo", "out"]);
}

#[test]
fn form_conditional_expression() {
    let c = commands("[[ -n $(docker rm -f x) && a == b ]] && echo ok\n");
    find(&c, &RM);
}

#[test]
fn form_arithmetic_expansion_with_substitution() {
    let c = commands("echo $(( $(docker rm -f x) + 1 )) $((1+2))\n");
    find(&c, &RM);
}

#[test]
fn form_dollar_paren_paren_that_is_a_subshell() {
    let c = commands("echo $( (docker rm -f x) )\n");
    find(&c, &RM);
}

#[test]
fn form_assignments_and_arrays_hold_substitutions() {
    let c = commands("A=$(docker rm -f x) B=1 env\nC=(a $(docker rm -f x))\n");
    find(&c, &RM);
}

#[test]
fn form_line_continuation_and_comments() {
    let c = commands("docker \\\n  rm -f x # docker rm -f z\n");
    assert_eq!(find(&c, &RM)["line"], 1);
    assert_eq!(c.len(), 1);
}

#[test]
fn form_ansi_c_is_dynamic_and_quotes_are_removed() {
    let c = commands("docker rm $'-f' '-f' \"x\"\n");
    find(&c, &["docker", "rm", "<dyn:$'-f'>", "-f", "x"]);
}

#[test]
fn form_dynamic_words_are_marked() {
    let (_, v) = run("docker rm -f $ID \"a$B\" *.txt\n");
    let w = &v["commands"][0]["argv"];
    assert_eq!(w[0], "docker");
    assert_eq!(w[3], serde_json::json!({"dynamic": true, "text": "$ID"}));
    assert_eq!(
        w[4],
        serde_json::json!({"dynamic": true, "text": "\"a$B\""})
    );
    assert_eq!(w[5]["dynamic"], true, "glob is not static");
}

#[test]
fn form_time_and_negation() {
    let c = commands("time ! docker rm -f x\n");
    find(&c, &RM);
}

// ---- wrappers ------------------------------------------------------------

#[test]
fn wrapper_sudo_env_command_exec_nohup_timeout_xargs() {
    for (src, w) in [
        ("sudo -u root docker rm -f x\n", vec!["sudo"]),
        ("env A=1 B=2 docker rm -f x\n", vec!["env"]),
        ("env -i -u HOME A=1 docker rm -f x\n", vec!["env"]),
        ("command docker rm -f x\n", vec!["command"]),
        ("command -p docker rm -f x\n", vec!["command"]),
        ("exec docker rm -f x\n", vec!["exec"]),
        ("exec -a name docker rm -f x\n", vec!["exec"]),
        ("nohup docker rm -f x\n", vec!["nohup"]),
        ("timeout 5 docker rm -f x\n", vec!["timeout"]),
        ("timeout -k 1 -s KILL 5s docker rm -f x\n", vec!["timeout"]),
        ("xargs docker rm -f x\n", vec!["xargs"]),
        ("xargs -n1 -P 4 -I{} docker rm -f x\n", vec!["xargs"]),
        (
            "sudo env A=1 nohup timeout 3 xargs -r docker rm -f x\n",
            vec!["sudo", "env", "nohup", "timeout", "xargs"],
        ),
        ("/usr/bin/sudo -E docker rm -f x\n", vec!["sudo"]),
        ("sudo -- docker rm -f x\n", vec!["sudo"]),
        ("A=1 sudo docker rm -f x\n", vec!["sudo"]),
    ] {
        let c = commands(src);
        let d = find(&c, &RM);
        assert_eq!(wraps(d), w, "wrappers for {src:?}");
    }
}

#[test]
fn wrapper_without_a_command_is_itself_the_command() {
    let c = commands("env\nsudo -v\ncommand -v docker\n");
    assert_eq!(argv(&c[0]), vec!["env"]);
    assert_eq!(argv(&c[1]), vec!["sudo", "-v"]);
    assert_eq!(argv(&c[2]), vec!["command", "-v", "docker"]);
    assert!(c.iter().all(|c| wraps(c).is_empty()));
}

#[test]
fn wrapper_dynamic_command_is_kept_dynamic() {
    let (_, v) = run("sudo $CMD rm\n");
    assert_eq!(v["commands"][0]["argv"][0]["dynamic"], true);
    assert_eq!(v["commands"][0]["wrappers"][0], "sudo");
}

#[test]
fn shell_dash_c_is_parsed_with_sh_c_context() {
    for src in [
        "sh -c 'docker rm -f x'\n",
        "bash -c \"docker rm -f x\"\n",
        "bash -lc 'docker rm -f x'\n",
        "sudo bash -o pipefail -c 'a | docker rm -f x'\n",
        "xargs -I{} sh -c 'docker rm -f x' _ {}\n",
        "sh -c 'sh -c \"docker rm -f x\"'\n",
        "sh -c 'echo a\ndocker rm -f x'\n",
    ] {
        let c = commands(src);
        let d = find(&c, &RM);
        assert!(ctx(d).contains(&"sh-c".to_string()), "context for {src:?}");
    }
}

#[test]
fn shell_dash_c_with_multiline_string_keeps_line() {
    let c = commands("echo a\nsh -c 'echo a\ndocker rm -f x'\n");
    assert_eq!(find(&c, &RM)["line"], 3);
}

#[test]
fn eval_string_is_parsed() {
    let c = commands("eval 'docker rm -f x'\neval docker rm -f x\n");
    let hits: Vec<_> = c.iter().filter(|c| argv(c) == RM).collect();
    assert_eq!(hits.len(), 2);
    assert!(ctx(hits[0]).contains(&"eval".to_string()));
}

// ---- fail closed ---------------------------------------------------------

#[test]
fn form_arithmetic_command_and_for_loop() {
    let c = commands("((x++)); (( $(docker rm -f x) > 1 ))\nfor ((i=0;i<3;i++)); do docker rm -f x; done\n(( (a) ))\n");
    assert_eq!(c.iter().filter(|c| argv(c) == RM).count(), 2);
    assert!(ctx(c.iter().rev().find(|c| argv(c) == RM).unwrap()).contains(&"for".to_string()));
}

#[test]
fn unparsed_unterminated_arithmetic() {
    assert_unparsed("(( 1 + \ndocker rm -f x\n");
    assert_unparsed("for (( i=0; do docker rm -f x; done\n");
}

#[test]
fn unparsed_named_fd_redirect() {
    assert_unparsed("exec {fd}>out\ndocker rm -f x\n");
}

#[test]
fn unparsed_unterminated_constructs() {
    for src in [
        "docker rm 'x\n",
        "docker rm \"x\n",
        "echo $(docker rm -f x\n",
        "echo `docker rm -f x\n",
        "echo ${A:-x\n",
        "if a; then b\n",
        "while a; do b\n",
        "case a in a) b;;\n",
        "{ docker rm -f x\n",
        "( docker rm -f x\n",
        "cat <<EOF\nnever closed\n",
        "[[ -n x\n",
        "echo $'abc\n",
    ] {
        assert_unparsed(src);
    }
}

#[test]
fn unparsed_stray_closers_and_operators() {
    for src in [
        "fi\n", "done\n", "esac\n", "}\n", ")\n", "a ;; b\n", "| a\n", "a &&\n",
    ] {
        assert_unparsed(src);
    }
}

#[test]
fn unparsed_unmodelled_wrapper_option() {
    assert_unparsed("sudo --weird docker rm -f x\n");
    assert_unparsed("sudo -Z docker rm -f x\n");
    assert_unparsed("xargs --bogus docker rm -f x\n");
    assert_unparsed("env -S 'docker rm -f x'\n");
    assert_unparsed("timeout -Q 5 docker rm\n");
}

#[test]
fn unparsed_shell_dash_c_without_string_or_bad_inner() {
    assert_unparsed("sh -c\n");
    assert_unparsed("bash -c 'if a; then'\n");
    assert_unparsed("eval 'docker rm \"x'\n");
}

#[test]
fn unparsed_dynamic_option_hiding_dash_c() {
    assert_unparsed("sh $OPT -c 'docker rm -f x'\n");
}

#[test]
fn unparsed_function_with_simple_body() {
    assert_unparsed("f() docker rm -f x\n");
}

#[test]
fn unparsed_bad_utf8_and_nul() {
    let out = Command::cargo_bin("bashrs")
        .unwrap()
        .args(["parse", "--format", "json", "-"])
        .write_stdin(vec![b'e', b'c', b'h', b'o', b' ', 0xff, b'\n'])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stdout).unwrap().contains("unparsed"));
    assert_unparsed("echo a\0b\n");
}

#[test]
fn unparsed_reports_the_failing_line() {
    let (code, v) = run("echo a\necho b\ndocker rm 'x\n");
    assert_eq!(code, 2);
    assert!(v["unparsed"][0]["line"].as_u64().unwrap() >= 3);
}

#[test]
fn file_argument_and_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("s.sh");
    std::fs::write(&p, "docker rm -f x\n").unwrap();
    let out = Command::cargo_bin("bashrs")
        .unwrap()
        .args(["parse", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(argv(&v["commands"][0]), RM);
    let out = Command::cargo_bin("bashrs")
        .unwrap()
        .args(["parse", "/nonexistent/zzz.sh"])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
}

#[test]
fn empty_and_comment_only_input_is_ok() {
    assert!(commands("").is_empty());
    assert!(commands("# nothing\n\n").is_empty());
}

// ---- property: commands == Simple nodes of ast ------------------------------

fn count_simple(v: &Value, lines: &mut Vec<u64>) {
    match v {
        Value::Object(m) => {
            if m.get("type").and_then(Value::as_str) == Some("simple")
                && !m["argv"].as_array().map(Vec::is_empty).unwrap_or(true)
            {
                lines.push(m["line"].as_u64().unwrap());
            }
            m.values().for_each(|x| count_simple(x, lines));
        }
        Value::Array(a) => a.iter().for_each(|x| count_simple(x, lines)),
        _ => {}
    }
}

const CORPUS: &[&str] = &[
    "a; b | c && d || e &\n",
    "if a; then b; elif c; then d; else e; fi\n",
    "while a; do b; done <<EOF\nx\nEOF\n",
    "for i in $(a) `b`; do c; done\n",
    "select s in a; do b; done\n",
    "coproc p { a; b; }\n",
    "case $(a) in x) b ;; *) c;; esac\n",
    "f() { a | { b; c; }; }\nfunction g { ( d ) ; }\n",
    "A=$(a) b <(c) >(d) \"$(e)\" `f` 2>&1\n",
    "sudo env A=1 sh -c 'a; b | c; $(d)' _ x\n",
    "eval 'a && b'; xargs -I{} sh -c 'c {}'\n",
    "[[ $(a) == b ]] && c\n",
    "echo $(( $(a) )) ${B:-$(c)} \"${D:-`e`}\"\n",
    "cat <<EOF\n$(a)\n`b`\nEOF\nc\n",
    "x=(a $(b) c); d\n",
    "a\\\n b\n",
];

#[test]
fn every_simple_node_in_ast_is_in_commands() {
    for src in CORPUS {
        let (code, v) = run(src);
        assert_eq!(code, 0, "{src:?}: {v}");
        let mut ast_lines = Vec::new();
        count_simple(&v["ast"], &mut ast_lines);
        let mut cmd_lines: Vec<u64> = v["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["line"].as_u64().unwrap())
            .collect();
        ast_lines.sort_unstable();
        cmd_lines.sort_unstable();
        assert!(!ast_lines.is_empty());
        assert_eq!(ast_lines, cmd_lines, "ast vs commands for {src:?}");
    }
}

#[test]
fn ast_nodes_carry_line_spans() {
    let (_, v) = run("a\nif b; then\n  c\nfi\n");
    assert_eq!(v["ast"][0]["line"], 1);
    assert_eq!(v["ast"][1]["type"], "if");
    assert_eq!(v["ast"][1]["line"], 2);
    assert_eq!(v["ast"][1]["end_line"], 4);
}

#[test]
fn form_declare_array_argument_with_substitution() {
    let c = commands("declare -a xs=($(docker rm -f x) b)\nlocal -a ys=(1 2)\n");
    assert!(ctx(find(&c, &RM)).contains(&"cmdsubst".to_string()));
    assert!(c
        .iter()
        .any(|x| argv(x).first().map(String::as_str) == Some("declare")));
}
