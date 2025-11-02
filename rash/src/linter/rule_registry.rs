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

        // === BATCH 2 CLASSIFICATIONS (25 rules) ===

        // [[ ]] test syntax rules (NotSh - bash/zsh/ksh only)
        registry.insert("SC2108", RuleMetadata {
            id: "SC2108",
            name: "In [[ ]], use && instead of -a",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2109", RuleMetadata {
            id: "SC2109",
            name: "In [[ ]], use || instead of -o",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2110", RuleMetadata {
            id: "SC2110",
            name: "Don't mix && and || with -a and -o in [[ ]]",
            compatibility: ShellCompatibility::NotSh,
        });

        // function keyword rules (NotSh - bash/ksh only, not POSIX)
        registry.insert("SC2111", RuleMetadata {
            id: "SC2111",
            name: "'function' keyword not supported in sh",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2112", RuleMetadata {
            id: "SC2112",
            name: "'function' keyword is non-standard",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2113", RuleMetadata {
            id: "SC2113",
            name: "'function' keyword with () is redundant",
            compatibility: ShellCompatibility::NotSh,
        });

        // Arithmetic expansion rules (Universal - $((...)) is POSIX)
        registry.insert("SC2003", RuleMetadata {
            id: "SC2003",
            name: "expr is antiquated. Use $((...))",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2004", RuleMetadata {
            id: "SC2004",
            name: "$/${} unnecessary on arithmetic variables",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2079", RuleMetadata {
            id: "SC2079",
            name: "Decimals not supported in (( ))",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2080", RuleMetadata {
            id: "SC2080",
            name: "Leading zero interpreted as octal",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2084", RuleMetadata {
            id: "SC2084",
            name: "Arithmetic expansion as command",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2085", RuleMetadata {
            id: "SC2085",
            name: "Local variable with arithmetic",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2133", RuleMetadata {
            id: "SC2133",
            name: "Unexpected tokens in arithmetic expansion",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2134", RuleMetadata {
            id: "SC2134",
            name: "Use (( )) for numeric tests",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2137", RuleMetadata {
            id: "SC2137",
            name: "Unnecessary braces in arithmetic",
            compatibility: ShellCompatibility::Universal,
        });

        // Quoting and subshell rules (Universal - POSIX concepts)
        registry.insert("SC2030", RuleMetadata {
            id: "SC2030",
            name: "Variable modified in subshell",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2031", RuleMetadata {
            id: "SC2031",
            name: "Variable was modified in subshell",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2032", RuleMetadata {
            id: "SC2032",
            name: "Variable in script with shebang",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2087", RuleMetadata {
            id: "SC2087",
            name: "Quote variables in sh -c",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2088", RuleMetadata {
            id: "SC2088",
            name: "Tilde expansion in quotes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2089", RuleMetadata {
            id: "SC2089",
            name: "Quotes in assignment treated literally",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2090", RuleMetadata {
            id: "SC2090",
            name: "Quotes in expansion treated literally",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2091", RuleMetadata {
            id: "SC2091",
            name: "Remove $() to avoid executing output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2092", RuleMetadata {
            id: "SC2092",
            name: "Remove backticks to avoid executing output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2093", RuleMetadata {
            id: "SC2093",
            name: "Remove exec if script should continue",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 3 CLASSIFICATIONS (30 rules) ===

        // Loop and iteration safety (Universal)
        registry.insert("SC2038", RuleMetadata {
            id: "SC2038",
            name: "Use -print0/-0 or find -exec instead of for loop over find",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2040", RuleMetadata {
            id: "SC2040",
            name: "Avoid passing -o to other commands (shell option confusion)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2041", RuleMetadata {
            id: "SC2041",
            name: "Use while read, not read in for loop",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2042", RuleMetadata {
            id: "SC2042",
            name: "Use printf instead of echo with backslash escapes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2043", RuleMetadata {
            id: "SC2043",
            name: "This loop will only run once (for x in y without wildcards)",
            compatibility: ShellCompatibility::Universal,
        });

        // Test operators and conditionals (Universal)
        registry.insert("SC2044", RuleMetadata {
            id: "SC2044",
            name: "For loops over find: use find -exec or process substitution",
            compatibility: ShellCompatibility::NotSh, // process substitution suggestion
        });
        registry.insert("SC2045", RuleMetadata {
            id: "SC2045",
            name: "Iterating over ls output is fragile",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2046", RuleMetadata {
            id: "SC2046",
            name: "Quote to prevent word splitting (CRITICAL)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2047", RuleMetadata {
            id: "SC2047",
            name: "Quote variables in [ ] to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2048", RuleMetadata {
            id: "SC2048",
            name: "Use \"$@\" (with quotes) to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2049", RuleMetadata {
            id: "SC2049",
            name: "Use =~ for regex matching (not = in [ ])",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2050", RuleMetadata {
            id: "SC2050",
            name: "This expression is constant (forgot $ on variable?)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2051", RuleMetadata {
            id: "SC2051",
            name: "Bash doesn't expand variables in brace ranges {$a..$b}",
            compatibility: ShellCompatibility::Universal,
        });

        // Quoting and glob safety (mostly Universal, one NotSh)
        registry.insert("SC2052", RuleMetadata {
            id: "SC2052",
            name: "Use [[ ]] instead of [ ] for glob patterns",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh
        });
        registry.insert("SC2053", RuleMetadata {
            id: "SC2053",
            name: "Quote RHS of = in [ ] to prevent glob matching",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2054", RuleMetadata {
            id: "SC2054",
            name: "Comma is just literal in [[ ]]; use array or separate comparison",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2055", RuleMetadata {
            id: "SC2055",
            name: "Deprecated -a operator in test (use &&)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2056", RuleMetadata {
            id: "SC2056",
            name: "Deprecated -o operator in test (use ||)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2057", RuleMetadata {
            id: "SC2057",
            name: "Unknown binary operator (===, =!, <>)",
            compatibility: ShellCompatibility::Universal,
        });
        // SC2058: Not implemented yet (unknown unary operator)
        // registry.insert("SC2058", RuleMetadata {
        //     id: "SC2058",
        //     name: "Unknown unary operator in test",
        //     compatibility: ShellCompatibility::Universal,
        // });

        // Command safety and redirection (Universal - CRITICAL security rules)
        registry.insert("SC2059", RuleMetadata {
            id: "SC2059",
            name: "Printf format string injection (CRITICAL security)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2060", RuleMetadata {
            id: "SC2060",
            name: "Unquoted tr parameters (glob expansion)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2061", RuleMetadata {
            id: "SC2061",
            name: "Quote parameters to tr to prevent globbing",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2062", RuleMetadata {
            id: "SC2062",
            name: "Grep pattern glob expansion prevention",
            compatibility: ShellCompatibility::Universal,
        });

        // Trap and signal handling (Universal - including CRITICAL timing bug)
        registry.insert("SC2063", RuleMetadata {
            id: "SC2063",
            name: "Grep regex vs literal string matching",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2064", RuleMetadata {
            id: "SC2064",
            name: "Trap command quoting (CRITICAL - timing bug)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2065", RuleMetadata {
            id: "SC2065",
            name: "Shell redirection interpretation in strings",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2066", RuleMetadata {
            id: "SC2066",
            name: "Missing semicolon before done in for loop",
            compatibility: ShellCompatibility::Universal,
        });

        // Most other SC2xxx rules are Universal (quoting, syntax, etc.)
        // They represent bugs or issues that apply regardless of shell
        // Examples: SC2086 (quote variables), etc.
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
    fn test_registry_has_72_rules() {
        // Batch 1: 8 SEC + 3 DET + 3 IDEM + 6 SC2xxx = 20 rules
        // Batch 2: 6 NotSh + 19 Universal = 25 rules
        // Batch 3: 2 NotSh + 25 Universal = 27 rules (SC2058 not implemented yet)
        // Total: 72 rules (20.2% of 357 total)
        assert_eq!(RULE_REGISTRY.len(), 72);
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

    // === Batch 2 Classification Tests ===

    #[test]
    fn test_double_bracket_rules_not_sh() {
        // [[ ]] syntax rules (SC2108-SC2110) should be NotSh
        let double_bracket_rules = vec!["SC2108", "SC2109", "SC2110"];

        for rule in double_bracket_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh),
                "{} should be NotSh",
                rule
            );

            // Should NOT apply to POSIX sh
            assert!(
                !should_apply_rule(rule, ShellType::Sh),
                "{} should not apply to sh",
                rule
            );

            // But SHOULD apply to bash and zsh
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Zsh),
                "{} should apply to zsh",
                rule
            );
        }
    }

    #[test]
    fn test_function_keyword_rules_not_sh() {
        // function keyword rules (SC2111-SC2113) should be NotSh
        let function_rules = vec!["SC2111", "SC2112", "SC2113"];

        for rule in function_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh),
                "{} should be NotSh",
                rule
            );

            // Should NOT apply to POSIX sh
            assert!(
                !should_apply_rule(rule, ShellType::Sh),
                "{} should not apply to sh",
                rule
            );

            // But SHOULD apply to bash and zsh
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
        }
    }

    #[test]
    fn test_arithmetic_rules_universal() {
        // Arithmetic rules (SC2003, SC2004, SC2079, SC2080, SC2084, SC2085, SC2133, SC2134, SC2137)
        let arithmetic_rules = vec![
            "SC2003", "SC2004", "SC2079", "SC2080", "SC2084", "SC2085", "SC2133", "SC2134",
            "SC2137",
        ];

        for rule in arithmetic_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );

            // Should apply to ALL shells
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Zsh),
                "{} should apply to zsh",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Sh),
                "{} should apply to sh",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Ksh),
                "{} should apply to ksh",
                rule
            );
        }
    }

    #[test]
    fn test_quoting_rules_universal() {
        // Quoting and subshell rules (SC2030, SC2031, SC2032, SC2087-SC2093)
        let quoting_rules = vec![
            "SC2030", "SC2031", "SC2032", "SC2087", "SC2088", "SC2089", "SC2090", "SC2091",
            "SC2092", "SC2093",
        ];

        for rule in quoting_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );

            // Should apply to ALL shells
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Sh),
                "{} should apply to sh",
                rule
            );
        }
    }

    #[test]
    fn test_batch2_notsh_count() {
        // Batch 2 should have 6 NotSh rules
        let notsh_rules = vec![
            "SC2108", "SC2109", "SC2110", // [[ ]]
            "SC2111", "SC2112", "SC2113", // function keyword
        ];

        for rule in notsh_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh)
            );
        }
    }

    #[test]
    fn test_batch2_universal_count() {
        // Batch 2 should have 19 Universal rules
        let universal_rules = vec![
            // Arithmetic (9 rules)
            "SC2003", "SC2004", "SC2079", "SC2080", "SC2084", "SC2085", "SC2133", "SC2134",
            "SC2137", // Quoting (10 rules)
            "SC2030", "SC2031", "SC2032", "SC2087", "SC2088", "SC2089", "SC2090", "SC2091",
            "SC2092", "SC2093",
        ];

        assert_eq!(universal_rules.len(), 19);

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal)
            );
        }
    }

    // === Batch 3 Classification Tests ===

    #[test]
    fn test_batch3_loop_safety_rules_universal() {
        // Loop safety rules (SC2038, SC2040-SC2043) should be Universal
        let loop_rules = vec!["SC2038", "SC2040", "SC2041", "SC2042", "SC2043"];

        for rule in loop_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );

            // Should apply to ALL shells
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Sh),
                "{} should apply to sh",
                rule
            );
        }
    }

    #[test]
    fn test_batch3_test_operators_mostly_universal() {
        // Most test operator rules are Universal
        let universal_test_rules = vec![
            "SC2045", "SC2046", "SC2047", "SC2048", "SC2049", "SC2050", "SC2051",
        ];

        for rule in universal_test_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }

        // SC2044 and SC2052 are NotSh (process substitution / [[ ]])
        assert_eq!(
            get_rule_compatibility("SC2044"),
            Some(ShellCompatibility::NotSh),
            "SC2044 should be NotSh (process substitution)"
        );
        assert_eq!(
            get_rule_compatibility("SC2052"),
            Some(ShellCompatibility::NotSh),
            "SC2052 should be NotSh ([[ ]] syntax)"
        );
    }

    #[test]
    fn test_batch3_critical_security_rules_universal() {
        // CRITICAL security rules must be Universal
        let critical_rules = vec![
            ("SC2059", "Printf format string injection"),
            ("SC2064", "Trap command timing bug"),
        ];

        for (rule, description) in critical_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} ({}) should be Universal - applies to all shells",
                rule,
                description
            );

            // Must apply to ALL shells
            for shell in [
                ShellType::Bash,
                ShellType::Zsh,
                ShellType::Sh,
                ShellType::Ksh,
            ] {
                assert!(
                    should_apply_rule(rule, shell),
                    "{} should apply to {:?}",
                    rule,
                    shell
                );
            }
        }
    }

    #[test]
    fn test_batch3_quoting_rules_universal() {
        // Quoting and glob safety rules (SC2053-SC2057, SC2060-SC2063, SC2065-SC2066) - SC2058 not implemented
        let quoting_rules = vec![
            "SC2053", "SC2054", "SC2055", "SC2056", "SC2057", "SC2060", "SC2061", "SC2062",
            "SC2063", "SC2065", "SC2066",
        ];

        for rule in quoting_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }

    #[test]
    fn test_batch3_notsh_count() {
        // Batch 3 should have 2 NotSh rules
        let notsh_rules = vec![
            "SC2044", // process substitution suggestion
            "SC2052", // [[ ]] for globs
        ];

        for rule in notsh_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh)
            );

            // Should NOT apply to POSIX sh
            assert!(!should_apply_rule(rule, ShellType::Sh));

            // But SHOULD apply to bash/zsh
            assert!(should_apply_rule(rule, ShellType::Bash));
            assert!(should_apply_rule(rule, ShellType::Zsh));
        }
    }

    #[test]
    fn test_batch3_universal_count() {
        // Batch 3 should have 25 Universal rules (SC2044 and SC2052 are NotSh, SC2058 not implemented)
        let universal_rules = vec![
            // Loop safety (5)
            "SC2038", "SC2040", "SC2041", "SC2042",
            "SC2043", // Test operators (7, excluding SC2044 which is NotSh)
            "SC2045", "SC2046", "SC2047", "SC2048", "SC2049", "SC2050", "SC2051",
            // Quoting/glob (9, excluding SC2052 NotSh and SC2058 not implemented)
            "SC2053", "SC2054", "SC2055", "SC2056", "SC2057", "SC2060", "SC2061", "SC2062",
            "SC2063", // Security and trap (4)
            "SC2059", // format injection
            "SC2064", // trap timing
            "SC2065", // shell redirection
            "SC2066", // missing semicolon
        ];

        // Should be 25 unique rules
        let unique_count = universal_rules.len();
        assert_eq!(unique_count, 25, "Batch 3 should have 25 Universal rules");

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }
}
