// Rule Registry
// Central metadata registry for all linter rules with shell compatibility

use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

/// Rule metadata including shell compatibility
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub compatibility: ShellCompatibility,
}

/// Get compatibility for a specific rule ID
pub fn get_rule_compatibility(rule_id: &str) -> Option<ShellCompatibility> {
    RULE_REGISTRY.get(rule_id).map(|meta| meta.compatibility)
}

/// Check if a rule should be applied for the given shell type
pub fn should_apply_rule(rule_id: &str, shell: crate::linter::shell_type::ShellType) -> bool {
    if let Some(compat) = get_rule_compatibility(rule_id) {
        compat.applies_to(shell)
    } else {
        // If rule not in registry, assume universal (conservative approach)
        true
    }
}

lazy_static::lazy_static! {
    /// Central registry of all linter rules with their compatibility
    static ref RULE_REGISTRY: HashMap<&'static str, RuleMetadata> = {
        let mut registry = HashMap::new();

        // Security Rules (8 rules) - Universal
        registry.insert("SEC001", RuleMetadata {
            id: "SEC001",
            name: "Command injection vulnerability",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC002", RuleMetadata {
            id: "SEC002",
            name: "Unsafe eval usage",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC003", RuleMetadata {
            id: "SEC003",
            name: "Unquoted variables (injection risk)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC004", RuleMetadata {
            id: "SEC004",
            name: "User input in commands",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC005", RuleMetadata {
            id: "SEC005",
            name: "Unsafe PATH modification",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC006", RuleMetadata {
            id: "SEC006",
            name: "Dangerous rm patterns",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC007", RuleMetadata {
            id: "SEC007",
            name: "Insecure temp file creation",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC008", RuleMetadata {
            id: "SEC008",
            name: "Source untrusted files",
            compatibility: ShellCompatibility::Universal,
        });

        // Determinism Rules (3 rules) - Universal
        registry.insert("DET001", RuleMetadata {
            id: "DET001",
            name: "$RANDOM usage (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("DET002", RuleMetadata {
            id: "DET002",
            name: "Timestamp usage (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("DET003", RuleMetadata {
            id: "DET003",
            name: "Wildcard ordering (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        });

        // Idempotency Rules (3 rules) - Universal
        registry.insert("IDEM001", RuleMetadata {
            id: "IDEM001",
            name: "mkdir without -p (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("IDEM002", RuleMetadata {
            id: "IDEM002",
            name: "rm without -f (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("IDEM003", RuleMetadata {
            id: "IDEM003",
            name: "ln without -sf (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        });

        // Bash-only rules: Arrays, [[]], process substitution, etc.
        // These should not fire on POSIX sh or pure zsh scripts

        // SC2039: Features undefined in POSIX sh (bash/zsh specific)
        registry.insert("SC2039", RuleMetadata {
            id: "SC2039",
            name: "Bash features undefined in POSIX sh",
            compatibility: ShellCompatibility::NotSh, // Works in bash/zsh/ksh but not sh
        });

        // SC2198: Arrays are bash-specific
        registry.insert("SC2198", RuleMetadata {
            id: "SC2198",
            name: "Array syntax (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // SC2199: Arrays are bash-specific
        registry.insert("SC2199", RuleMetadata {
            id: "SC2199",
            name: "Array expansion (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // SC2200: Arrays are bash-specific
        registry.insert("SC2200", RuleMetadata {
            id: "SC2200",
            name: "Array iteration (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // SC2201: Arrays are bash-specific
        registry.insert("SC2201", RuleMetadata {
            id: "SC2201",
            name: "Array assignment (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // Process substitution rules (bash/zsh, not POSIX sh)
        registry.insert("SC2002", RuleMetadata {
            id: "SC2002",
            name: "Useless cat (can use process substitution in bash/zsh)",
            compatibility: ShellCompatibility::NotSh,
        });

        // Most other SC2xxx rules are Universal (quoting, syntax, etc.)
        // They represent bugs or issues that apply regardless of shell
        // Examples: SC2046 (quote substitutions), SC2086 (quote variables)
        // These will be added as "Universal" as we classify them

        registry
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::shell_type::ShellType;

    #[test]
    fn test_sec_rules_are_universal() {
        for i in 1..=8 {
            let rule_id = format!("SEC{:03}", i);
            let compat = get_rule_compatibility(&rule_id);
            assert_eq!(compat, Some(ShellCompatibility::Universal));
        }
    }

    #[test]
    fn test_det_rules_are_universal() {
        for i in 1..=3 {
            let rule_id = format!("DET{:03}", i);
            let compat = get_rule_compatibility(&rule_id);
            assert_eq!(compat, Some(ShellCompatibility::Universal));
        }
    }

    #[test]
    fn test_idem_rules_are_universal() {
        for i in 1..=3 {
            let rule_id = format!("IDEM{:03}", i);
            let compat = get_rule_compatibility(&rule_id);
            assert_eq!(compat, Some(ShellCompatibility::Universal));
        }
    }

    #[test]
    fn test_should_apply_universal_rules_to_all_shells() {
        let shells = vec![
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Sh,
            ShellType::Ksh,
            ShellType::Auto,
        ];

        for shell in shells {
            assert!(should_apply_rule("SEC001", shell));
            assert!(should_apply_rule("DET001", shell));
            assert!(should_apply_rule("IDEM001", shell));
        }
    }

    #[test]
    fn test_unknown_rule_defaults_to_universal() {
        // Unknown rules default to universal (conservative)
        assert!(should_apply_rule("UNKNOWN999", ShellType::Bash));
        assert!(should_apply_rule("UNKNOWN999", ShellType::Zsh));
        assert!(should_apply_rule("UNKNOWN999", ShellType::Sh));
    }

    #[test]
    fn test_registry_has_20_rules() {
        // 8 SEC + 3 DET + 3 IDEM + 6 SC2xxx = 20 rules
        assert_eq!(RULE_REGISTRY.len(), 20);
    }

    #[test]
    fn test_bash_specific_rules_not_sh() {
        // Array and process substitution rules should be NotSh
        assert_eq!(
            get_rule_compatibility("SC2198"),
            Some(ShellCompatibility::NotSh)
        );
        assert_eq!(
            get_rule_compatibility("SC2199"),
            Some(ShellCompatibility::NotSh)
        );
        assert_eq!(
            get_rule_compatibility("SC2039"),
            Some(ShellCompatibility::NotSh)
        );
    }

    #[test]
    fn test_should_skip_bash_rules_for_sh() {
        // Bash-specific rules should not apply to POSIX sh
        assert!(!should_apply_rule("SC2198", ShellType::Sh));
        assert!(!should_apply_rule("SC2199", ShellType::Sh));

        // But should apply to bash and zsh
        assert!(should_apply_rule("SC2198", ShellType::Bash));
        assert!(should_apply_rule("SC2198", ShellType::Zsh));
    }
}
