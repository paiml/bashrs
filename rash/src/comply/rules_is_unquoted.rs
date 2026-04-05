fn is_unquoted_var_expansion(chars: &[char], j: usize, trimmed: &str) -> bool {
    j + 1 < chars.len()
        && chars[j + 1].is_alphabetic()
        && chars[j + 1] != '('
        && chars[j + 1] != '{'
        && !trimmed[..j].ends_with("((")
}

fn check_posix_patterns(content: &str) -> RuleResult {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            continue;
        }
        check_posix_line(trimmed, i, &mut violations);
    }

    RuleResult {
        rule: RuleId::Posix,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_posix_line(trimmed: &str, i: usize, violations: &mut Vec<Violation>) {
    // Bash-specific shebang (only on first line)
    if i == 0 && trimmed.starts_with("#!/bin/bash") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(1),
            message: "Non-POSIX shebang: use #!/bin/sh".to_string(),
        });
    }

    // Table-driven POSIX violation checks (all use line i+1)
    let posix_checks: ComplianceCheck<'_> = &[
        (
            &|t| t.contains("declare -a") || t.contains("declare -A"),
            "Bash-specific: declare -a/-A (arrays)",
        ),
        (
            &|t| t.contains("[[") && t.contains("]]"),
            "Bash-specific: [[ ]] (use [ ] for POSIX)",
        ),
        (
            &|t| is_function_keyword(t),
            "Bash-specific: function keyword (use name() for POSIX)",
        ),
        (
            &|t| is_standalone_arithmetic(t),
            "Bash-specific: (( )) arithmetic (use $(( )) or [ ] for POSIX)",
        ),
        (
            &|t| t.contains("<<<"),
            "Bash-specific: <<< here-string (use echo | or heredoc for POSIX)",
        ),
        (
            &|t| is_select_statement(t),
            "Bash-specific: select statement",
        ),
        (
            &|t| has_bash_parameter_expansion(t),
            "Bash-specific: ${var/...} pattern substitution",
        ),
        (
            &|t| has_bash_case_modification(t),
            "Bash-specific: ${var,,} or ${var^^} case modification",
        ),
        (
            &|t| t.contains("set -o pipefail") || t.contains("set -euo pipefail"),
            "Bash-specific: set -o pipefail (not in POSIX)",
        ),
        (
            &|t| is_bash_redirect(t),
            "Bash-specific: &> redirect (use >file 2>&1 for POSIX)",
        ),
    ];

    for (check_fn, message) in posix_checks {
        if check_fn(trimmed) {
            violations.push(Violation {
                rule: RuleId::Posix,
                line: Some(i + 1),
                message: message.to_string(),
            });
        }
    }
}

/// Detect `function name` syntax (bash-specific, POSIX uses `name()`)
fn is_function_keyword(trimmed: &str) -> bool {
    if let Some(rest) = trimmed.strip_prefix("function ") {
        // Must be followed by a function name (word chars)
        let rest = rest.trim_start();
        return rest
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_');
    }
    false
}

/// Detect standalone (( )) arithmetic (not $(( )) which is POSIX)
fn is_standalone_arithmetic(trimmed: &str) -> bool {
    // Standalone (( starts at beginning or after ; && ||
    if trimmed.starts_with("((") {
        return true;
    }
    for sep in &["; ", "&& ", "|| "] {
        if let Some(pos) = trimmed.find(sep) {
            let after = trimmed[pos + sep.len()..].trim_start();
            if after.starts_with("((") {
                return true;
            }
        }
    }
    false
}

/// Detect `select` statement (bash-specific)
fn is_select_statement(trimmed: &str) -> bool {
    trimmed.starts_with("select ") && trimmed.contains(" in ")
}

/// Detect ${var/pattern/repl} or ${var//pattern/repl} pattern substitution
/// Must not flag POSIX expansions like ${var:-/path}, ${var#*/}, ${var%/*}
fn has_bash_parameter_expansion(trimmed: &str) -> bool {
    for (start, _) in brace_expansion_starts(trimmed) {
        if is_pattern_substitution(trimmed.as_bytes(), start) {
            return true;
        }
    }
    false
}

/// Check if a ${...} starting at `start` (after `${`) is a pattern substitution
fn is_pattern_substitution(bytes: &[u8], start: usize) -> bool {
    let mut j = start;
    // Skip past variable name (alphanumeric + underscore)
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
        j += 1;
    }
    // Pattern substitution: first non-varname char is /
    j < bytes.len() && bytes[j] == b'/'
}

/// Detect ${var,,} ${var^^} case modification (bash 4.0+)
fn has_bash_case_modification(trimmed: &str) -> bool {
    for (start, _) in brace_expansion_starts(trimmed) {
        if is_case_modification(trimmed.as_bytes(), start) {
            return true;
        }
    }
    false
}

/// Check if a ${...} starting at `start` (after `${`) contains ,, or ^^
fn is_case_modification(bytes: &[u8], start: usize) -> bool {
    let mut j = start;
    while j + 1 < bytes.len() && bytes[j] != b'}' {
        if is_case_mod_operator(bytes[j], bytes[j + 1]) {
            return true;
        }
        j += 1;
    }
    false
}

fn is_case_mod_operator(a: u8, b: u8) -> bool {
    (a == b',' && b == b',') || (a == b'^' && b == b'^')
}

/// Yield (content_start, brace_pos) for each `${` in the string
fn brace_expansion_starts(s: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let bytes = s.as_bytes();
    (0..bytes.len().saturating_sub(1)).filter_map(move |i| {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            Some((i + 2, i))
        } else {
            None
        }
    })
}

/// Detect &> redirect (bash-specific combined stdout+stderr redirect)
fn is_bash_redirect(trimmed: &str) -> bool {
    // &> or &>> (but not >&2 which is POSIX fd redirect)
    if let Some(pos) = trimmed.find("&>") {
        // Make sure it's not >&N (fd redirect) — check preceding char
        if pos > 0 && trimmed.as_bytes()[pos - 1] == b'>' {
            // This is >&... which could be POSIX >&2
            return false;
        }
        return true;
    }
    false
}

fn check_shellcheck_patterns(content: &str) -> RuleResult {
    // Lightweight shellcheck-equivalent patterns
    // Full shellcheck integration deferred to Phase 2
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        check_shellcheck_line(trimmed, i + 1, &mut violations);
    }

    RuleResult {
        rule: RuleId::ShellCheck,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_shellcheck_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // SC2006: Use $(...) instead of backticks
    if trimmed.contains('`') && !trimmed.contains("\\`") {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2006: Use $(...) instead of backticks".to_string(),
        });
    }

    // SC2115: Use "${var:?}" to fail if empty
    if is_dangerous_rm_rf(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2115: Dangerous rm -rf with variable path".to_string(),
        });
    }

    // SC2164: cd without || exit (silent failure)
    if is_bare_cd(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2164: Use cd ... || exit in case cd fails".to_string(),
        });
    }

    // SC2162: read without -r (mangles backslashes)
    if is_read_without_r(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2162: read without -r mangles backslashes".to_string(),
        });
    }

    // SC2181: if [ $? ... ] instead of direct command check
    if is_dollar_question_check(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2181: Check exit code directly with if cmd, not $?".to_string(),
        });
    }

    // SC2012: for f in $(ls ...) — use glob instead
    if is_ls_iteration(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2012: Use glob instead of ls to iterate files".to_string(),
        });
    }

    // SC2035: Use ./* glob to avoid filenames starting with -
    if is_bare_glob(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2035: Use ./* instead of * to avoid - filename issues".to_string(),
        });
    }
}

fn is_dangerous_rm_rf(trimmed: &str) -> bool {
    (trimmed.starts_with("rm -rf /") || trimmed.starts_with("rm -rf \"/$"))
        && trimmed.contains('$')
        && !trimmed.contains(":?")
}

/// SC2164: cd without error handling (|| exit, || return, || die)
fn is_bare_cd(trimmed: &str) -> bool {
    if !trimmed.starts_with("cd ") {
        return false;
    }
    // Allow: cd ... || exit, cd ... || return, cd ... && ..., cd alone (cd ~)
    let has_error_handling = trimmed.contains("|| exit")
        || trimmed.contains("|| return")
        || trimmed.contains("|| die")
        || trimmed.contains("|| {");
    // cd to home (just "cd" or "cd ~") is always safe
    let is_safe_cd = trimmed == "cd" || trimmed == "cd ~";
    !has_error_handling && !is_safe_cd
}

/// SC2162: read without -r flag
fn is_read_without_r(trimmed: &str) -> bool {
    if !trimmed.starts_with("read ") && !trimmed.contains("| read ") {
        return false;
    }
    // Extract the read command portion
    let read_part = if let Some(pos) = trimmed.find("| read ") {
        &trimmed[pos + 2..]
    } else {
        trimmed
    };
    // Check if -r is present (as a flag, not part of a variable name)
    !read_part
        .split_whitespace()
        .any(|w| w == "-r" || w.starts_with("-r") && w.len() > 2 && w.as_bytes()[2] != b' ')
}

/// SC2181: if [ $? ... ] pattern
fn is_dollar_question_check(trimmed: &str) -> bool {
    trimmed.contains("$?") && (trimmed.contains("[ $?") || trimmed.contains("[$?"))
}

/// SC2012: for f in $(ls ...) iteration
fn is_ls_iteration(trimmed: &str) -> bool {
    trimmed.contains("$(ls") || trimmed.contains("`ls ")
}

/// SC2035: Bare * glob in command arguments (use ./* instead)
fn is_bare_glob(trimmed: &str) -> bool {
    // Only flag: for f in *; (bare * as sole glob, not *.txt or ./*)
    if trimmed.starts_with("for ") {
        if let Some(pos) = trimmed.find(" in ") {
            let after_in = trimmed[pos + 4..].trim_start();
            // Bare * followed by ; or end-of-line
            return after_in.starts_with("*;") || after_in.starts_with("* ;") || after_in == "*";
        }
    }
    false
}

include!("rules_makefile_docker.rs");
