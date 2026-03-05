//! DET004: Non-deterministic system state commands
//!
//! **Rule**: Detect command substitution using commands whose output depends on
//! runtime system state (disk, memory, load, processes, network).
//!
//! **Why this matters**:
//! Commands like `df`, `free`, `uptime`, `ps`, `who`, `netstat` return different
//! values on every invocation. Scripts depending on them are non-deterministic.
//!
//! ## Examples
//!
//! Bad (non-deterministic):
//! ```bash
//! avail=$(df -m / | awk 'NR==2{print $4}')
//! mem=$(free -m | awk '/Mem/{print $2}')
//! load=$(uptime | awk '{print $NF}')
//! ```
//!
//! Good (deterministic):
//! ```bash
//! avail="${REQUIRED_DISK_MB:-500}"  # Pass as parameter
//! mem="${REQUIRED_MEM_MB:-1024}"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use std::sync::LazyLock;

/// System-state commands that produce non-deterministic output.
const STATE_COMMANDS: &[&str] = &[
    "df", "free", "uptime", "vmstat", "iostat", "mpstat", "sar", "ps", "top", "pgrep", "lsof",
    "fuser", "who", "w", "last", "lastlog", "netstat", "ss", "ifconfig", "ip addr", "sensors",
    "lscpu", "nproc",
];

static RE_CMD_SUB: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\$\(([^)]+)\)").expect("valid regex"));

/// Check for system-state-dependent command substitution.
pub fn check(source: &str) -> LintResult {
    let mut diagnostics = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let ln = line_num + 1;

        for cap in RE_CMD_SUB.captures_iter(line) {
            let inner = cap.get(1).map_or("", |m| m.as_str());
            // Check if the command substitution starts with or pipes through a state command
            for cmd in STATE_COMMANDS {
                if command_in_pipeline(inner, cmd) {
                    diagnostics.push(Diagnostic::new(
                        "DET004",
                        Severity::Warning,
                        format!(
                            "Command substitution uses `{cmd}` which depends on system state — non-deterministic"
                        ),
                        Span::new(ln, 1, ln, line.len()),
                    ));
                    break; // One diagnostic per $(...) is enough
                }
            }
        }

        // Also check backtick command substitution
        if line.contains('`') {
            for cmd in STATE_COMMANDS {
                if line.contains(&format!("`{cmd} ")) || line.contains(&format!("`{cmd}`")) {
                    diagnostics.push(Diagnostic::new(
                        "DET004",
                        Severity::Warning,
                        format!(
                            "Command substitution uses `{cmd}` which depends on system state — non-deterministic"
                        ),
                        Span::new(ln, 1, ln, line.len()),
                    ));
                    break;
                }
            }
        }
    }

    LintResult { diagnostics }
}

/// Check if a state command appears at the start or in a pipeline within command substitution.
fn command_in_pipeline(inner: &str, cmd: &str) -> bool {
    // Split by pipe and check each segment
    for segment in inner.split('|') {
        let trimmed = segment.trim();
        // Check if segment starts with the command (possibly with flags)
        if trimmed == cmd
            || trimmed.starts_with(&format!("{cmd} "))
            || trimmed.starts_with(&format!("{cmd}\t"))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_df_in_command_substitution() {
        let diags = check("avail=$(df -m / | awk 'NR==2{print $4}')").diagnostics;
        assert!(!diags.is_empty(), "Should catch df in command substitution");
        assert_eq!(diags[0].code, "DET004");
    }

    #[test]
    fn test_free_command() {
        let diags = check("mem=$(free -m | awk '/Mem/{print $2}')").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_uptime_command() {
        let diags = check("load=$(uptime | awk '{print $NF}')").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_ps_command() {
        let diags = check("procs=$(ps aux | wc -l)").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_who_command() {
        let diags = check("users=$(who | wc -l)").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_netstat_command() {
        let diags = check("conns=$(netstat -an | grep ESTABLISHED | wc -l)").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_no_false_positive_echo() {
        let diags = check("echo hello world").diagnostics;
        assert!(diags.is_empty());
    }

    #[test]
    fn test_no_false_positive_date() {
        // date is handled by DET002, not DET004
        let diags = check("ts=$(date +%s)").diagnostics;
        assert!(diags.is_empty());
    }

    #[test]
    fn test_nproc_command() {
        let diags = check("cores=$(nproc)").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_backtick_df() {
        let diags = check("avail=`df -m /`").diagnostics;
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_command_in_pipeline_helper() {
        assert!(command_in_pipeline("df -m / | awk 'NR==2{print $4}'", "df"));
        assert!(command_in_pipeline("cat /proc/meminfo | free -m", "free"));
        assert!(!command_in_pipeline("echo hello", "df"));
        assert!(!command_in_pipeline("dfile=test", "df"));
    }

    #[test]
    fn test_gen019_disk_space_conditional() {
        // The exact GEN-019 generalization test case
        let script = r#"avail=$(df -m / | awk 'NR==2{print $4}'); [ "$avail" -lt 100 ] && cleanup"#;
        let diags = check(script).diagnostics;
        assert!(!diags.is_empty(), "Should catch df as non-deterministic");
    }
}
