//! Compliance rules (COMPLY-001 through COMPLY-010)
//!
//! Each rule is a falsifiable hypothesis per Popper (1959).
//! The check attempts to falsify compliance; survival = provisional acceptance.

use super::discovery::{Artifact, ArtifactKind};

/// A compliance check: predicate function paired with a violation message.
type ComplianceCheck<'a> = &'a [(&'a dyn Fn(&str) -> bool, &'a str)];

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

include!("rules_check.rs");
