//! CWE taxonomy mapping for bashrs linter rules.
//!
//! Maps each linter rule (SEC/DET/IDEM) to its MITRE CWE identifier,
//! CVSS v3.1 base score, and OWASP category. Used by:
//! - ShellSafetyBench benchmark export (S14.4)
//! - Eval harness CWE scoring metric (S14.5)
//! - Model card / dataset card generation

/// A CWE mapping entry for a single linter rule.
#[derive(Debug, Clone)]
pub struct CweMapping {
    /// bashrs rule ID (e.g., "SEC001")
    pub rule: &'static str,
    /// Human-readable pattern description
    pub pattern: &'static str,
    /// MITRE CWE identifier (e.g., "CWE-78")
    pub cwe: &'static str,
    /// Numeric CWE ID (e.g., 78)
    pub cwe_id: u32,
    /// CVSS v3.1 base score (0.0 - 10.0)
    pub cvss_score: f64,
    /// CVSS severity label
    pub cvss_severity: CvssSeverity,
    /// OWASP category
    pub owasp: &'static str,
}

/// CVSS v3.1 severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvssSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl CvssSeverity {
    /// Get severity from CVSS score per FIRST.org specification.
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s == 0.0 => Self::None,
            s if s <= 3.9 => Self::Low,
            s if s <= 6.9 => Self::Medium,
            s if s <= 8.9 => Self::High,
            _ => Self::Critical,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

impl std::fmt::Display for CvssSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// OOD (out-of-distribution) CWE for eval only — NOT in bashrs linter.
#[derive(Debug, Clone)]
pub struct OodCwe {
    /// MITRE CWE identifier
    pub cwe: &'static str,
    /// Numeric CWE ID
    pub cwe_id: u32,
    /// Human-readable name
    pub name: &'static str,
    /// Description of the vulnerability pattern
    pub description: &'static str,
    /// CVSS v3.1 base score
    pub cvss_score: f64,
    /// CVSS severity
    pub cvss_severity: CvssSeverity,
}

/// Complete CWE mapping table for all 14 bashrs linter rules.
pub static CWE_MAPPINGS: &[CweMapping] = &[
    // Security rules
    CweMapping {
        rule: "SEC001",
        pattern: "Unquoted variable expansion",
        cwe: "CWE-78",
        cwe_id: 78,
        cvss_score: 7.8,
        cvss_severity: CvssSeverity::High,
        owasp: "OS Command Injection",
    },
    CweMapping {
        rule: "SEC002",
        pattern: "eval usage",
        cwe: "CWE-94",
        cwe_id: 94,
        cvss_score: 8.8,
        cvss_severity: CvssSeverity::High,
        owasp: "Code Injection",
    },
    CweMapping {
        rule: "SEC003",
        pattern: "Unquoted command substitution",
        cwe: "CWE-78",
        cwe_id: 78,
        cvss_score: 7.8,
        cvss_severity: CvssSeverity::High,
        owasp: "OS Command Injection",
    },
    CweMapping {
        rule: "SEC004",
        pattern: "Backtick command substitution",
        cwe: "CWE-78",
        cwe_id: 78,
        cvss_score: 7.8,
        cvss_severity: CvssSeverity::High,
        owasp: "OS Command Injection",
    },
    CweMapping {
        rule: "SEC005",
        pattern: "Source/eval of variable",
        cwe: "CWE-94",
        cwe_id: 94,
        cvss_score: 8.8,
        cvss_severity: CvssSeverity::High,
        owasp: "Code Injection",
    },
    CweMapping {
        rule: "SEC006",
        pattern: "Curl piped to shell",
        cwe: "CWE-829",
        cwe_id: 829,
        cvss_score: 9.8,
        cvss_severity: CvssSeverity::Critical,
        owasp: "Inclusion of Untrusted Functionality",
    },
    CweMapping {
        rule: "SEC007",
        pattern: "World-writable permissions",
        cwe: "CWE-732",
        cwe_id: 732,
        cvss_score: 5.3,
        cvss_severity: CvssSeverity::Medium,
        owasp: "Incorrect Permission Assignment",
    },
    CweMapping {
        rule: "SEC008",
        pattern: "Hardcoded credentials",
        cwe: "CWE-798",
        cwe_id: 798,
        cvss_score: 7.5,
        cvss_severity: CvssSeverity::High,
        owasp: "Use of Hard-coded Credentials",
    },
    CweMapping {
        rule: "SEC013",
        pattern: "Insecure /tmp usage",
        cwe: "CWE-377",
        cwe_id: 377,
        cvss_score: 5.9,
        cvss_severity: CvssSeverity::Medium,
        owasp: "Insecure Temporary File",
    },
    // Determinism rules
    CweMapping {
        rule: "DET001",
        pattern: "$RANDOM usage",
        cwe: "CWE-330",
        cwe_id: 330,
        cvss_score: 3.7,
        cvss_severity: CvssSeverity::Low,
        owasp: "Insufficient Randomness",
    },
    CweMapping {
        rule: "DET002",
        pattern: "Timestamp in output",
        cwe: "CWE-330",
        cwe_id: 330,
        cvss_score: 3.7,
        cvss_severity: CvssSeverity::Low,
        owasp: "Insufficient Randomness",
    },
    CweMapping {
        rule: "DET003",
        pattern: "Unsorted glob expansion",
        cwe: "CWE-330",
        cwe_id: 330,
        cvss_score: 3.7,
        cvss_severity: CvssSeverity::Low,
        owasp: "Insufficient Randomness",
    },
    // Idempotency rules
    CweMapping {
        rule: "IDEM001",
        pattern: "mkdir without -p",
        cwe: "CWE-362",
        cwe_id: 362,
        cvss_score: 4.7,
        cvss_severity: CvssSeverity::Medium,
        owasp: "Race Condition (TOCTOU)",
    },
    CweMapping {
        rule: "IDEM002",
        pattern: "rm without -f",
        cwe: "CWE-362",
        cwe_id: 362,
        cvss_score: 4.7,
        cvss_severity: CvssSeverity::Medium,
        owasp: "Race Condition (TOCTOU)",
    },
];

/// OOD CWEs for eval-only (not in bashrs linter, tests generalization).
pub static OOD_CWES: &[OodCwe] = &[
    OodCwe {
        cwe: "CWE-426",
        cwe_id: 426,
        name: "Untrusted Search Path",
        description: "PATH manipulation allows execution of attacker-controlled binaries",
        cvss_score: 7.8,
        cvss_severity: CvssSeverity::High,
    },
    OodCwe {
        cwe: "CWE-77",
        cwe_id: 77,
        name: "Command Injection via xargs",
        description: "xargs without -0 splits on whitespace, enabling injection",
        cvss_score: 8.1,
        cvss_severity: CvssSeverity::High,
    },
    OodCwe {
        cwe: "CWE-116",
        cwe_id: 116,
        name: "Improper Output Encoding",
        description: "Log injection via echo of untrusted data (ANSI escapes, newlines)",
        cvss_score: 5.3,
        cvss_severity: CvssSeverity::Medium,
    },
    OodCwe {
        cwe: "CWE-250",
        cwe_id: 250,
        name: "Execution with Unnecessary Privileges",
        description: "Unnecessary sudo in scripts creates privilege escalation risk",
        cvss_score: 7.8,
        cvss_severity: CvssSeverity::High,
    },
];

/// Look up CWE mapping for a given rule ID.
pub fn lookup_rule(rule_id: &str) -> Option<&'static CweMapping> {
    CWE_MAPPINGS.iter().find(|m| m.rule == rule_id)
}

/// Get all unique CWE IDs covered by the linter.
pub fn linter_cwe_ids() -> Vec<u32> {
    let mut ids: Vec<u32> = CWE_MAPPINGS.iter().map(|m| m.cwe_id).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Get all OOD CWE IDs (for eval only).
pub fn ood_cwe_ids() -> Vec<u32> {
    OOD_CWES.iter().map(|o| o.cwe_id).collect()
}

/// Verify that OOD CWEs do not overlap with linter CWEs.
pub fn verify_ood_disjoint() -> bool {
    let linter_ids = linter_cwe_ids();
    OOD_CWES.iter().all(|ood| !linter_ids.contains(&ood.cwe_id))
}

/// Get a summary string for display.
pub fn summary() -> String {
    let linter_ids = linter_cwe_ids();
    let ood_ids = ood_cwe_ids();
    format!(
        "{} rules → {} unique CWEs (linter), {} OOD CWEs (eval-only), disjoint={}",
        CWE_MAPPINGS.len(),
        linter_ids.len(),
        ood_ids.len(),
        verify_ood_disjoint()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cwe_mapping_covers_all_rules() {
        // FALSIFY-SSB-001: All 14 linter rules must be mapped
        assert_eq!(CWE_MAPPINGS.len(), 14);

        let expected_rules = [
            "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
            "SEC013", "DET001", "DET002", "DET003", "IDEM001", "IDEM002",
        ];
        for rule in &expected_rules {
            assert!(
                lookup_rule(rule).is_some(),
                "Missing CWE mapping for rule {}",
                rule
            );
        }
    }

    #[test]
    fn test_cvss_scores_valid() {
        // FALSIFY-SSB-005: All CVSS scores in [0.0, 10.0]
        for mapping in CWE_MAPPINGS {
            assert!(
                (0.0..=10.0).contains(&mapping.cvss_score),
                "Rule {} has invalid CVSS score: {}",
                mapping.rule,
                mapping.cvss_score
            );
            // Verify severity matches score
            let expected = CvssSeverity::from_score(mapping.cvss_score);
            assert_eq!(
                mapping.cvss_severity, expected,
                "Rule {} severity mismatch: declared={:?}, computed={:?} for score {}",
                mapping.rule, mapping.cvss_severity, expected, mapping.cvss_score
            );
        }
    }

    #[test]
    fn test_ood_cwes_disjoint() {
        // FALSIFY-SSB-004: OOD CWEs must not overlap with linter CWEs
        assert!(verify_ood_disjoint());
        let linter_ids = linter_cwe_ids();
        for ood in OOD_CWES {
            assert!(
                !linter_ids.contains(&ood.cwe_id),
                "OOD CWE {} overlaps with linter",
                ood.cwe
            );
        }
    }

    #[test]
    fn test_ood_cwes_count() {
        assert_eq!(OOD_CWES.len(), 4);
    }

    #[test]
    fn test_lookup_existing_rule() {
        let m = lookup_rule("SEC006").expect("SEC006 should exist");
        assert_eq!(m.cwe, "CWE-829");
        assert_eq!(m.cvss_score, 9.8);
        assert_eq!(m.cvss_severity, CvssSeverity::Critical);
    }

    #[test]
    fn test_lookup_nonexistent_rule() {
        assert!(lookup_rule("SEC999").is_none());
    }

    #[test]
    fn test_unique_cwe_ids() {
        let ids = linter_cwe_ids();
        // 14 rules map to fewer unique CWEs (e.g., SEC001/003/004 all map to CWE-78)
        assert!(ids.len() < CWE_MAPPINGS.len());
        assert!(ids.len() >= 7); // At least 7 unique CWEs
    }

    #[test]
    fn test_summary_format() {
        let s = summary();
        assert!(s.contains("14 rules"));
        assert!(s.contains("disjoint=true"));
    }

    #[test]
    fn test_cvss_severity_from_score() {
        assert_eq!(CvssSeverity::from_score(0.0), CvssSeverity::None);
        assert_eq!(CvssSeverity::from_score(2.5), CvssSeverity::Low);
        assert_eq!(CvssSeverity::from_score(5.0), CvssSeverity::Medium);
        assert_eq!(CvssSeverity::from_score(7.5), CvssSeverity::High);
        assert_eq!(CvssSeverity::from_score(9.8), CvssSeverity::Critical);
    }

    #[test]
    fn test_all_mappings_have_nonempty_fields() {
        for m in CWE_MAPPINGS {
            assert!(!m.rule.is_empty(), "Empty rule ID");
            assert!(!m.pattern.is_empty(), "Empty pattern for {}", m.rule);
            assert!(!m.cwe.is_empty(), "Empty CWE for {}", m.rule);
            assert!(!m.owasp.is_empty(), "Empty OWASP for {}", m.rule);
            assert!(m.cwe_id > 0, "Zero CWE ID for {}", m.rule);
        }
    }
}
