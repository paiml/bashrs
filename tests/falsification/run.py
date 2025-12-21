#!/usr/bin/env python3
import subprocess
import os
import sys
import tempfile
import json
from dataclasses import dataclass

@dataclass
class TestCase:
    id: str
    code: str
    forbidden: str
    desc: str

TEST_CASES = [
    # 6.1 Sudo and Permissions
    TestCase("F001", "sudo sh -c 'echo 1 > /f'", "SC2024", "Sudo redirect wrapped"),
    TestCase("F002", "echo 1 | sudo tee /f", "SC2024", "Sudo tee pattern"),
    TestCase("F003", "echo 1 | sudo tee /f >/dev/null", "SC2024", "Sudo tee output redirect"),
    TestCase("F004", "sudo -u user cmd > /tmp/f", "SC2024", "Sudo redirect writable target"),
    TestCase("F005", "sudo -v", "SC2024", "Sudo flag-only"),
    TestCase("F006", "sudo -k && sudo -n ls", "SC2024", "Sudo non-exec flags"),
    TestCase("F007", "sudo bash -c \"cmd | pipe\"", "SC2024", "Sudo internal pipes"),
    TestCase("F008", "pkexec cmd > /f", "SC2024", "pkexec redirect"),
    TestCase("F009", "doas cmd > /f", "SC2024", "doas redirect"),
    TestCase("F010", "sudo env PATH=$P cmd", "SC2024", "Sudo env wrapper"),

    # 6.2 Redirection and Pipes
    TestCase("F011", "cmd 2>&1 | other", "SC2069", "Stdout/err order"),
    TestCase("F012", "cmd >/dev/null 2>&1", "SC2069", "Silence pattern"),
    TestCase("F013", "cmd &> file", "SC2069", "Bash shorthand"),
    TestCase("F014", "exec 3>&1", "SC2069", "FD manipulation"),
    TestCase("F015", "cmd |& other", "SC2069", "Pipe-both shorthand"),
    TestCase("F016", "echo \"x\" >&2", "SC2069", "Stderr redirect"),
    TestCase("F017", "read -r x <<< \"str\"", "SC2069", "Here-string"),
    TestCase("F018", "cmd <(list)", "SC2069", "Process subst input"),
    TestCase("F019", "cmd > >(other)", "SC2069", "Process subst output"),
    TestCase("F020", "{ cmd; } > file", "SC2024", "Block redirection"),

    # 6.3 Quoting and Heredocs
    TestCase("F021", "cat << 'EOF'\n$var\nEOF", "SC2016", "Single quote heredoc literal"),
    TestCase("F022", "cat << \"EOF\"\n$var\nEOF", "SC2016", "Double quote heredoc"),
    TestCase("F023", "cat <<-'EOF'\n  literal\nEOF", "SC2016", "Indented literal heredoc"),
    TestCase("F024", "echo \"Don't\"", "SC2016", "Apostrophe in double quotes"),
    TestCase("F025", "echo 'Value: \"$var\"'", "SC2016", "Quotes inside literal"),
    TestCase("F026", "printf '%s\n' \"$v\"", "SC2059", "Constant format string"),
    TestCase("F027", "echo \"Only $ var\"", "SC2016", "Detached dollar sign"),
    TestCase("F028", "echo 'a'\''b'", "SC2016", "Concatted escaped single quote"),
    TestCase("F029", "find . -name '*.c'", "SC2035", "Find name glob"),
    TestCase("F030", "grep -r '*.c' .", "SC2035", "Grep regex pattern"),

    # 6.4 Variables and Parameters
    TestCase("F031", "echo \"${var:-default}\"", "SC2086", "Safe parameter expansion"),
    TestCase("F032", "echo \"${var#*}\"", "SC2086", "Manipulation safety"),
    TestCase("F033", "echo \"${!prefix@}\"", "SC2086", "Indirect expansion"),
    TestCase("F034", "echo \"${arr[@]}\"", "SC2068", "Double quote array distinct args"),
    TestCase("F035", "echo ${#arr[@]}", "SC2086", "Array count is numeric"),
    TestCase("F036", "(( var++ ))", "SC2086", "Arithmetic context"),
    TestCase("F037", "[[ -n $var ]]", "SC2086", "Test context"),
    TestCase("F038", "f() { local var; echo $var; }", "SC2034", "Local variable usage"),
    TestCase("F039", "export VAR=1", "SC2034", "Exported variable"),
    TestCase("F040", "readonly VAR=1", "SC2034", "Readonly variable"),
    TestCase("F041", "_unused_arg=1", "SC2034", "Unused prefix convention"),
    TestCase("F042", "typeset -n ref=$1", "SC2034", "Nameref"),
    TestCase("F043", "PS1='prompt'", "SC2034", "Shell variable PS1"),
    TestCase("F044", "PROMPT_COMMAND='cmd'", "SC2034", "Hook variable"),
    TestCase("F045", "trap 'echo $SIG' SIGINT", "SC2034", "Trap variable"),

    # 6.5 Control Flow
    TestCase("F046", "if true; then echo yes; fi", "Parser", "Inline if"),
    TestCase("F047", "case $x in *) ;; esac", "SC2154", "Case default coverage"),
    TestCase("F048", "for ((i=0;i<10;i++)); do echo $i; done", "SC2086", "C-style for loop"),
    TestCase("F049", "select x in list; do echo $x; done", "Parser", "Select construct"),
    TestCase("F050", "while read -r; do echo $REPLY; done < f", "SC2034", "Implicit REPLY"),
    TestCase("F051", "until [[ cond ]]; do echo x; done", "Parser", "Until loop"),
    TestCase("F052", "[ \"$a\" ] && [ \"$b\" ]", "SC2015", "Chain logic"),
    TestCase("F053", "! command", "Parser", "Negation pipeline"),
    TestCase("F054", "time command", "Parser", "Time keyword"),
    TestCase("F055", "coproc command", "Parser", "Coproc keyword"),
    TestCase("F056", "f() { return 0 2>/dev/null; }", "SC2086", "Return redirect"),
    TestCase("F057", "break 2", "Parser", "Break argument"),
    TestCase("F058", "continue 2", "Parser", "Continue argument"),
    TestCase("F059", "exit 0; echo unreachable", "SC2317", "Exit unreachable assumption"),
    TestCase("F060", "function f { cmd; }", "Parser", "Keyword function"),

    # 6.6 Builtins and Environment
    TestCase("F061", "echo $EUID", "SC2154", "EUID builtin"),
    TestCase("F062", "echo $UID", "SC2154", "UID builtin"),
    TestCase("F063", "echo $BASH_VERSION", "SC2154", "BASH_VERSION builtin"),
    TestCase("F064", "echo $PIPESTATUS", "SC2154", "PIPESTATUS builtin"),
    TestCase("F065", "echo $RANDOM", "SC2154", "RANDOM builtin"),
    TestCase("F066", "echo $LINENO", "SC2154", "LINENO builtin"),
    TestCase("F067", "echo $SECONDS", "SC2154", "SECONDS builtin"),
    TestCase("F068", "echo $PWD", "SC2154", "PWD builtin"),
    TestCase("F069", "echo $OLDPWD", "SC2154", "OLDPWD builtin"),
    TestCase("F070", "echo $SHLVL", "SC2154", "SHLVL builtin"),

    # 6.7 Subshells and Command Subs
    TestCase("F071", "( cd dir && cmd )", "SC2034", "Subshell scope"),
    TestCase("F072", "echo $(command)", "SC2034", "Command sub scope"),
    TestCase("F073", "var=$(cmd)", "SC2031", "Assignment output capture"),
    TestCase("F074", "var=\"$(cmd)\"", "SC2031", "Quoted assignment capture"),
    TestCase("F075", "echo $( < file )", "SC2002", "Useless cat equivalent"),
    TestCase("F076", "diff <(cmd1) <(cmd2)", "Parser", "Process subst args"),
    TestCase("F077", "exec > >(logger)", "Parser", "Process subst redirect"),
    TestCase("F078", "x=$( (cmd) )", "Parser", "Nested subshells"),
    TestCase("F079", "x=$( { cmd; } )", "Parser", "Block inside cmd sub"),
    TestCase("F080", "x=`cmd`", "SC2006", "Backticks legacy support"),

    # 6.8 Traps and Signals
    TestCase("F081", "trap 'rm $f' EXIT", "SC2064", "Trap single quote"),
    TestCase("F082", "trap \"echo $v\" INT", "SC2064", "Trap double quote"),
    TestCase("F083", "kill -9 $$ ", "SC2086", "PID numeric"),
    TestCase("F084", "wait $!", "SC2086", "Background PID numeric"),
    TestCase("F085", "disown -h", "Parser", "Disown builtin"),
    TestCase("F086", "suspend -f", "Parser", "Suspend builtin"),
    TestCase("F087", "ulimit -n 1024", "Parser", "Ulimit builtin"),
    TestCase("F088", "umask 077", "Parser", "Umask builtin"),
    TestCase("F089", "set -e", "SC2034", "Set flags"),
    TestCase("F090", "shopt -s extglob", "Parser", "Shopt builtin"),

    # 6.9 Parsing and Formatting
    TestCase("F091", "echo # comment", "Parser", "Comments"),
    TestCase("F092", "echo \\# literal", "Parser", "Escaped hash"),
    TestCase("F093", "x=()", "Parser", "Empty array"),
    TestCase("F094", "x=([0]=a [2]=c)", "Parser", "Sparse array"),
    TestCase("F095", "x+=(\"new\")", "Parser", "Array append"),
    TestCase("F096", "[[ $x =~ ^[a-z]+$ ]]", "Parser", "Regex operator"),
    TestCase("F097", "echo *", "SC2035", "Naked glob"),
    TestCase("F098", "echo {1..10}", "Parser", "Brace expansion"),
    TestCase("F099", "echo {a,b,c}", "Parser", "Brace expansion list"),
    TestCase("F100", "echo $\'\\t\'", "Parser", "ANSI-C quoting"),

    # 6.10 Arrays
    TestCase("F101", "arr=(a b c); echo ${arr[0]}", "SC2086", "Array index access"),
    TestCase("F102", "arr=(\"$@\"); echo ${#arr[@]}", "SC2086", "Array from args"),
    TestCase("F103", "declare -A map; map[key]=val", "Parser", "Associative array declare"),
    TestCase("F104", "arr=(); arr+=(item)", "Parser", "Array append operator"),
    TestCase("F105", "echo \"${arr[*]}\"", "SC2086", "Array star expansion quoted"),
    TestCase("F106", "for i in \"${arr[@]}\"; do echo \"$i\"; done", "SC2086", "Array iteration quoted"),
    TestCase("F107", "unset arr[0]", "Parser", "Array element unset"),
    TestCase("F108", "arr=([0]=a [2]=c)", "SC2086", "Sparse array literal"),
    TestCase("F109", "echo ${!arr[@]}", "SC2086", "Array indices expansion"),
    TestCase("F110", "readarray -t lines < file", "Parser", "Readarray builtin"),

    # 6.11 String Operations
    TestCase("F111", "echo ${var:0:5}", "SC2086", "Substring extraction"),
    TestCase("F112", "echo ${var/old/new}", "SC2086", "Pattern substitution"),
    TestCase("F113", "echo ${var//old/new}", "SC2086", "Global substitution"),
    TestCase("F114", "echo ${var,,}", "SC2086", "Lowercase transform"),
    TestCase("F115", "echo ${var^^}", "SC2086", "Uppercase transform"),
    TestCase("F116", "echo ${#var}", "SC2086", "String length"),
    TestCase("F117", "echo ${var%suffix}", "SC2086", "Remove shortest suffix"),
    TestCase("F118", "echo ${var%%pattern}", "SC2086", "Remove longest suffix"),
    TestCase("F119", "echo ${var#prefix}", "SC2086", "Remove shortest prefix"),
    TestCase("F120", "echo ${var##pattern}", "SC2086", "Remove longest prefix"),
]

def run_test(case):
    # Create temp file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False) as tmp:
        tmp.write("#!/bin/bash\n")
        tmp.write(case.code + "\n")
        tmp_path = tmp.name

    try:
        # Run bashrs lint
        # Using release binary for speed
        binary_path = os.path.abspath("target/release/bashrs")
        if not os.path.exists(binary_path):
             # Fallback if running from a different CWD or not built
             binary_path = "target/debug/bashrs"
        
        cmd = [binary_path, "lint", "--format", "json", tmp_path]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        # Check output
        failed = False
        failure_reason = ""

        if result.returncode != 0 and "error" in result.stderr.lower() and not result.stdout.strip():
             # Basic execution failure (likely parser error)
             failed = True
             failure_reason = f"Execution failed: {result.stderr.strip()}"
        
        if not failed:
            try:
                # Filter out potential log lines from stdout
                stdout_str = result.stdout
                json_start = stdout_str.find('{')
                if json_start != -1:
                    json_str = stdout_str[json_start:]
                    output = json.loads(json_str)
                else:
                    output = [] # No JSON object found, maybe empty or error
                    if result.returncode != 0:
                         # If no JSON and error code, it's a crash or error
                         failed = True
                         failure_reason = f"No JSON found and exit code {result.returncode}: {result.stderr.strip()}"

                if not failed:
                    # Check for forbidden rule
                    for diag in output.get("diagnostics", []):
                        rule_id = diag.get("code")
                        if not rule_id:
                            rule_id = diag.get("rule", "")

                        if case.forbidden == "Parser":
                             pass 
                        
                        if rule_id == case.forbidden:
                            failed = True
                            failure_reason = f"Triggered forbidden rule {case.forbidden}"
                            break
                        
                        if case.forbidden == "Parser" and ("parse" in str(diag).lower() or "syntax" in str(diag).lower()):
                             failed = True
                             failure_reason = "Triggered Parser Error"
                             break

            except json.JSONDecodeError:
                # If not JSON, check stderr for parser panics or errors
                if result.returncode != 0:
                    failed = True
                    failure_reason = f"Non-JSON output (likely crash/panic): {result.stderr.strip()}"
                elif case.forbidden == "Parser" and result.returncode != 0:
                     failed = True
                     failure_reason = f"Parser failure: {result.stderr.strip()}"

        status = "FAIL" if failed else "PASS"
        print(f"[{status}] {case.id}: {case.desc}")
        if failed:
            print(f"    Code: {case.code}")
            print(f"    Reason: {failure_reason}")
            return False
        return True

    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)

def main():
    print(f"Running {len(TEST_CASES)} Falsification Tests...")
    passes = 0
    failures = 0
    
    for case in TEST_CASES:
        if run_test(case):
            passes += 1
        else:
            failures += 1
            
    print("""----------------------------------------""")
    print(f"Total: {len(TEST_CASES)}")
    print(f"Passed: {passes}")
    print(f"Failed: {failures}")
    
    if failures > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
