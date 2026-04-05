fn check_idempotency_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    let checks: ComplianceCheck<'_> = &[
        (
            &|t| is_cmd(t, "mkdir") && !t.contains("-p") && !t.contains("--parents"),
            "Non-idempotent: mkdir without -p (fails if dir exists)",
        ),
        (
            &|t| {
                is_cmd(t, "rm") && !t.contains("-f") && !t.contains("-rf") && !t.contains("--force")
            },
            "Non-idempotent: rm without -f (fails if file missing)",
        ),
        (
            &|t| {
                (t.starts_with("ln -s ") || t.contains("&& ln -s "))
                    && !t.contains("-sf")
                    && !t.contains("-snf")
            },
            "Non-idempotent: ln -s without -f (fails if link exists)",
        ),
        (
            &|t| is_unguarded_adduser(t),
            "Non-idempotent: useradd/groupadd without existence check",
        ),
        (
            &|t| is_unguarded_git_clone(t),
            "Non-idempotent: git clone without directory check",
        ),
        (
            &|t| is_unguarded_createdb(t),
            "Non-idempotent: createdb without --if-not-exists guard",
        ),
        (
            &|t| is_append_to_config(t),
            "Non-idempotent: >> append may duplicate content on rerun",
        ),
    ];

    for (check_fn, message) in checks {
        if check_fn(trimmed) {
            violations.push(Violation {
                rule: RuleId::Idempotency,
                line: Some(line_num),
                message: message.to_string(),
            });
        }
    }
}

/// Check if trimmed line starts with or chains a given command
fn is_cmd(trimmed: &str, cmd: &str) -> bool {
    trimmed.starts_with(cmd)
        && trimmed
            .as_bytes()
            .get(cmd.len())
            .is_some_and(|&b| b == b' ' || b == b'\t')
        || trimmed.contains(&format!("&& {} ", cmd))
}

/// useradd / groupadd without || true or id -u check
fn is_unguarded_adduser(trimmed: &str) -> bool {
    (trimmed.starts_with("useradd ") || trimmed.starts_with("groupadd "))
        && !trimmed.contains("|| true")
        && !trimmed.contains("|| :")
        && !trimmed.contains("2>/dev/null")
        && !trimmed.contains("if ")
}

/// git clone without directory existence check
fn is_unguarded_git_clone(trimmed: &str) -> bool {
    trimmed.starts_with("git clone ")
        && !trimmed.contains("|| true")
        && !trimmed.contains("if ")
        && !trimmed.contains("[ -d")
        && !trimmed.contains("test -d")
}

/// createdb without IF NOT EXISTS guard
fn is_unguarded_createdb(trimmed: &str) -> bool {
    if trimmed.starts_with("createdb ") {
        return !trimmed.contains("|| true") && !trimmed.contains("2>/dev/null");
    }
    false
}

/// Append redirection to config-like files (profile, rc, env)
fn is_append_to_config(trimmed: &str) -> bool {
    if !trimmed.contains(">>") {
        return false;
    }
    // Only flag appends to common config files
    let config_patterns = [
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".zshrc",
        "/etc/profile",
        "/etc/environment",
        ".env",
        "crontab",
    ];
    config_patterns.iter().any(|p| trimmed.contains(p))
        && !trimmed.contains("grep -q")
        && !trimmed.contains("if ")
}

fn check_security(content: &str) -> RuleResult {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        check_security_line(trimmed, i + 1, &mut violations);
    }

    RuleResult {
        rule: RuleId::Security,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_security_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // SEC001: eval with variable input
    if is_eval_command(trimmed) && (trimmed.contains('$') || trimmed.contains('`')) {
        violations.push(Violation {
            rule: RuleId::Security,
            line: Some(line_num),
            message: "SEC001: eval with variable input (injection risk)".to_string(),
        });
    }
    // SEC002: curl | bash
    if is_pipe_to_shell(trimmed) {
        violations.push(Violation {
            rule: RuleId::Security,
            line: Some(line_num),
            message: "SEC002: piping remote content to shell".to_string(),
        });
    }
    // SEC004: TLS verification disabled
    if is_tls_disabled(trimmed) {
        violations.push(Violation {
            rule: RuleId::Security,
            line: Some(line_num),
            message: "SEC004: TLS verification disabled".to_string(),
        });
    }
    // SEC005: Hardcoded secrets in assignments
    if is_hardcoded_secret(trimmed) {
        violations.push(Violation {
            rule: RuleId::Security,
            line: Some(line_num),
            message: "SEC005: hardcoded secret in variable assignment".to_string(),
        });
    }
    // SEC006: Unsafe temporary file
    if is_unsafe_tmp(trimmed) {
        violations.push(Violation {
            rule: RuleId::Security,
            line: Some(line_num),
            message: "SEC006: unsafe temp file (use mktemp instead of /tmp/ literal)".to_string(),
        });
    }
    // SEC007: sudo with dangerous command and unquoted variable
    if is_sudo_danger(trimmed) {
        violations.push(Violation {
            rule: RuleId::Security,
            line: Some(line_num),
            message: "SEC007: sudo with dangerous command and unquoted variable".to_string(),
        });
    }
}

fn is_pipe_to_shell(trimmed: &str) -> bool {
    (trimmed.contains("curl") || trimmed.contains("wget"))
        && (trimmed.contains("| bash") || trimmed.contains("| sh") || trimmed.contains("|sh"))
}

fn is_tls_disabled(trimmed: &str) -> bool {
    trimmed.contains("--no-check-certificate")
        || trimmed.contains("--insecure")
        || is_curl_k(trimmed)
}

/// Detect `curl -k` as a separate flag (not part of another word)
fn is_curl_k(trimmed: &str) -> bool {
    trimmed.contains("curl") && trimmed.split_whitespace().any(|w| w == "-k")
}

/// Detect hardcoded secrets: KEY="literal_value" (not KEY="$VAR" or KEY="${VAR}")
fn is_hardcoded_secret(trimmed: &str) -> bool {
    const SECRET_PREFIXES: &[&str] = &[
        "API_KEY=",
        "SECRET=",
        "PASSWORD=",
        "TOKEN=",
        "AWS_SECRET",
        "PRIVATE_KEY=",
    ];
    const SECRET_LITERALS: &[&str] = &["sk-", "ghp_", "gho_", "glpat-"];

    // Check for secret-named assignments with literal values
    for prefix in SECRET_PREFIXES {
        if let Some(pos) = trimmed.find(prefix) {
            let after = &trimmed[pos + prefix.len()..];
            // Has a value that's not a variable expansion
            if has_literal_value(after) {
                return true;
            }
        }
    }

    // Check for known secret token prefixes in assignment values
    for literal in SECRET_LITERALS {
        if trimmed.contains('=') && trimmed.contains(literal) {
            return true;
        }
    }

    false
}

/// Check if an assignment value is a literal (not a variable expansion)
fn has_literal_value(after: &str) -> bool {
    let val = after.trim_start_matches(['"', '\'']);
    !val.is_empty() && !val.starts_with('$') && !val.starts_with('}')
}

/// Detect unsafe temp file usage: assignment to /tmp/ path without mktemp
fn is_unsafe_tmp(trimmed: &str) -> bool {
    // Only flag assignments (VAR=/tmp/...) not usage
    trimmed.contains("=\"/tmp/") && !trimmed.contains("mktemp")
}

/// Detect sudo with dangerous commands and unquoted variables
fn is_sudo_danger(trimmed: &str) -> bool {
    if !trimmed.contains("sudo ") {
        return false;
    }
    let has_dangerous = trimmed.contains("rm -rf")
        || trimmed.contains("chmod 777")
        || trimmed.contains("chmod -R")
        || trimmed.contains("chown -R");
    // Unquoted variable: space followed by $ then alpha (not in quotes)
    let has_unquoted_var =
        trimmed.contains(" $") && !trimmed.contains(" \"$") && !trimmed.contains(" '");
    has_dangerous && has_unquoted_var
}

fn check_quoting(content: &str) -> RuleResult {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        find_unquoted_vars(trimmed, i + 1, &mut violations);
    }

    RuleResult {
        rule: RuleId::Quoting,
        passed: violations.is_empty(),
        violations,
    }
}

/// Detect unquoted $VAR expansions outside of quotes
fn find_unquoted_vars(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    let chars: Vec<char> = trimmed.chars().collect();
    let mut state = QuoteState::default();
    let mut j = 0;

    while j < chars.len() {
        j = scan_quoting_char(&chars, j, trimmed, line_num, &mut state, violations);
    }
}

#[derive(Default)]
struct QuoteState {
    in_double_quote: bool,
    in_single_quote: bool,
}

/// Process one character for quote-aware scanning; returns next index
fn scan_quoting_char(
    chars: &[char],
    j: usize,
    trimmed: &str,
    line_num: usize,
    state: &mut QuoteState,
    violations: &mut Vec<Violation>,
) -> usize {
    match chars[j] {
        // Backslash escapes: skip the next character entirely
        '\\' if !state.in_single_quote => j + 2,
        '\'' if !state.in_double_quote => {
            state.in_single_quote = !state.in_single_quote;
            j + 1
        }
        '"' if !state.in_single_quote => {
            state.in_double_quote = !state.in_double_quote;
            j + 1
        }
        // $() subshell: skip to matching closing paren (nested quote context)
        '$' if !state.in_single_quote && j + 1 < chars.len() && chars[j + 1] == '(' => {
            skip_subshell(chars, j + 1)
        }
        '$' if !state.in_single_quote && !state.in_double_quote => {
            if is_unquoted_var_expansion(chars, j, trimmed) {
                violations.push(Violation {
                    rule: RuleId::Quoting,
                    line: Some(line_num),
                    message: format!("Unquoted variable expansion at column {}", j + 1),
                });
                skip_var_name(chars, j + 1)
            } else {
                j + 1
            }
        }
        _ => j + 1,
    }
}

/// Skip past a $() or $(()) subshell, handling nested parens
fn skip_subshell(chars: &[char], start: usize) -> usize {
    let mut depth = 0;
    let mut j = start;
    while j < chars.len() {
        match chars[j] {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            '\\' => {
                j += 1; // skip escaped char
            }
            _ => {}
        }
        j += 1;
    }
    j // unterminated subshell, consume rest
}

fn skip_var_name(chars: &[char], start: usize) -> usize {
    let mut j = start;
    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
        j += 1;
    }
    j
}

/// Check if $ at position j is an unquoted variable expansion ($VAR, not $( or ${ or $$)

include!("rules_is_unquoted.rs");
