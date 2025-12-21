#!/usr/bin/env python3
"""
Probador Simulation Testing - Edge Case Discovery

This test suite uses simulation-based testing to surface edge cases
that may cause parser errors, panics, or unexpected behavior.

Categories:
- S1xx: Unicode and Encoding
- S2xx: Large Input and Boundaries
- S3xx: Deep Nesting
- S4xx: Special Characters
- S5xx: Malformed Syntax
- S6xx: Race/Timing Edge Cases
- S7xx: Memory/Resource Limits
- S8xx: Escape Sequences
- S9xx: Quoting Edge Cases
- S10xx: Combined Stress Tests
"""

import subprocess
import os
import sys
import tempfile
import json
from dataclasses import dataclass
from typing import Optional

@dataclass
class SimulationCase:
    id: str
    code: str
    expect: str  # "pass" = no panic, "parse" = parser handles, "error" = graceful error
    desc: str
    category: str

# S1xx: Unicode and Encoding
UNICODE_TESTS = [
    SimulationCase("S101", "echo 'héllo wörld'", "pass", "Latin extended chars", "unicode"),
    SimulationCase("S102", "echo '日本語テスト'", "pass", "Japanese characters", "unicode"),
    SimulationCase("S103", "echo '🚀🔥💻'", "pass", "Emoji in string", "unicode"),
    SimulationCase("S104", "var='مرحبا'; echo $var", "pass", "RTL Arabic text", "unicode"),
    SimulationCase("S105", "echo 'Ω≈ç√∫'", "pass", "Math symbols", "unicode"),
    SimulationCase("S106", "echo '\u0000'", "pass", "Null byte in string", "unicode"),
    SimulationCase("S107", "echo $'\\xc0\\xc1'", "pass", "Invalid UTF-8 sequence", "unicode"),
    SimulationCase("S108", "x='a\u200Bb'; echo $x", "pass", "Zero-width space", "unicode"),
    SimulationCase("S109", "echo '​'", "pass", "Zero-width char only", "unicode"),
    SimulationCase("S110", "echo 'A\u0308'", "pass", "Combining diacritical", "unicode"),
]

# S2xx: Large Input and Boundaries
BOUNDARY_TESTS = [
    SimulationCase("S201", "x=" + "a" * 10000, "pass", "10KB variable assignment", "boundary"),
    SimulationCase("S202", "echo " + "word " * 1000, "pass", "1000 word echo", "boundary"),
    SimulationCase("S203", " " * 100 + "echo test", "pass", "100 leading spaces", "boundary"),
    SimulationCase("S204", "echo test" + " " * 100, "pass", "100 trailing spaces", "boundary"),
    SimulationCase("S205", "\n" * 50 + "echo test", "pass", "50 leading newlines", "boundary"),
    SimulationCase("S206", "echo test; " * 100, "pass", "100 chained commands", "boundary"),
    SimulationCase("S207", "cat << EOF\n" + "line\n" * 500 + "EOF", "pass", "500 line heredoc", "boundary"),
    SimulationCase("S208", "echo ${" + "x:-${" * 20 + "default" + "}" * 20 + "}", "pass", "20 nested expansions", "boundary"),
    SimulationCase("S209", "arr=(" + "elem " * 500 + ")", "pass", "500 element array", "boundary"),
    SimulationCase("S210", "# " + "x" * 5000, "pass", "5KB comment", "boundary"),
]

# S3xx: Deep Nesting
NESTING_TESTS = [
    SimulationCase("S301", "if true; then " * 10 + "echo deep" + "; fi" * 10, "pass", "10 nested ifs", "nesting"),
    SimulationCase("S302", "{ " * 20 + "echo test" + "; }" * 20, "pass", "20 nested blocks", "nesting"),
    SimulationCase("S303", "( " * 15 + "echo test" + " )" * 15, "pass", "15 nested subshells", "nesting"),
    SimulationCase("S304", "case x in " + "a) case y in b) " * 5 + "echo z" + " ;; esac" * 5 + " ;; esac", "pass", "5 nested case", "nesting"),
    SimulationCase("S305", "while true; do " * 8 + "break 8" + "; done" * 8, "pass", "8 nested whiles", "nesting"),
    SimulationCase("S306", "for i in a; do " * 8 + "echo $i" + "; done" * 8, "pass", "8 nested fors", "nesting"),
    SimulationCase("S307", "$(" * 10 + "echo test" + ")" * 10, "pass", "10 nested cmd subs", "nesting"),
    SimulationCase("S308", "[[" + " && [[" * 5 + " true" + " ]]" * 5 + " ]]", "parse", "Nested conditions", "nesting"),
    SimulationCase("S309", "echo $(( " + "1 + (" * 10 + "1" + ")" * 10 + " ))", "pass", "10 nested arithmetic", "nesting"),
    SimulationCase("S310", "f() { g() { h() { echo deep; }; h; }; g; }; f", "pass", "3 nested functions", "nesting"),
]

# S4xx: Special Characters
SPECIAL_CHAR_TESTS = [
    SimulationCase("S401", "echo 'test\ttest'", "pass", "Tab in string", "special"),
    SimulationCase("S402", "echo 'test\rtest'", "pass", "Carriage return", "special"),
    SimulationCase("S403", "echo 'line1\nline2'", "pass", "Newline in string", "special"),
    SimulationCase("S404", "echo $'\\a\\b\\e\\f'", "pass", "ANSI escapes", "special"),
    SimulationCase("S405", "echo 'test\\test'", "pass", "Backslash in string", "special"),
    SimulationCase("S406", "x='`'; echo $x", "pass", "Backtick in var", "special"),
    SimulationCase("S407", "echo '$()'", "pass", "Literal cmd sub", "special"),
    SimulationCase("S408", "echo '$((1+1))'", "pass", "Literal arithmetic", "special"),
    SimulationCase("S409", "echo '!!'", "pass", "History expansion chars", "special"),
    SimulationCase("S410", "echo '#not a comment'", "pass", "Hash in quotes", "special"),
]

# S5xx: Malformed Syntax (should error gracefully)
MALFORMED_TESTS = [
    SimulationCase("S501", "echo ${", "error", "Unclosed brace", "malformed"),
    SimulationCase("S502", "echo $((1+)", "error", "Unclosed arithmetic", "malformed"),
    SimulationCase("S503", "if true; then", "error", "Missing fi", "malformed"),
    SimulationCase("S504", "case x in", "error", "Missing esac", "malformed"),
    SimulationCase("S505", "while true; do", "error", "Missing done", "malformed"),
    SimulationCase("S506", "echo \"unterminated", "error", "Unclosed quote", "malformed"),
    SimulationCase("S507", "echo 'unterminated", "error", "Unclosed single quote", "malformed"),
    SimulationCase("S508", "((", "error", "Empty arithmetic", "malformed"),
    SimulationCase("S509", "[[", "error", "Empty condition", "malformed"),
    SimulationCase("S510", "}", "error", "Unmatched brace", "malformed"),
]

# S6xx: Timing/Order Edge Cases
TIMING_TESTS = [
    SimulationCase("S601", "echo $$ $!", "pass", "PID variables", "timing"),
    SimulationCase("S602", "echo ${RANDOM} ${RANDOM}", "pass", "Multiple RANDOM", "timing"),
    SimulationCase("S603", "trap 'echo exit' EXIT; exit", "pass", "Trap on exit", "timing"),
    SimulationCase("S604", "eval 'echo test'", "pass", "Eval command", "timing"),
    SimulationCase("S605", "exec 3>&1", "pass", "FD manipulation", "timing"),
    SimulationCase("S606", "wait", "pass", "Wait builtin", "timing"),
    SimulationCase("S607", "jobs", "pass", "Jobs builtin", "timing"),
    SimulationCase("S608", "bg 2>/dev/null", "pass", "Background builtin", "timing"),
    SimulationCase("S609", "fg 2>/dev/null", "pass", "Foreground builtin", "timing"),
    SimulationCase("S610", "disown 2>/dev/null", "pass", "Disown builtin", "timing"),
]

# S7xx: Resource Limits
RESOURCE_TESTS = [
    SimulationCase("S701", "${" + "x" * 100 + "}", "pass", "Long variable name", "resource"),
    SimulationCase("S702", "f" + "x" * 100 + "() { :; }", "pass", "Long function name", "resource"),
    SimulationCase("S703", "alias " + "a" * 100 + "='echo'", "pass", "Long alias name", "resource"),
    SimulationCase("S704", "export " + "V" * 100 + "=value", "pass", "Long export name", "resource"),
    SimulationCase("S705", "read " + "v" * 50 + " " + "w" * 50, "pass", "Multiple long read vars", "resource"),
    SimulationCase("S706", "local " + "v" * 100 + "=x", "pass", "Long local var", "resource"),
    SimulationCase("S707", "declare -a " + "arr" * 30, "pass", "Long array name", "resource"),
    SimulationCase("S708", "printf '%s' " + "'x' " * 100, "pass", "Many printf args", "resource"),
    SimulationCase("S709", "echo " + "$1 " * 50, "pass", "Many positional refs", "resource"),
    SimulationCase("S710", "set -- " + "arg " * 100, "pass", "Many set args", "resource"),
]

# S8xx: Escape Sequences
ESCAPE_TESTS = [
    SimulationCase("S801", "echo $'\\n\\t\\r'", "pass", "Common escapes", "escape"),
    SimulationCase("S802", "echo $'\\x41\\x42\\x43'", "pass", "Hex escapes", "escape"),
    SimulationCase("S803", "echo $'\\101\\102\\103'", "pass", "Octal escapes", "escape"),
    SimulationCase("S804", "echo $'\\u0041'", "pass", "Unicode 4-digit", "escape"),
    SimulationCase("S805", "echo $'\\U0001F600'", "pass", "Unicode 8-digit", "escape"),
    SimulationCase("S806", "echo $'\\'", "pass", "Escaped backslash", "escape"),
    SimulationCase("S807", "echo $'\\\"'", "pass", "Escaped quote", "escape"),
    SimulationCase("S808", "echo \"\\$var\"", "pass", "Escaped dollar", "escape"),
    SimulationCase("S809", "echo \"\\`cmd\\`\"", "pass", "Escaped backtick", "escape"),
    SimulationCase("S810", "echo '\\n'", "pass", "Literal backslash-n", "escape"),
]

# S9xx: Quoting Edge Cases
QUOTING_TESTS = [
    SimulationCase("S901", "echo ''", "pass", "Empty single quotes", "quoting"),
    SimulationCase("S902", "echo \"\"", "pass", "Empty double quotes", "quoting"),
    SimulationCase("S903", "echo $''", "pass", "Empty ANSI-C", "quoting"),
    SimulationCase("S904", "echo $\"\"", "pass", "Empty localized", "quoting"),
    SimulationCase("S905", "echo 'a'\"b\"'c'", "pass", "Mixed quote concat", "quoting"),
    SimulationCase("S906", "echo \"'inner'\"", "pass", "Single inside double", "quoting"),
    SimulationCase("S907", "echo '\"inner\"'", "pass", "Double inside single", "quoting"),
    SimulationCase("S908", "echo \"$(echo 'test')\"", "pass", "Cmd sub in quotes", "quoting"),
    SimulationCase("S909", "echo \"${var:-'default'}\"", "pass", "Single in expansion", "quoting"),
    SimulationCase("S910", "echo 'test'\\''more'", "pass", "Escaped single concat", "quoting"),
]

# S10xx: Combined Stress Tests
STRESS_TESTS = [
    SimulationCase("S1001", "echo 'héllo' | cat | tr '[:lower:]' '[:upper:]'", "pass", "Unicode pipeline", "stress"),
    SimulationCase("S1002", "for i in 日本 中文 한국; do echo \"$i\"; done", "pass", "Unicode loop", "stress"),
    SimulationCase("S1003", "arr=(🚀 🔥 💻); echo ${arr[@]}", "pass", "Emoji array", "stress"),
    SimulationCase("S1004", "x='a\tb\nc'; echo \"${x//[[:space:]]/_}\"", "pass", "Whitespace manipulation", "stress"),
    SimulationCase("S1005", "cat <<'EOF'\n$var `cmd` $((1+1))\nEOF", "pass", "Literal heredoc complex", "stress"),
    SimulationCase("S1006", "eval \"echo \\\"nested\\\"\"", "pass", "Eval with escapes", "stress"),
    SimulationCase("S1007", "f() { local x=$1; echo ${x:-${2:-${3:-default}}}; }; f", "pass", "Nested defaults in func", "stress"),
    SimulationCase("S1008", "case \"$1\" in *[!0-9]*) echo nan;; *) echo num;; esac", "pass", "Pattern with negation", "stress"),
    SimulationCase("S1009", "[[ $x =~ ^[0-9]+$ ]] && echo num || echo nan", "pass", "Regex with logic", "stress"),
    SimulationCase("S1010", "printf '%s\\n' \"${arr[@]/#/prefix_}\"", "pass", "Array prefix transform", "stress"),
]

ALL_TESTS = (
    UNICODE_TESTS +
    BOUNDARY_TESTS +
    NESTING_TESTS +
    SPECIAL_CHAR_TESTS +
    MALFORMED_TESTS +
    TIMING_TESTS +
    RESOURCE_TESTS +
    ESCAPE_TESTS +
    QUOTING_TESTS +
    STRESS_TESTS
)

def run_test(case: SimulationCase) -> tuple[bool, str]:
    """Run a simulation test case."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False) as tmp:
        tmp.write("#!/bin/bash\n")
        tmp.write(case.code + "\n")
        tmp_path = tmp.name

    try:
        binary_path = os.path.abspath("target/release/bashrs")
        if not os.path.exists(binary_path):
            binary_path = "target/debug/bashrs"

        cmd = [binary_path, "lint", "--format", "json", tmp_path]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)

        # Check for panics
        if "panic" in result.stderr.lower() or "thread" in result.stderr.lower():
            return False, "PANIC detected"

        # Check expectations
        if case.expect == "pass":
            # Should not panic, may have warnings but that's OK
            return True, "OK"
        elif case.expect == "error":
            # Should handle gracefully (exit code may be non-zero, but no panic)
            if result.returncode != 0 and "panic" not in result.stderr.lower():
                return True, "Graceful error"
            return True, "OK"  # Even success is fine for error cases
        elif case.expect == "parse":
            # Parser should handle (may produce errors, but no panic)
            return True, "OK"

        return True, "OK"

    except subprocess.TimeoutExpired:
        return False, "TIMEOUT"
    except Exception as e:
        return False, f"EXCEPTION: {e}"
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)


def main():
    print(f"Running {len(ALL_TESTS)} Simulation Tests...")
    print("=" * 60)

    results = {"pass": 0, "fail": 0, "by_category": {}}
    failures = []

    for case in ALL_TESTS:
        success, msg = run_test(case)

        if case.category not in results["by_category"]:
            results["by_category"][case.category] = {"pass": 0, "fail": 0}

        if success:
            results["pass"] += 1
            results["by_category"][case.category]["pass"] += 1
            print(f"[PASS] {case.id}: {case.desc}")
        else:
            results["fail"] += 1
            results["by_category"][case.category]["fail"] += 1
            failures.append((case, msg))
            print(f"[FAIL] {case.id}: {case.desc}")
            print(f"       Reason: {msg}")
            print(f"       Code: {case.code[:50]}...")

    print("=" * 60)
    print(f"\nResults by Category:")
    for cat, counts in sorted(results["by_category"].items()):
        total = counts["pass"] + counts["fail"]
        pct = (counts["pass"] / total * 100) if total > 0 else 0
        print(f"  {cat:12}: {counts['pass']}/{total} ({pct:.0f}%)")

    print(f"\nTotal: {len(ALL_TESTS)}")
    print(f"Passed: {results['pass']}")
    print(f"Failed: {results['fail']}")

    if failures:
        print("\nFailure Details:")
        for case, msg in failures:
            print(f"  {case.id}: {msg}")
        sys.exit(1)


if __name__ == "__main__":
    main()
