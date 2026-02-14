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
            ArtifactKind::Dockerfile => vec![
                RuleId::Security,
                RuleId::DockerfileBest,
            ],
            ArtifactKind::ShellConfig => vec![
                RuleId::Security,
                RuleId::Quoting,
                RuleId::ConfigHygiene,
            ],
            ArtifactKind::Workflow => vec![
                RuleId::Security,
            ],
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
    if (trimmed.starts_with("mkdir ") || trimmed.contains("&& mkdir "))
        && !trimmed.contains("-p")
        && !trimmed.contains("--parents")
    {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: mkdir without -p".to_string(),
        });
    }
    // rm without -f (but not rm -r or rm -rf which are fine)
    if (trimmed.starts_with("rm ") || trimmed.contains("&& rm "))
        && !trimmed.contains("-f")
        && !trimmed.contains("-rf")
        && !trimmed.contains("--force")
    {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: rm without -f".to_string(),
        });
    }
    // ln without -sf
    if (trimmed.starts_with("ln -s ") || trimmed.contains("&& ln -s "))
        && !trimmed.contains("-sf")
        && !trimmed.contains("-snf")
    {
        violations.push(Violation {
            rule: RuleId::Idempotency,
            line: Some(line_num),
            message: "Non-idempotent: ln -s without -f".to_string(),
        });
    }
}

fn check_security(content: &str) -> RuleResult {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        // SEC001: eval with variable input
        // Only flag when eval is the command itself, not a subcommand of another
        // tool (e.g. `yq eval`, `jq eval`, `helm eval`).
        if is_eval_command(trimmed)
            && (trimmed.contains('$') || trimmed.contains('`'))
        {
            violations.push(Violation {
                rule: RuleId::Security,
                line: Some(i + 1),
                message: "SEC001: eval with variable input (injection risk)".to_string(),
            });
        }
        // SEC002: curl | bash
        if (trimmed.contains("curl") || trimmed.contains("wget"))
            && (trimmed.contains("| bash") || trimmed.contains("| sh") || trimmed.contains("|sh"))
        {
            violations.push(Violation {
                rule: RuleId::Security,
                line: Some(i + 1),
                message: "SEC002: piping remote content to shell".to_string(),
            });
        }
    }

    RuleResult {
        rule: RuleId::Security,
        passed: violations.is_empty(),
        violations,
    }
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

        // Bash-specific shebang
        if i == 0 && trimmed.starts_with("#!/bin/bash") {
            violations.push(Violation {
                rule: RuleId::Posix,
                line: Some(1),
                message: "Non-POSIX shebang: use #!/bin/sh".to_string(),
            });
        }

        // Bash arrays
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
    }

    RuleResult {
        rule: RuleId::Posix,
        passed: violations.is_empty(),
        violations,
    }
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

        // SC2006: Use $(...) instead of backticks
        if trimmed.contains('`') && !trimmed.contains("\\`") {
            violations.push(Violation {
                rule: RuleId::ShellCheck,
                line: Some(i + 1),
                message: "SC2006: Use $(...) instead of backticks".to_string(),
            });
        }

        // SC2115: Use "${var:?}" to fail if empty
        if (trimmed.starts_with("rm -rf /") || trimmed.starts_with("rm -rf \"/$"))
            && trimmed.contains('$')
            && !trimmed.contains(":?")
        {
            violations.push(Violation {
                rule: RuleId::ShellCheck,
                line: Some(i + 1),
                message: "SC2115: Dangerous rm -rf with variable path".to_string(),
            });
        }
    }

    RuleResult {
        rule: RuleId::ShellCheck,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_makefile_safety(content: &str) -> RuleResult {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Shell injection in recipes (tab-indented lines are recipes)
        if line.starts_with('\t') && trimmed.contains("eval ") {
            violations.push(Violation {
                rule: RuleId::MakefileSafety,
                line: Some(i + 1),
                message: "eval in Makefile recipe (injection risk)".to_string(),
            });
        }
    }

    RuleResult {
        rule: RuleId::MakefileSafety,
        passed: violations.is_empty(),
        violations,
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

        // DOCKER010: Missing USER directive check (done at end)

        // DOCKER008: ADD instead of COPY for local files
        if trimmed.starts_with("ADD ") && !trimmed.contains("http://") && !trimmed.contains("https://") {
            violations.push(Violation {
                rule: RuleId::DockerfileBest,
                line: Some(i + 1),
                message: "DOCKER008: Use COPY instead of ADD for local files".to_string(),
            });
        }
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
