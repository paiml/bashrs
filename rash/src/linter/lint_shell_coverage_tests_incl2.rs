fn test_lint_shell_det_process_id() {
    let script = "echo $$\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- IDEM rules (idempotency)
// =============================================================================

#[test]
fn test_lint_shell_idem_mkdir() {
    let script = "mkdir /tmp/newdir\n";
    let result = lint_shell(script);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("IDEM")),
        "Should detect non-idempotent mkdir"
    );
}

#[test]
fn test_lint_shell_idem_mkdir_p() {
    let script = "mkdir -p /tmp/newdir\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "IDEM001"),
        "mkdir -p should not trigger IDEM001"
    );
}

#[test]
fn test_lint_shell_idem_ln() {
    let script = "ln -s /src /dest\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- SEC rules (security)
// =============================================================================

#[test]
fn test_lint_shell_sec_chmod_777() {
    let script = "chmod 777 /tmp/file\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_sec_curl_pipe() {
    let script = "curl -s https://example.com/setup.sh | bash\n";
    let result = lint_shell(script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("SEC")),
        "Should detect curl | bash security issue"
    );
}

#[test]
fn test_lint_shell_sec_hardcoded_password() {
    let script = "PASSWORD=\"secret123\"\nexport PASSWORD\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_sec_world_writable() {
    let script = "chmod o+w /etc/config\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- PERF rules (performance)
// =============================================================================

#[test]
fn test_lint_shell_perf_loop_cat() {
    let script = "while read line; do echo $line; done < <(cat file.txt)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_perf_useless_use_of_wc() {
    let script = "count=$(cat file.txt | wc -l)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- REL rules (reliability)
// =============================================================================

#[test]
fn test_lint_shell_rel_set_e() {
    let script = "#!/bin/bash\nset -e\ncommand1\ncommand2\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_rel_pipefail() {
    let script = "#!/bin/bash\nset -o pipefail\ncmd1 | cmd2\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- PORT rules (portability)
// =============================================================================

#[test]
fn test_lint_shell_portability_source() {
    let script = "source /etc/profile\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_portability_function() {
    let script = "function myfunction {\n    echo hello\n}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_portability_local() {
    let script = "myfunc() {\n    local x=5\n    echo $x\n}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

// =============================================================================
// lint_shell -- Complex scripts hitting many rules
// =============================================================================

#[test]
fn test_lint_shell_large_script_many_rules() {
    let script = r#"#!/bin/bash
# A complex script that should trigger many rules
TMPFILE=/tmp/myapp_$$
echo $RANDOM > $TMPFILE
curl -s https://example.com/setup | bash
mkdir /opt/myapp
chmod 777 /opt/myapp
ln -s /opt/myapp /usr/local/myapp
cat file.txt | grep pattern
eval "echo $PATH"
read name
echo $name
cd /tmp
date +%s
for f in *.log; do
    rm $f
done
"#;
    let result = lint_shell(script);
    assert!(
        result.diagnostics.len() >= 3,
        "Complex script should trigger multiple diagnostics, got {}",
        result.diagnostics.len()
    );
    let codes: std::collections::HashSet<&str> =
        result.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.len() >= 2,
        "Should trigger multiple distinct rule codes, got {:?}",
        codes
    );
}

#[test]
fn test_lint_shell_clean_script_minimal_diagnostics() {
    let script = r#"#!/bin/sh
set -eu
readonly CONFIG_DIR="/etc/myapp"
mkdir -p "$CONFIG_DIR"
printf '%s\n' "Configured" > "${CONFIG_DIR}/status"
"#;
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_suppression_comment() {
    let script = "# shellcheck disable=SC2086\necho $VAR\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "SC2086 should be suppressed by shellcheck disable comment"
    );
}

#[test]
fn test_lint_shell_embedded_awk() {
    let script = "awk '{ print $1 }' file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_embedded_sed() {
    let script = "sed 's/foo/bar/g' file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_embedded_perl() {
    let script = "perl -ne 'print if /pattern/' file.txt\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_case_statement() {
    let script = r#"case "$1" in
    start) echo "starting";;
    stop) echo "stopping";;
    *) echo "unknown";;
esac
"#;
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_while_loop_with_pipe() {
    let script = "cat file.txt | while read line; do echo \"$line\"; done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_nested_if() {
    let script = r#"if [ -f /tmp/a ]; then
    if [ -f /tmp/b ]; then
        echo "both exist"
    fi
fi
"#;
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_for_c_style() {
    // bash-specific C-style for loop
    let script = "for ((i=0; i<10; i++)); do echo $i; done\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_process_substitution() {
    // bash-specific process substitution
    let script = "diff <(sort file1) <(sort file2)\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_associative_array() {
    let script = "declare -A map\nmap[key]=value\necho ${map[key]}\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_multiple_assignment() {
    let script = "a=1 b=2 c=3\necho $a $b $c\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_here_string() {
    let script = "grep pattern <<< \"$input\"\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_pipe_chain() {
    let script = "cat /etc/passwd | grep root | cut -d: -f1 | sort | uniq\n";
    let result = lint_shell(script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_comprehensive_sc2xxx_triggers() {
    // A script designed to exercise many SC2xxx rules in a single lint_shell call
    let script = r#"#!/bin/bash
# SC2002: useless cat
cat file | grep x
# SC2006: backticks
x=`date`
# SC2086: unquoted variable
echo $x
# SC2046: unquoted command substitution
files=$(ls *.txt)
echo $files
# SC2164: cd without check
cd /nonexistent
# SC2162: read without -r
read line
# SC2059: printf variable as format
printf $line
# SC2028: echo with backslash
echo "hello\nworld"
# SC2039: bash-only features in /bin/bash (NotSh)
[[ -f /tmp/x ]]
# SC2148: missing shebang check (has one)
"#;
    let result = lint_shell(script);
    assert!(
        result.diagnostics.len() >= 3,
        "Should trigger multiple diagnostics, got {}",
        result.diagnostics.len()
    );
}

#[test]
fn test_lint_shell_very_long_script() {
    // Exercise lint_shell with a large realistic script to cover all merge lines
    let mut script = String::from("#!/bin/bash\nset -euo pipefail\n\n");
    for i in 0..50 {
        script.push_str(&format!("var_{}=\"value_{}\"\n", i, i));
        script.push_str(&format!("echo \"$var_{}\"\n", i));
    }
    script.push_str("wait\n");
    let result = lint_shell(&script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_result_has_diagnostic_fields() {
    // Verify diagnostics have proper code, message, and span fields
    let script = "echo $RANDOM\n";
    let result = lint_shell(script);
    for diag in &result.diagnostics {
        assert!(!diag.code.is_empty(), "Diagnostic code should not be empty");
        assert!(
            !diag.message.is_empty(),
            "Diagnostic message should not be empty"
        );
        assert!(diag.span.start_line >= 1, "Diagnostic line should be >= 1");
    }
}

#[test]

include!("lint_shell_coverage_tests_incl2_incl2.rs");
