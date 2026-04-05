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
fn test_registry_has_373_rules() {
    // Batch 1: 8 SEC + 3 DET + 3 IDEM + 6 SC2xxx = 20 rules
    // Batch 2: 6 NotSh + 19 Universal = 25 rules
    // Batch 3: 2 NotSh + 26 Universal = 28 rules
    // Batch 4: 1 NotSh + 27 Universal = 28 rules (SC2120 has false positives, not enabled)
    // Batch 5: 0 NotSh + 20 Universal = 20 rules
    // Batch 6: 1 NotSh + 19 Universal = 20 rules
    // Batch 7: 0 NotSh + 20 Universal = 20 rules
    // Batch 8: 1 NotSh + 19 Universal = 20 rules
    // Batch 9: 5 NotSh + 15 Universal = 20 rules
    // Batch 10: 2 NotSh + 18 Universal = 20 rules
    // Batch 11: 0 NotSh + 20 Universal = 20 rules
    // Batch 12: 0 NotSh + 20 Universal = 20 rules
    // Batch 13: 0 NotSh + 20 Universal = 20 rules
    // Batch 14: 4 NotSh + 6 Universal = 10 rules
    // Batch 15: 2 NotSh + 11 Universal = 13 rules
    // Batch 16: 1 NotSh + 5 Universal = 6 rules
    // Batch 17: 5 NotSh + 16 Universal = 21 rules (ALL REMAINING UNCLASSIFIED)
    // Batch 18: 0 NotSh + 7 Universal = 7 rules (SC2008-SC2014 file/command best practices)
    // Batch 19: 0 NotSh + 20 Universal = 20 rules (MAKE001-MAKE020 Makefile linter rules)
    // Batch 20: 5 PERF + 5 PORT (ShOnly) + 5 REL = 15 rules (performance, portability, reliability)
    // SC1xxx: 60 rules (shebang, quoting, spacing, syntax, here-doc, unicode, portability, source)
    // Total: 396 rules
    assert_eq!(RULE_REGISTRY.len(), 396);
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
        "SC2003", "SC2004", "SC2079", "SC2080", "SC2084", "SC2085", "SC2133", "SC2134", "SC2137",
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
        "SC2030", "SC2031", "SC2032", "SC2087", "SC2088", "SC2089", "SC2090", "SC2091", "SC2092",
        "SC2093",
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
        "SC2030", "SC2031", "SC2032", "SC2087", "SC2088", "SC2089", "SC2090", "SC2091", "SC2092",
        "SC2093",
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
    // Quoting and glob safety rules (SC2053-SC2058, SC2060-SC2063, SC2065-SC2066)
    let quoting_rules = vec![
        "SC2053", "SC2054", "SC2055", "SC2056", "SC2057", "SC2058", "SC2060", "SC2061", "SC2062",
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
    // Batch 3 should have 26 Universal rules (SC2044 and SC2052 are NotSh)
    let universal_rules = vec![
        // Loop safety (5)
        "SC2038", "SC2040", "SC2041", "SC2042",
        "SC2043", // Test operators (7, excluding SC2044 which is NotSh)
        "SC2045", "SC2046", "SC2047", "SC2048", "SC2049", "SC2050", "SC2051",
        // Quoting/glob (10, excluding SC2052 NotSh)
        "SC2053", "SC2054", "SC2055", "SC2056", "SC2057", "SC2058", "SC2060", "SC2061", "SC2062",
        "SC2063", // Security and trap (4)
        "SC2059", // format injection
        "SC2064", // trap timing
        "SC2065", // shell redirection
        "SC2066", // missing semicolon
    ];

    // Should be 26 unique rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 26, "Batch 3 should have 26 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }
}

// === Batch 4 Classification Tests ===

#[test]
fn test_batch4_variable_safety_universal() {
    // Variable and parameter safety rules (SC2067-SC2074) should be Universal
    let variable_rules = vec![
        "SC2067", "SC2068", "SC2069", "SC2070", "SC2071", "SC2072", "SC2073", "SC2074",
    ];

    for rule in variable_rules {
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
fn test_batch4_quoting_safety_universal() {
    // Quote and expansion safety rules should be Universal
    let quoting_rules = vec![
        "SC2075", "SC2076", "SC2077", "SC2078", "SC2081", "SC2082", "SC2083",
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
