//! Compliance rules (COMPLY-001 through COMPLY-010)
//!
//! Each rule is a falsifiable hypothesis per Popper (1959).
//! The check attempts to falsify compliance; survival = provisional acceptance.

use super::discovery::{Artifact, ArtifactKind};

/// Compliance rule identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleId {
    /// COMPLY-001: POSIX compliance (shellcheck -s sh)
    Posix,
    /// COMPLY-002: Determinism (no $RANDOM, $$, date +%s)
    Determinism,
    /// COMPLY-003: Idempotency (mkdir -p, rm -f, ln -sf)
    Idempotency,
    /// COMPLY-004: Security (SEC001-SEC008)
    Security,
    /// COMPLY-005: Variable quoting
    Quoting,
    /// COMPLY-006: ShellCheck clean
    ShellCheck,
    /// COMPLY-007: Makefile safety
    MakefileSafety,
    /// COMPLY-008: Dockerfile best practices
    DockerfileBest,
    /// COMPLY-009: Config hygiene
    ConfigHygiene,
    /// COMPLY-010: pzsh startup budget
    PzshBudget,
}

impl RuleId {
    pub fn code(&self) -> &'static str {
        match self {
            RuleId::Posix => "COMPLY-001",
            RuleId::Determinism => "COMPLY-002",
            RuleId::Idempotency => "COMPLY-003",
            RuleId::Security => "COMPLY-004",
            RuleId::Quoting => "COMPLY-005",
            RuleId::ShellCheck => "COMPLY-006",
            RuleId::MakefileSafety => "COMPLY-007",
            RuleId::DockerfileBest => "COMPLY-008",
            RuleId::ConfigHygiene => "COMPLY-009",
            RuleId::PzshBudget => "COMPLY-010",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            RuleId::Posix => "POSIX Compliance",
            RuleId::Determinism => "Determinism",
            RuleId::Idempotency => "Idempotency",
            RuleId::Security => "Security",
            RuleId::Quoting => "Variable Quoting",
            RuleId::ShellCheck => "ShellCheck Clean",
            RuleId::MakefileSafety => "Makefile Safety",
            RuleId::DockerfileBest => "Dockerfile Best Practices",
            RuleId::ConfigHygiene => "Config Hygiene",
            RuleId::PzshBudget => "pzsh Startup Budget",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RuleId::Posix => "Detects bash-specific constructs: [[ ]], (( )), <<<, select, ${var/}, pipefail, &>",
            RuleId::Determinism => "Flags non-deterministic: $RANDOM, $SRANDOM, $BASHPID, $$, date, mktemp, shuf, /dev/urandom",
            RuleId::Idempotency => "Requires safe-to-rerun: mkdir -p, rm -f, ln -sf, useradd guards, git clone checks",
            RuleId::Security => "Checks SEC001-SEC008: eval injection, curl|bash, rm -rf, secrets, temp files",
            RuleId::Quoting => "Detects unquoted variable expansions that risk word splitting or globbing",
            RuleId::ShellCheck => "Lightweight ShellCheck: SC2006 backticks, SC2115 rm, SC2164 cd, SC2012 ls",
            RuleId::MakefileSafety => "Makefile risks: eval in recipes, bare make, rm -rf with vars, missing .PHONY",
            RuleId::DockerfileBest => "Dockerfile hygiene: unpinned base, ADD vs COPY, apt cleanup, USER directive",
            RuleId::ConfigHygiene => "Config file quality: PATH manipulation, alias complexity, source safety",
            RuleId::PzshBudget => "pzsh shell startup time budget enforcement",
        }
    }

    /// Artifact types this rule applies to
    pub fn applies_to(&self) -> &'static [&'static str] {
        match self {
            RuleId::Posix => &["shell"],
            RuleId::Determinism => &["shell", "makefile"],
            RuleId::Idempotency => &["shell", "makefile"],
            RuleId::Security => &["shell", "makefile", "dockerfile", "config", "workflow"],
            RuleId::Quoting => &["shell", "config"],
            RuleId::ShellCheck => &["shell"],
            RuleId::MakefileSafety => &["makefile"],
            RuleId::DockerfileBest => &["dockerfile"],
            RuleId::ConfigHygiene => &["config"],
            RuleId::PzshBudget => &["config"],
        }
    }

    /// All rule IDs in order
    pub fn all() -> &'static [RuleId] {
        &[
            RuleId::Posix,
            RuleId::Determinism,
            RuleId::Idempotency,
            RuleId::Security,
            RuleId::Quoting,
            RuleId::ShellCheck,
            RuleId::MakefileSafety,
            RuleId::DockerfileBest,
            RuleId::ConfigHygiene,
            RuleId::PzshBudget,
        ]
    }

    pub fn weight(&self) -> u32 {
        match self {
            RuleId::Posix => 20,
            RuleId::Determinism => 15,
            RuleId::Idempotency => 15,
            RuleId::Security => 20,
            RuleId::Quoting => 10,
            RuleId::ShellCheck => 10,
            RuleId::MakefileSafety => 5,
            RuleId::DockerfileBest => 5,
            RuleId::ConfigHygiene => 5,
            RuleId::PzshBudget => 5,
        }
    }

    /// Rules applicable to a given artifact kind
    pub fn applicable_rules(kind: ArtifactKind) -> Vec<RuleId> {
        match kind {
            ArtifactKind::ShellScript => vec![
                RuleId::Posix,
                RuleId::Determinism,
                RuleId::Idempotency,
                RuleId::Security,
                RuleId::Quoting,
                RuleId::ShellCheck,
            ],
            ArtifactKind::Makefile => vec![
                RuleId::Determinism,
                RuleId::Idempotency,
                RuleId::Security,
                RuleId::MakefileSafety,
            ],
            ArtifactKind::Dockerfile => vec![RuleId::Security, RuleId::DockerfileBest],
            ArtifactKind::ShellConfig => {
                vec![RuleId::Security, RuleId::Quoting, RuleId::ConfigHygiene]
            }
            ArtifactKind::Workflow => vec![RuleId::Security],
            ArtifactKind::DevContainer => vec![],
        }
    }
}

/// Result of a single rule check against an artifact
#[derive(Clone, Debug)]
pub struct RuleResult {
    pub rule: RuleId,
    pub passed: bool,
    pub violations: Vec<Violation>,
}

/// A specific compliance violation
#[derive(Clone, Debug)]
pub struct Violation {
    pub rule: RuleId,
    pub line: Option<usize>,
    pub message: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "{}: line {}: {}", self.rule.code(), line, self.message)
        } else {
            write!(f, "{}: {}", self.rule.code(), self.message)
        }
    }
}

/// Check a single rule against an artifact's content
pub fn check_rule(rule: RuleId, content: &str, artifact: &Artifact) -> RuleResult {
    match rule {
        RuleId::Determinism => check_determinism(content, artifact),
        RuleId::Idempotency => check_idempotency(content, artifact),
        RuleId::Security => check_security(content),
        RuleId::Quoting => check_quoting(content),
        RuleId::Posix => check_posix_patterns(content),
        RuleId::ShellCheck => check_shellcheck_patterns(content),
        RuleId::MakefileSafety => check_makefile_safety(content),
        RuleId::DockerfileBest => check_dockerfile_best(content),
        RuleId::ConfigHygiene => check_config_hygiene(content),
        RuleId::PzshBudget => RuleResult {
            rule,
            passed: true,
            violations: vec![],
        }, // Handled externally
    }
}

fn check_determinism(content: &str, artifact: &Artifact) -> RuleResult {
    let is_makefile = artifact.kind == ArtifactKind::Makefile;
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        check_determinism_line(trimmed, i + 1, is_makefile, &mut violations);
    }

    RuleResult {
        rule: RuleId::Determinism,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_determinism_line(
    trimmed: &str,
    line_num: usize,
    is_makefile: bool,
    violations: &mut Vec<Violation>,
) {
    if trimmed.contains("$RANDOM") {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: $RANDOM".to_string(),
        });
    }
    // $SRANDOM (bash 5.1+) and $BASHPID are non-deterministic
    if trimmed.contains("$SRANDOM") {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: $SRANDOM".to_string(),
        });
    }
    if trimmed.contains("$BASHPID") {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: $BASHPID".to_string(),
        });
    }
    // In Makefiles, $$ is always Make's escape for a literal $, never bash's $$ PID.
    if !is_makefile && is_pid_usage(trimmed) {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: $$ (process ID)".to_string(),
        });
    }
    if trimmed.contains("date +%s") || trimmed.contains("date +%N") {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: timestamp command".to_string(),
        });
    }
    // /dev/urandom and /dev/random — entropy sources
    if trimmed.contains("/dev/urandom") || trimmed.contains("/dev/random") {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: /dev/urandom or /dev/random".to_string(),
        });
    }
    // mktemp generates random filenames
    if is_mktemp_call(trimmed) {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: mktemp generates random names".to_string(),
        });
    }
    // shuf — random shuffle
    if is_shuf_call(trimmed) {
        violations.push(Violation {
            rule: RuleId::Determinism,
            line: Some(line_num),
            message: "Non-deterministic: shuf produces random output".to_string(),
        });
    }
}

/// Detect mktemp as a command (not in comments or strings)
fn is_mktemp_call(trimmed: &str) -> bool {
    trimmed.starts_with("mktemp")
        || trimmed.contains("$(mktemp")
        || trimmed.contains("`mktemp")
        || trimmed.contains("| mktemp")
}

/// Detect shuf as a command
fn is_shuf_call(trimmed: &str) -> bool {
    trimmed.starts_with("shuf ")
        || trimmed.starts_with("shuf\t")
        || trimmed.contains("| shuf")
        || trimmed.contains("$(shuf")
}

/// Check if `eval` appears as a shell command (first token), not as a subcommand
/// of another tool (e.g. `yq eval`, `jq eval`, `helm eval`).
fn is_eval_command(trimmed: &str) -> bool {
    // eval as first word on the line
    if trimmed.starts_with("eval ") {
        return true;
    }
    // eval after command separators: ; && || |
    for sep in &["; ", "&& ", "|| "] {
        if let Some(pos) = trimmed.find(sep) {
            let after = trimmed[pos + sep.len()..].trim_start();
            if after.starts_with("eval ") {
                return true;
            }
        }
    }
    false
}

/// Detect $$ PID usage (not Makefile $$ escaping like $$@, $$<, $$^)
fn is_pid_usage(trimmed: &str) -> bool {
    trimmed.contains("$$")
        && !trimmed.contains("\"$$")
        && !trimmed.contains("$$@")
        && !trimmed.contains("$$<")
        && !trimmed.contains("$$(")
        && !trimmed.contains("$$^")
}

fn check_idempotency(content: &str, artifact: &Artifact) -> RuleResult {
    let mut violations = Vec::new();

    // Only check shell scripts and configs
    if !matches!(
        artifact.kind,
        ArtifactKind::ShellScript | ArtifactKind::ShellConfig
    ) {
        return RuleResult {
            rule: RuleId::Idempotency,
            passed: true,
            violations,
        };
    }

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        check_idempotency_line(trimmed, i + 1, &mut violations);
    }

    RuleResult {
        rule: RuleId::Idempotency,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_idempotency_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // mkdir without -p
    if is_cmd(trimmed, "mkdir") && !trimmed.contains("-p") && !trimmed.contains("--parents") {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: mkdir without -p (fails if dir exists)".to_string(),
        });
    }
    // rm without -f (but not rm -r or rm -rf which are fine)
    if is_cmd(trimmed, "rm")
        && !trimmed.contains("-f")
        && !trimmed.contains("-rf")
        && !trimmed.contains("--force")
    {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: rm without -f (fails if file missing)".to_string(),
        });
    }
    // ln -s without -f
    if (trimmed.starts_with("ln -s ") || trimmed.contains("&& ln -s "))
        && !trimmed.contains("-sf")
        && !trimmed.contains("-snf")
    {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: ln -s without -f (fails if link exists)".to_string(),
        });
    }
    // useradd / groupadd without guard — fails if user/group already exists
    if is_unguarded_adduser(trimmed) {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: useradd/groupadd without existence check".to_string(),
        });
    }
    // git clone without checking if directory exists
    if is_unguarded_git_clone(trimmed) {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: git clone without directory check".to_string(),
        });
    }
    // createdb / CREATE DATABASE — fails if database exists
    if is_unguarded_createdb(trimmed) {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: createdb without --if-not-exists guard".to_string(),
        });
    }
    // Append redirection (>>) to config files — duplicates content on rerun
    if is_append_to_config(trimmed) {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: >> append may duplicate content on rerun".to_string(),
        });
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
    // Bash-specific shebang
    if i == 0 && trimmed.starts_with("#!/bin/bash") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(1),
            message: "Non-POSIX shebang: use #!/bin/sh".to_string(),
        });
    }

    // Bash arrays: declare -a/-A
    if trimmed.contains("declare -a") || trimmed.contains("declare -A") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: declare -a/-A (arrays)".to_string(),
        });
    }

    // [[ ]] double brackets
    if trimmed.contains("[[") && trimmed.contains("]]") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: [[ ]] (use [ ] for POSIX)".to_string(),
        });
    }

    // function keyword: `function name` (POSIX only supports `name()`)
    if is_function_keyword(trimmed) {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: function keyword (use name() for POSIX)".to_string(),
        });
    }

    // Standalone (( )) arithmetic (not $(( )) which is POSIX)
    if is_standalone_arithmetic(trimmed) {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: (( )) arithmetic (use $(( )) or [ ] for POSIX)".to_string(),
        });
    }

    // <<< here-strings
    if trimmed.contains("<<<") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: <<< here-string (use echo | or heredoc for POSIX)".to_string(),
        });
    }

    // select statement
    if is_select_statement(trimmed) {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: select statement".to_string(),
        });
    }

    // ${var//pattern/repl} and ${var/pattern/repl} pattern substitution
    if has_bash_parameter_expansion(trimmed) {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: ${var/...} pattern substitution".to_string(),
        });
    }

    // ${var,,} and ${var^^} case modification
    if has_bash_case_modification(trimmed) {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: ${var,,} or ${var^^} case modification".to_string(),
        });
    }

    // set -o pipefail (bash-specific)
    if trimmed.contains("set -o pipefail") || trimmed.contains("set -euo pipefail") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: set -o pipefail (not in POSIX)".to_string(),
        });
    }

    // &> combined redirect (bash-specific, POSIX uses >file 2>&1)
    if is_bash_redirect(trimmed) {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(i + 1),
            message: "Bash-specific: &> redirect (use >file 2>&1 for POSIX)".to_string(),
        });
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

fn check_makefile_safety(content: &str) -> RuleResult {
    let mut violations = Vec::new();
    let mut declared_phony: Vec<String> = Vec::new();
    let mut defined_targets: Vec<(String, usize)> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Collect .PHONY declarations
        if trimmed.starts_with(".PHONY") {
            collect_phony_targets(trimmed, &mut declared_phony);
        }

        // Collect target definitions (name: ...)
        if is_target_definition(line) {
            if let Some(name) = extract_target_name(line) {
                defined_targets.push((name, i + 1));
            }
        }

        // Recipe-level checks (tab-indented lines)
        if line.starts_with('\t') {
            check_makefile_recipe(trimmed, i + 1, &mut violations);
        }
    }

    // Check for common targets missing .PHONY
    check_missing_phony(&declared_phony, &defined_targets, &mut violations);

    RuleResult {
        rule: RuleId::MakefileSafety,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_makefile_recipe(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // MAKE001: eval in recipe (injection risk)
    if trimmed.contains("eval ") {
        violations.push(Violation {
            rule: RuleId::MakefileSafety,
            line: Some(line_num),
            message: "MAKE001: eval in Makefile recipe (injection risk)".to_string(),
        });
    }

    // MAKE002: bare 'make' instead of $(MAKE) in recipes
    if is_recursive_make_bare(trimmed) {
        violations.push(Violation {
            rule: RuleId::MakefileSafety,
            line: Some(line_num),
            message: "MAKE002: Use $(MAKE) instead of bare make for recursive calls".to_string(),
        });
    }

    // MAKE003: rm -rf with variable in recipe (dangerous)
    if is_dangerous_recipe_rm(trimmed) {
        violations.push(Violation {
            rule: RuleId::MakefileSafety,
            line: Some(line_num),
            message: "MAKE003: Dangerous rm -rf with variable in recipe".to_string(),
        });
    }
}

/// Detect bare `make target` instead of `$(MAKE) target` in recipes
fn is_recursive_make_bare(trimmed: &str) -> bool {
    // Match: starts with make, or has `&& make`, `; make`, `|| make`
    let starts_bare = trimmed.starts_with("make ") || trimmed == "make";
    let has_chained =
        trimmed.contains("&& make ") || trimmed.contains("; make ") || trimmed.contains("|| make ");
    // Exclude $(MAKE) and ${MAKE} and @make (suppressed)
    let is_variable = trimmed.contains("$(MAKE)") || trimmed.contains("${MAKE}");
    (starts_bare || has_chained) && !is_variable
}

/// Detect rm -rf with unguarded variable expansion in recipes
fn is_dangerous_recipe_rm(trimmed: &str) -> bool {
    if !trimmed.contains("rm -rf") && !trimmed.contains("rm -fr") {
        return false;
    }
    // Check for variable in the rm path ($$var or $(VAR))
    // Only flag if the variable isn't guarded with :? or similar
    let has_var = trimmed.contains("$$") || trimmed.contains("$(");
    let has_guard = trimmed.contains(":?") || trimmed.contains("|| true");
    has_var && !has_guard
}

fn collect_phony_targets(line: &str, phonys: &mut Vec<String>) {
    // .PHONY: target1 target2 target3
    if let Some(targets) = line.strip_prefix(".PHONY:") {
        for target in targets.split_whitespace() {
            phonys.push(target.to_string());
        }
    } else if let Some(targets) = line.strip_prefix(".PHONY :") {
        for target in targets.split_whitespace() {
            phonys.push(target.to_string());
        }
    }
}

/// Check if a line is a target definition (not a variable assignment)
fn is_target_definition(line: &str) -> bool {
    // Target definitions start at column 0, contain `:`, are not variable assignments (=)
    if line.starts_with('\t') || line.starts_with(' ') || line.starts_with('#') {
        return false;
    }
    line.contains(':') && !line.contains('=') && !line.starts_with('.')
}

fn extract_target_name(line: &str) -> Option<String> {
    line.split(':').next().map(|s| s.trim().to_string())
}

/// Common targets that should always have .PHONY
const COMMON_PHONY_TARGETS: &[&str] = &[
    "all", "clean", "test", "build", "install", "check", "lint", "format", "help", "dist",
    "release", "deploy", "coverage", "bench",
];

fn check_missing_phony(
    phonys: &[String],
    targets: &[(String, usize)],
    violations: &mut Vec<Violation>,
) {
    for (name, line) in targets {
        if COMMON_PHONY_TARGETS.contains(&name.as_str()) && !phonys.contains(name) {
            violations.push(Violation {
                rule: RuleId::MakefileSafety,
                line: Some(*line),
                message: format!("MAKE004: Target '{}' should be declared .PHONY", name),
            });
        }
    }
}

fn check_dockerfile_best(content: &str) -> RuleResult {
    let mut violations = Vec::new();
    let mut has_user = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("USER ") {
            has_user = true;
        }

        check_dockerfile_line(trimmed, i + 1, &mut violations);
    }

    if !has_user && !content.is_empty() {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: None,
            message: "DOCKER010: Missing USER directive (runs as root)".to_string(),
        });
    }

    RuleResult {
        rule: RuleId::DockerfileBest,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_dockerfile_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // DOCKER008: ADD instead of COPY for local files
    if trimmed.starts_with("ADD ") && !trimmed.contains("http://") && !trimmed.contains("https://")
    {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER008: Use COPY instead of ADD for local files".to_string(),
        });
    }

    // DOCKER001: Untagged or :latest base image (non-deterministic)
    if is_unpinned_from(trimmed) {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER001: Pin base image version (avoid untagged or :latest)".to_string(),
        });
    }

    // DOCKER003: apt-get install without cleanup
    if is_apt_without_clean(trimmed) {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER003: apt-get install without cleanup (add rm -rf /var/lib/apt/lists/*)"
                .to_string(),
        });
    }

    // DOCKER004: EXPOSE with no port number
    if trimmed == "EXPOSE" {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER004: EXPOSE requires a port number".to_string(),
        });
    }
}

/// Detect FROM without a pinned tag (FROM image or FROM image:latest)
fn is_unpinned_from(trimmed: &str) -> bool {
    if !trimmed.starts_with("FROM ") {
        return false;
    }
    let image = trimmed[5..].split_whitespace().next().unwrap_or("");
    // Skip scratch (no tag needed) and $ARG references
    if image == "scratch" || image.starts_with('$') {
        return false;
    }
    // Check for tag: image:tag (but not image:latest)
    if let Some(tag) = image.split(':').nth(1) {
        tag == "latest"
    } else {
        // No tag at all — unpinned
        !image.contains('@') // @sha256: is pinned by digest
    }
}

/// Detect apt-get install without cleanup in the same RUN layer
fn is_apt_without_clean(trimmed: &str) -> bool {
    if !trimmed.contains("apt-get install") {
        return false;
    }
    // Check if cleanup is in the same RUN command
    !trimmed.contains("rm -rf /var/lib/apt")
        && !trimmed.contains("apt-get clean")
        && !trimmed.contains("apt-get autoremove")
}

fn check_config_hygiene(content: &str) -> RuleResult {
    let mut violations = Vec::new();
    let mut path_exports = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        // Detect PATH modifications
        if trimmed.contains("export PATH=") || trimmed.contains("PATH=") {
            path_exports.push(i + 1);
        }
    }

    // Multiple PATH exports suggest potential duplication
    if path_exports.len() > 3 {
        violations.push(Violation {
            rule: RuleId::ConfigHygiene,
            line: Some(path_exports[0]),
            message: format!(
                "PATH modified {} times (potential duplication)",
                path_exports.len()
            ),
        });
    }

    RuleResult {
        rule: RuleId::ConfigHygiene,
        passed: violations.is_empty(),
        violations,
    }
}
