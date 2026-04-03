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

#[test]
fn test_batch4_command_safety_universal() {
    // Command and redirection safety rules should be Universal
    let command_rules = vec!["SC2094", "SC2095", "SC2096", "SC2097", "SC2098", "SC2103"];

    for rule in command_rules {
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
fn test_batch4_critical_dangerous_rm_universal() {
    // CRITICAL: Dangerous rm -rf rules (SC2114, SC2115) MUST be Universal
    let critical_rules = vec![
        ("SC2114", "Dangerous rm -rf without validation"),
        ("SC2115", "Use ${var:?} to ensure var is set before rm -rf"),
    ];

    for (rule, description) in critical_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} ({}) should be Universal - applies to all shells",
            rule,
            description
        );

        // Must apply to ALL shells (CRITICAL safety)
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
fn test_batch4_notsh_count() {
    // Batch 4 should have 1 NotSh rule (SC2120 has false positives, not enabled)
    let notsh_rules = vec![
        // "SC2120", // Function parameter analysis (has false positives, not enabled)
        "SC2128", // Array expansion without index
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
fn test_batch4_universal_count() {
    // Batch 4 should have 27 Universal rules
    let universal_rules = vec![
        // Variable safety (8)
        "SC2067", "SC2068", "SC2069", "SC2070", "SC2071", "SC2072", "SC2073", "SC2074",
        // Quoting safety (7)
        "SC2075", "SC2076", "SC2077", "SC2078", "SC2081", "SC2082", "SC2083",
        // Command safety (6)
        "SC2094", "SC2095", "SC2096", "SC2097", "SC2098", "SC2103",
        // Test safety (3)
        "SC2104", "SC2105", "SC2107", // CRITICAL dangerous rm (2)
        "SC2114", "SC2115", // Echo safety (1)
        "SC2116",
    ];

    // Total: 8+7+6+3+2+1 = 27 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 27, "Batch 4 should have 27 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }
}

// === Batch 5 Classification Tests ===

#[test]
fn test_batch5_command_optimization_universal() {
    // Command optimization rules (SC2001, SC2005-2007) should be Universal
    let command_rules = vec!["SC2001", "SC2005", "SC2006", "SC2007"];

    for rule in command_rules {
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
fn test_batch5_logic_and_tr_universal() {
    // Logic, quoting, and tr character class rules should be Universal
    let logic_and_tr_rules = vec![
        // Logic (3)
        "SC2015", "SC2016", "SC2017", // tr character classes (4)
        "SC2018", "SC2019", "SC2020", "SC2021",
    ];

    for rule in logic_and_tr_rules {
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
fn test_batch5_ssh_and_quoting_universal() {
    // SSH, sudo, quoting, and echo safety rules should be Universal
    let ssh_and_quoting_rules = vec![
        // SSH and command safety (5)
        "SC2022", "SC2023", "SC2024", "SC2025", "SC2026", // Quoting and echo (3)
        "SC2027", "SC2028", "SC2029",
    ];

    for rule in ssh_and_quoting_rules {
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
fn test_batch5_critical_word_splitting_universal() {
    // CRITICAL: SC2086 (quote to prevent word splitting) MUST be Universal
    let critical_rule = "SC2086";

    assert_eq!(
        get_rule_compatibility(critical_rule),
        Some(ShellCompatibility::Universal),
        "SC2086 (CRITICAL word splitting) should be Universal"
    );

    // Must apply to ALL shells (CRITICAL safety)
    for shell in [
        ShellType::Bash,
        ShellType::Zsh,
        ShellType::Sh,
        ShellType::Ksh,
    ] {
        assert!(
            should_apply_rule(critical_rule, shell),
            "SC2086 should apply to {:?}",
            shell
        );
    }
}

#[test]
fn test_batch5_universal_count() {
    // Batch 5 should have 20 Universal rules
    let universal_rules = vec![
        // Command optimization (4)
        "SC2001", "SC2005", "SC2006", "SC2007", // Logic and quoting (3)
        "SC2015", "SC2016", "SC2017", // tr character classes (4)
        "SC2018", "SC2019", "SC2020", "SC2021", // SSH and command safety (5)
        "SC2022", "SC2023", "SC2024", "SC2025", "SC2026", // Quoting and echo (3)
        "SC2027", "SC2028", "SC2029", // CRITICAL word splitting (1)
        "SC2086",
    ];

    // Total: 4+3+4+5+3+1 = 20 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 20, "Batch 5 should have 20 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }
}

// === Batch 6 Classification Tests ===

#[test]
fn test_batch6_variable_function_safety_universal() {
    // Variable and function safety rules (SC2033-2035) should be Universal
    let variable_rules = vec!["SC2033", "SC2034", "SC2035"];

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
fn test_batch6_command_best_practices_universal() {
    // Command best practices (SC2099-2102, SC2106, SC2117) should be Universal
    let command_rules = vec!["SC2099", "SC2100", "SC2101", "SC2102", "SC2106", "SC2117"];

    for rule in command_rules {
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
fn test_batch6_ksh_specific_notsh() {
    // SC2118 (ksh set -A arrays) should be NotSh
    let rule = "SC2118";

    assert_eq!(
        get_rule_compatibility(rule),
        Some(ShellCompatibility::NotSh),
        "{} should be NotSh (ksh-specific)",
        rule
    );

    // Should NOT apply to POSIX sh
    assert!(
        !should_apply_rule(rule, ShellType::Sh),
        "{} should not apply to sh",
        rule
    );

    // But SHOULD apply to bash/zsh/ksh
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

#[test]
fn test_batch6_quality_efficiency_universal() {
    // Quality/efficiency rules should be Universal
    let quality_rules = vec![
        "SC2121", "SC2122", "SC2126", "SC2127", "SC2129", "SC2130", "SC2131", "SC2132", "SC2135",
        "SC2136",
    ];

    for rule in quality_rules {
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
fn test_batch6_universal_count() {
    // Batch 6 should have 20 rules total (19 Universal + 1 NotSh)
    let universal_rules = vec![
        // Variable/function safety (3)
        "SC2033", "SC2034", "SC2035", // Command best practices (6)
        "SC2099", "SC2100", "SC2101", "SC2102", "SC2106", "SC2117",
        // Quality/efficiency (10)
        "SC2121", "SC2122", "SC2126", "SC2127", "SC2129", "SC2130", "SC2131", "SC2132", "SC2135",
        "SC2136",
    ];

    // 19 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 19, "Batch 6 should have 19 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }

    // 1 NotSh rule (ksh-specific)
    assert_eq!(
        get_rule_compatibility("SC2118"),
        Some(ShellCompatibility::NotSh),
        "SC2118 should be NotSh"
    );
}

// === Batch 7 Classification Tests ===

#[test]
fn test_batch7_alias_function_context_universal() {
    // Alias and function context safety rules (SC2138-SC2142) should be Universal
    let alias_function_rules = vec!["SC2138", "SC2139", "SC2140", "SC2141", "SC2142"];

    for rule in alias_function_rules {
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
fn test_batch7_find_glob_efficiency_universal() {
    // Find and glob efficiency rules (SC2143-SC2150) should be Universal
    let find_glob_rules = vec![
        "SC2143", "SC2144", "SC2145", "SC2146", "SC2147", "SC2148", "SC2149", "SC2150",
    ];

    for rule in find_glob_rules {
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
fn test_batch7_exit_code_safety_universal() {
    // Return/exit code and control flow safety rules (SC2151-SC2157) should be Universal
    let exit_code_rules = vec![
        "SC2151", "SC2152", "SC2153", "SC2154", "SC2155", "SC2156", "SC2157",
    ];

    for rule in exit_code_rules {
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
fn test_batch7_universal_count() {
    // Batch 7 should have 20 rules total (all Universal)
    let universal_rules = vec![
        // Alias/function context (5)
        "SC2138", "SC2139", "SC2140", "SC2141", "SC2142", // Find/glob efficiency (8)
        "SC2143", "SC2144", "SC2145", "SC2146", "SC2147", "SC2148", "SC2149", "SC2150",
        // Return/exit codes (7)
        "SC2151", "SC2152", "SC2153", "SC2154", "SC2155", "SC2156", "SC2157",
    ];

    // 20 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 20, "Batch 7 should have 20 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }
}

#[test]
fn test_batch7_no_notsh_rules() {
    // Batch 7 should have NO NotSh rules (all Universal)
    let batch7_rules = vec![
        "SC2138", "SC2139", "SC2140", "SC2141", "SC2142", "SC2143", "SC2144", "SC2145", "SC2146",
        "SC2147", "SC2148", "SC2149", "SC2150", "SC2151", "SC2152", "SC2153", "SC2154", "SC2155",
        "SC2156", "SC2157",
    ];

    for rule in batch7_rules {
        let compat = get_rule_compatibility(rule);
        assert_eq!(
            compat,
            Some(ShellCompatibility::Universal),
            "{} should be Universal (not NotSh)",
            rule
        );

        // Should apply to ALL shells including sh
        assert!(
            should_apply_rule(rule, ShellType::Sh),
            "{} should apply to sh",
            rule
        );
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
            should_apply_rule(rule, ShellType::Ksh),
            "{} should apply to ksh",
            rule
        );
    }
}

// === Batch 8 Classification Tests ===

#[test]
fn test_batch8_exit_code_bracket_universal() {
    // Exit code & bracket safety rules (SC2158-SC2161) should be Universal
    let exit_bracket_rules = vec!["SC2158", "SC2159", "SC2160", "SC2161"];

    for rule in exit_bracket_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
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
fn test_batch8_read_trap_safety_universal() {
    // read command and trap safety rules (SC2162-SC2167, excluding SC2168) should be Universal
    let read_trap_rules = vec!["SC2162", "SC2163", "SC2164", "SC2165", "SC2166", "SC2167"];

    for rule in read_trap_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch8_local_keyword_notsh() {
    // SC2168 ('local' keyword) should be NotSh (bash/ksh/zsh specific)
    let rule = "SC2168";

    assert_eq!(
        get_rule_compatibility(rule),
        Some(ShellCompatibility::NotSh),
        "{} should be NotSh (local is bash/ksh/zsh specific)",
        rule
    );

    // Should NOT apply to POSIX sh
    assert!(
        !should_apply_rule(rule, ShellType::Sh),
        "{} should not apply to sh",
        rule
    );

    // But SHOULD apply to bash/zsh/ksh
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
        should_apply_rule(rule, ShellType::Ksh),
        "{} should apply to ksh",
        rule
    );
}

#[test]
fn test_batch8_test_operators_universal() {
    // Test operators and security rules (SC2169-SC2177) should be Universal
    let test_security_rules = vec![
        "SC2169", "SC2170", "SC2171", "SC2172", "SC2173", "SC2174", "SC2175", "SC2176", "SC2177",
    ];

    for rule in test_security_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch8_universal_count() {
    // Batch 8 should have 20 rules total (19 Universal + 1 NotSh)
    let universal_rules = vec![
        // Exit code/bracket safety (4)
        "SC2158", "SC2159", "SC2160", "SC2161", // read command safety (3)
        "SC2162", "SC2163", "SC2164", // Trap/signal handling (3 Universal, SC2168 is NotSh)
        "SC2165", "SC2166", "SC2167", // Test operators & security (9)
        "SC2169", "SC2170", "SC2171", "SC2172", "SC2173", "SC2174", "SC2175", "SC2176", "SC2177",
    ];

    // 19 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 19, "Batch 8 should have 19 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }

    // 1 NotSh rule (local keyword)
    assert_eq!(
        get_rule_compatibility("SC2168"),
        Some(ShellCompatibility::NotSh),
        "SC2168 should be NotSh (local keyword is bash/ksh/zsh specific)"
    );

    // Total: 19 Universal + 1 NotSh = 20 rules
    // This brings total from 160 → 180 (50.4% coverage - 🎉 50% MILESTONE!)
}

// === Batch 9 Classification Tests ===

#[test]
fn test_batch9_array_operations_notsh() {
    // Array operations (SC2178-SC2180) should be NotSh (bash/zsh/ksh only)
    let array_rules = vec!["SC2178", "SC2179", "SC2180"];

    for rule in array_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh (arrays are bash/zsh/ksh specific)",
            rule
        );

        // Should NOT apply to POSIX sh
        assert!(
            !should_apply_rule(rule, ShellType::Sh),
            "{} should not apply to sh",
            rule
        );

        // But SHOULD apply to bash/zsh/ksh
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
fn test_batch9_associative_arrays_notsh() {
    // Associative arrays (SC2190-SC2191) should be NotSh (bash 4+/zsh)
    let assoc_array_rules = vec!["SC2190", "SC2191"];

    for rule in assoc_array_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh (associative arrays are bash 4+/zsh specific)",
            rule
        );

        // Should NOT apply to POSIX sh
        assert!(
            !should_apply_rule(rule, ShellType::Sh),
            "{} should not apply to sh",
            rule
        );

        // But SHOULD apply to bash/zsh
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
fn test_batch9_exit_code_patterns_universal() {
    // Exit code and printf patterns (SC2181-SC2182) should be Universal
    let exit_code_rules = vec!["SC2181", "SC2182"];

    for rule in exit_code_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch9_assignment_expansion_universal() {
    // Assignment, expansion, and command composition rules should be Universal
    let universal_rules = vec![
        "SC2183", "SC2184", "SC2185", "SC2186", "SC2187", "SC2188", "SC2189", "SC2192", "SC2193",
        "SC2194", "SC2195", "SC2196", "SC2197",
    ];

    for rule in universal_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch9_universal_count() {
    // Batch 9 should have 20 rules total (15 Universal + 5 NotSh)
    let universal_rules = vec![
        // Exit code/printf (2)
        "SC2181", "SC2182", // Assignment/expansion safety (4)
        "SC2183", "SC2184", "SC2185", "SC2186", // Shell directives/redirection (3)
        "SC2187", "SC2188", "SC2189", // Command composition/regex (6)
        "SC2192", "SC2193", "SC2194", "SC2195", "SC2196", "SC2197",
    ];

    // 15 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 15, "Batch 9 should have 15 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }

    // 5 NotSh rules (arrays)
    let notsh_rules = vec![
        "SC2178", "SC2179", "SC2180", // Array operations
        "SC2190", "SC2191", // Associative arrays
    ];

    for rule in notsh_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh (array operations)",
            rule
        );
    }

    // Total: 15 Universal + 5 NotSh = 20 rules
    // This brings total from 180 → 200 (56.0% coverage - Approaching 60%!)
}

// === Batch 10 Classification Tests ===

#[test]
fn test_batch10_array_quoting_notsh() {
    // Array quoting rules (SC2206-SC2207) should be NotSh (bash/zsh/ksh only)
    let array_rules = vec!["SC2206", "SC2207"];

    for rule in array_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh (arrays are bash/zsh/ksh specific)",
            rule
        );

        // Should NOT apply to POSIX sh
        assert!(
            !should_apply_rule(rule, ShellType::Sh),
            "{} should not apply to sh",
            rule
        );

        // But SHOULD apply to bash/zsh/ksh
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
fn test_batch10_command_structure_universal() {
    // Command structure rules should be Universal
    let command_rules = vec![
        "SC2202", "SC2203", "SC2204", "SC2205", "SC2208", "SC2209", "SC2216", "SC2217",
    ];

    for rule in command_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch10_arithmetic_operations_universal() {
    // Arithmetic operation rules should be Universal
    let arithmetic_rules = vec!["SC2210", "SC2211", "SC2214", "SC2215", "SC2220", "SC2221"];

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
            should_apply_rule(rule, ShellType::Sh),
            "{} should apply to sh",
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
fn test_batch10_control_flow_universal() {
    // Control flow and test operator rules should be Universal
    let control_flow_rules = vec!["SC2212", "SC2213", "SC2218", "SC2219"];

    for rule in control_flow_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch10_universal_count() {
    // Batch 10 should have 20 rules total (18 Universal + 2 NotSh)
    let universal_rules = vec![
        // Command structure (8)
        "SC2202", "SC2203", "SC2204", "SC2205", "SC2208", "SC2209", "SC2216", "SC2217",
        // Arithmetic operations (6)
        "SC2210", "SC2211", "SC2214", "SC2215", "SC2220", "SC2221",
        // Control flow (4)
        "SC2212", "SC2213", "SC2218", "SC2219",
    ];

    // 18 Universal rules
    let unique_count = universal_rules.len();
    assert_eq!(unique_count, 18, "Batch 10 should have 18 Universal rules");

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal",
            rule
        );
    }

    // 2 NotSh rules (arrays)
    let notsh_rules = vec![
        "SC2206", "SC2207", // Array quoting
    ];

    for rule in notsh_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh (array operations)",
            rule
        );
    }

    // Total: 18 Universal + 2 NotSh = 20 rules
    // This brings total from 200 → 220 (61.6% coverage - 🎯 CROSSED 60% MILESTONE! 🎯)
}

// === BATCH 11 TESTS ===

#[test]
fn test_batch11_case_statement_syntax_universal() {
    // Case statement syntax rules should be Universal (POSIX feature)
    let case_rules = vec!["SC2222", "SC2223"];

    for rule in case_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal (case is POSIX)",
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch11_control_flow_universal() {
    // Control flow and test operator rules should be Universal
    let control_flow_rules = vec!["SC2224", "SC2225", "SC2226", "SC2227", "SC2228", "SC2229"];

    for rule in control_flow_rules {
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch11_command_portability_universal() {
    // Command existence and portability rules should be Universal
    let portability_rules = vec!["SC2230", "SC2231", "SC2232", "SC2233", "SC2234"];

    for rule in portability_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal (POSIX portability)",
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch11_quoting_safety_universal() {
    // Quoting and expansion safety rules should be Universal
    let quoting_rules = vec![
        "SC2235", "SC2236", "SC2237", "SC2238", "SC2239", "SC2240", "SC2241",
    ];

    for rule in quoting_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal (quoting is universal)",
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch11_universal_count() {
    // Batch 11: All 20 rules are Universal (0 NotSh)
    // This validates our classification strategy

    let batch11_rules = vec![
        // Case statement (2)
        "SC2222", "SC2223", // Control flow (6)
        "SC2224", "SC2225", "SC2226", "SC2227", "SC2228", "SC2229",
        // Command portability (5)
        "SC2230", "SC2231", "SC2232", "SC2233", "SC2234", // Quoting safety (7)
        "SC2235", "SC2236", "SC2237", "SC2238", "SC2239", "SC2240", "SC2241",
    ];

    // All batch 11 rules should be Universal
    for rule in &batch11_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 11 rule {} should be Universal",
            rule
        );
    }

    // Verify count: 20 Universal rules
    assert_eq!(batch11_rules.len(), 20);

    // Total: 20 Universal + 0 NotSh = 20 rules
    // This brings total from 220 → 240 (67.2% coverage - Approaching 70% milestone!)
}

// === BATCH 12 TESTS ===

#[test]
fn test_batch12_control_flow_universal() {
    let control_rules = vec!["SC2242", "SC2243", "SC2244", "SC2245", "SC2246"];
    for rule in control_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch12_test_operators_universal() {
    let test_rules = vec!["SC2247", "SC2248", "SC2249", "SC2250", "SC2251"];
    for rule in test_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch12_loop_patterns_universal() {
    let loop_rules = vec!["SC2252", "SC2253", "SC2254", "SC2255", "SC2256"];
    for rule in loop_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch12_quoting_safety_universal() {
    let quoting_rules = vec!["SC2257", "SC2258", "SC2259", "SC2260", "SC2261"];
    for rule in quoting_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch12_universal_count() {
    let batch12_rules = vec![
        "SC2242", "SC2243", "SC2244", "SC2245", "SC2246", "SC2247", "SC2248", "SC2249", "SC2250",
        "SC2251", "SC2252", "SC2253", "SC2254", "SC2255", "SC2256", "SC2257", "SC2258", "SC2259",
        "SC2260", "SC2261",
    ];
    for rule in &batch12_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 12 rule {} should be Universal",
            rule
        );
    }
    assert_eq!(batch12_rules.len(), 20);
    // Total: 20 Universal + 0 NotSh = 20 rules
    // This brings total from 240 → 260 (72.8% coverage - 🎯 CROSSED 70% MILESTONE! 🎯)
}

// === BATCH 13 TESTS ===

#[test]
fn test_batch13_quoting_safety_universal() {
    let quoting_rules = vec![
        "SC2262", "SC2263", "SC2264", "SC2265", "SC2266", "SC2267", "SC2268", "SC2269",
    ];
    for rule in quoting_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch13_argument_parsing_universal() {
    let arg_parsing_rules = vec!["SC2270", "SC2271", "SC2272", "SC2273", "SC2274"];
    for rule in arg_parsing_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch13_word_splitting_universal() {
    let word_splitting_rules = vec![
        "SC2275", "SC2276", "SC2277", "SC2278", "SC2279", "SC2280", "SC2281",
    ];
    for rule in word_splitting_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal)
        );
        assert!(should_apply_rule(rule, ShellType::Bash));
        assert!(should_apply_rule(rule, ShellType::Sh));
        assert!(should_apply_rule(rule, ShellType::Zsh));
    }
}

#[test]
fn test_batch13_no_notsh_rules() {
    // Batch 13 should have NO NotSh rules (all Universal)
    let batch13_rules = vec![
        "SC2262", "SC2263", "SC2264", "SC2265", "SC2266", "SC2267", "SC2268", "SC2269", "SC2270",
        "SC2271", "SC2272", "SC2273", "SC2274", "SC2275", "SC2276", "SC2277", "SC2278", "SC2279",
        "SC2280", "SC2281",
    ];

    for rule in batch13_rules {
        let compat = get_rule_compatibility(rule);
        assert_eq!(
            compat,
            Some(ShellCompatibility::Universal),
            "{} should be Universal (not NotSh)",
            rule
        );

        // Should apply to ALL shells including sh
        assert!(
            should_apply_rule(rule, ShellType::Sh),
            "{} should apply to sh",
            rule
        );
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
            should_apply_rule(rule, ShellType::Ksh),
            "{} should apply to ksh",
            rule
        );
    }
}

#[test]
fn test_batch13_universal_count() {
    let batch13_rules = vec![
        "SC2262", "SC2263", "SC2264", "SC2265", "SC2266", "SC2267", "SC2268", "SC2269", "SC2270",
        "SC2271", "SC2272", "SC2273", "SC2274", "SC2275", "SC2276", "SC2277", "SC2278", "SC2279",
        "SC2280", "SC2281",
    ];
    for rule in &batch13_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 13 rule {} should be Universal",
            rule
        );
    }
    assert_eq!(batch13_rules.len(), 20);
    // Total: 20 Universal + 0 NotSh = 20 rules
    // This brings total from 260 → 280 (78.4% coverage - Approaching 80% milestone!)
}

// === BATCH 14 TESTS ===

#[test]
fn test_batch14_parameter_expansion_universal() {
    // Parameter expansion rules (SC2282-SC2285) should be Universal (POSIX)
    let param_expansion_rules = vec!["SC2282", "SC2283", "SC2284", "SC2285"];

    for rule in param_expansion_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} should be Universal (POSIX parameter expansion)",
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch14_bash_specific_notsh() {
    // Bash-specific rules (SC2286, SC2287, SC2290, SC2291) should be NotSh
    let bash_specific_rules = vec![
        ("SC2286", "mapfile/readarray bash 4+"),
        ("SC2287", "[[ -v var ]] bash/zsh/ksh"),
        ("SC2290", "arrays bash-specific"),
        ("SC2291", "[[ ! -v var ]] bash/zsh/ksh"),
    ];

    for (rule, description) in bash_specific_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} ({}) should be NotSh",
            rule,
            description
        );

        // Should NOT apply to POSIX sh
        assert!(
            !should_apply_rule(rule, ShellType::Sh),
            "{} should not apply to sh",
            rule
        );

        // But SHOULD apply to bash/zsh
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
fn test_batch14_style_rules_universal() {
    // Style and best practice rules (SC2288, SC2289) should be Universal
    let style_rules = vec![
        ("SC2288", "Use true/false instead of [ 1 = 1 ]"),
        ("SC2289", "Use ${#var} instead of expr length"),
    ];

    for (rule, description) in style_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "{} ({}) should be Universal",
            rule,
            description
        );

        // Should apply to ALL shells
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
fn test_batch14_notsh_count() {
    // Batch 14 should have 4 NotSh rules
    let notsh_rules = vec![
        "SC2286", // mapfile/readarray
        "SC2287", // [[ -v var ]]
        "SC2290", // array indexing
        "SC2291", // [[ ! -v var ]]
    ];

    for rule in notsh_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh",
            rule
        );
    }
}

#[test]
fn test_batch14_universal_count() {
    // Batch 14: 6 Universal + 4 NotSh = 10 rules
    let universal_rules = vec![
        // Parameter expansion (4)
        "SC2282", "SC2283", "SC2284", "SC2285", // Style (2)
        "SC2288", "SC2289",
    ];

    // 6 Universal rules
    assert_eq!(universal_rules.len(), 6);

    for rule in universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 14 rule {} should be Universal",
            rule
        );
    }

    // Total: 6 Universal + 4 NotSh = 10 rules
    // This brings total from 280 → 290 (81.2% coverage - 🎯 CROSSED 80% MILESTONE! 🎯)
}

// === BATCH 15 TESTS (5 tests) ===

#[test]
fn test_batch15_posix_parameter_expansion_universal() {
    // SC2307-SC2309, SC2311, SC2315 - POSIX parameter expansion
    let posix_expansion_rules = vec![
        ("SC2307", "${var#prefix}"),
        ("SC2308", "${var%suffix}"),
        ("SC2309", "${var##prefix}"),
        ("SC2311", "${var%%suffix}"),
        ("SC2315", "${var:+replacement}"),
    ];

    for (rule, description) in posix_expansion_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 15 POSIX rule {} ({}) should be Universal",
            rule,
            description
        );
    }
}

#[test]
fn test_batch15_bash_specific_notsh() {
    // SC2306, SC2314 - Bash-specific features
    let bash_specific_rules = vec![
        ("SC2306", "${var//old/new} bash expansion"),
        ("SC2314", "[[ ]] pattern matching"),
    ];

    for (rule, description) in bash_specific_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "Batch 15 bash-specific rule {} ({}) should be NotSh",
            rule,
            description
        );
    }
}

#[test]
fn test_batch15_control_flow_universal() {
    // SC2310, SC2316, SC2317 - Control flow & set -e behavior
    let control_flow_rules = vec![
        ("SC2310", "set -e in conditions"),
        ("SC2316", "command group precedence"),
        ("SC2317", "unreachable code"),
    ];

    for (rule, description) in control_flow_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 15 control flow rule {} ({}) should be Universal",
            rule,
            description
        );
    }
}

#[test]
fn test_batch15_deprecated_syntax_universal() {
    // SC2312, SC2313, SC2318 - Deprecated syntax warnings
    let deprecated_rules = vec![
        ("SC2312", "local -x deprecated"),
        ("SC2313", "use $(( ))"),
        ("SC2318", "$[ ] deprecated"),
    ];

    for (rule, description) in deprecated_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 15 deprecated syntax rule {} ({}) should be Universal",
            rule,
            description
        );
    }
}

#[test]
fn test_batch15_split_universal_vs_notsh() {
    // Batch 15: 11 Universal + 2 NotSh = 13 rules total
    let universal_rules = vec![
        "SC2307", "SC2308", "SC2309", "SC2311", "SC2315", // POSIX parameter expansion
        "SC2310", "SC2316", "SC2317", // Control flow
        "SC2312", "SC2313", "SC2318", // Deprecated syntax
    ];

    let notsh_rules = vec![
        "SC2306", // ${var//} bash expansion
        "SC2314", // [[ ]] pattern matching
    ];

    // Verify Universal rules
    for rule in &universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Rule {} should be Universal",
            rule
        );
    }

    // Verify NotSh rules
    for rule in &notsh_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "Rule {} should be NotSh",
            rule
        );
    }

    // Total: 11 Universal + 2 NotSh = 13 rules
    // This brings total from 290 → 303 (84.9% coverage - 🎯 REACHED 85% MILESTONE! 🎯)
}

// === BATCH 16 TESTS (5 tests) ===

#[test]
fn test_batch16_posix_universal() {
    // SC2320, SC2322, SC2323, SC2324, SC2325 - POSIX positional parameters & arithmetic
    let posix_rules = vec![
        ("SC2320", "positional parameter $N quoting"),
        ("SC2322", "arithmetic argument count"),
        ("SC2323", "arithmetic = vs =="),
        ("SC2324", "${var:+value} conditional"),
        ("SC2325", "$var vs ${var} in arithmetic"),
    ];

    for (rule, description) in posix_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 16 POSIX rule {} ({}) should be Universal",
            rule,
            description
        );
    }
}

#[test]
fn test_batch16_bracket_specific_notsh() {
    // SC2321 - [[ ]] specific rule
    assert_eq!(
        get_rule_compatibility("SC2321"),
        Some(ShellCompatibility::NotSh),
        "Batch 16 rule SC2321 ([[ ]] logical AND) should be NotSh"
    );
}

#[test]
fn test_batch16_split_universal_vs_notsh() {
    // Batch 16: 5 Universal + 1 NotSh = 6 rules total
    let universal_rules = vec![
        "SC2320", // Positional parameter quoting
        "SC2322", // Arithmetic argument count
        "SC2323", // Arithmetic = vs ==
        "SC2324", // ${var:+value}
        "SC2325", // $var vs ${var}
    ];

    let notsh_rules = vec![
        "SC2321", // [[ ]] logical AND
    ];

    // Verify Universal rules
    for rule in &universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Rule {} should be Universal",
            rule
        );
    }

    // Verify NotSh rules
    for rule in &notsh_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "Rule {} should be NotSh",
            rule
        );
    }

    // Total: 5 Universal + 1 NotSh = 6 rules
    // This brings total from 303 → 309 (86.6% coverage - Approaching 90% milestone!)
}

#[test]
fn test_batch16_arithmetic_context() {
    // SC2322, SC2323, SC2325 - Arithmetic context rules
    let arithmetic_rules = vec!["SC2322", "SC2323", "SC2325"];

    for rule in arithmetic_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 16 arithmetic rule {} should be Universal",
            rule
        );
    }
}

#[test]
fn test_batch16_parameter_expansion() {
    // SC2320, SC2324 - Parameter expansion & positional parameters
    let param_rules = vec![
        ("SC2320", "positional parameter"),
        ("SC2324", "${var:+value}"),
    ];

    for (rule, description) in param_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 16 parameter rule {} ({}) should be Universal",
            rule,
            description
        );
    }
}

// === BATCH 17 TESTS (6 tests) ===

#[test]
fn test_batch17_backtick_command_substitution_universal() {
    // Batch 17: Backtick & command substitution (Universal - POSIX)
    let backtick_rules = vec![
        ("SC2036", "Quotes in backticks need escaping"),
        ("SC2037", "To assign command output, use var=$(cmd)"),
    ];

    for (rule, description) in backtick_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 17 backtick rule {} ({}) should be Universal (POSIX)",
            rule,
            description
        );

        // Should apply to ALL shells
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
fn test_batch17_function_parameter_usage_universal() {
    // Batch 17: Function & parameter usage (Universal - POSIX positional params, PATH, brace expansion)
    let function_universal_rules = vec![
        (
            "SC2119",
            "Use foo \"$@\" if function's $1 should mean script's $1",
        ),
        (
            "SC2123",
            "PATH is the shell search path. Assign to path instead",
        ),
        ("SC2125", "Brace expansion doesn't happen in [[ ]]"),
    ];

    for (rule, description) in function_universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 17 function rule {} ({}) should be Universal (POSIX)",
            rule,
            description
        );

        // Should apply to ALL shells
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
fn test_batch17_array_usage_notsh() {
    // Batch 17: Array usage (NotSh - bash/zsh/ksh specific)
    let array_rule = "SC2124";

    assert_eq!(
        get_rule_compatibility(array_rule),
        Some(ShellCompatibility::NotSh),
        "Batch 17 array rule {} should be NotSh (arrays are bash/zsh/ksh specific)",
        array_rule
    );

    // Should NOT apply to POSIX sh
    assert!(
        !should_apply_rule(array_rule, ShellType::Sh),
        "{} should not apply to sh",
        array_rule
    );

    // But SHOULD apply to bash/zsh/ksh
    assert!(
        should_apply_rule(array_rule, ShellType::Bash),
        "{} should apply to bash",
        array_rule
    );
    assert!(
        should_apply_rule(array_rule, ShellType::Zsh),
        "{} should apply to zsh",
        array_rule
    );
}

#[test]
fn test_batch17_parameter_expansion_universal() {
    // Batch 17: Parameter expansion & command optimization (Universal - POSIX)
    let param_expansion_rules = vec![
        (
            "SC2294",
            "Use arithmetic expansion ((...)) for simple assignments",
        ),
        (
            "SC2295",
            "Expansions inside ${} need to be quoted separately",
        ),
        ("SC2296", "Parameter expansions can't be nested"),
        ("SC2297", "Redirect before pipe"),
        ("SC2298", "Useless use of cat before pipe"),
        ("SC2299", "Parameter expansion only allows literals here"),
        ("SC2300", "Use ${var:?} for required environment variables"),
        ("SC2303", "Arithmetic base only allowed in assignments"),
        ("SC2304", "Command appears to be undefined"),
        ("SC2305", "Use ${var:=value} to assign default value"),
        (
            "SC2319",
            "This $? refers to a condition, not the previous command",
        ),
    ];

    for (rule, description) in param_expansion_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 17 parameter expansion rule {} ({}) should be Universal (POSIX)",
            rule,
            description
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
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }
}

#[test]
fn test_batch17_bash_specific_notsh() {
    // Batch 17: Bash-specific parameter expansion and array operations (NotSh)
    let bash_specific_rules = vec![
        (
            "SC2292",
            "Prefer ${var:0:1} over expr substr - bash substring expansion",
        ),
        ("SC2293", "Use += to append to arrays - bash array operator"),
        (
            "SC2301",
            "Use [[ -v array[0] ]] to check if array element exists - arrays + [[ -v ]]",
        ),
        (
            "SC2302",
            "Prefer ${var// /} over tr - bash ${var//} expansion",
        ),
    ];

    for (rule, description) in bash_specific_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "Batch 17 bash-specific rule {} ({}) should be NotSh",
            rule,
            description
        );

        // Should NOT apply to POSIX sh
        assert!(
            !should_apply_rule(rule, ShellType::Sh),
            "{} should not apply to sh",
            rule
        );

        // But SHOULD apply to bash/zsh
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
fn test_batch17_split_universal_vs_notsh() {
    // Batch 17: 15 Universal + 6 NotSh = 21 rules total (ALL REMAINING UNCLASSIFIED)
    let universal_rules = vec![
        // Backtick & command substitution (2)
        "SC2036", "SC2037", // Function & parameter usage (3)
        "SC2119", "SC2123", "SC2125",
        // Parameter expansion & command optimization (11)
        "SC2294", "SC2295", "SC2296", "SC2297", "SC2298", "SC2299", "SC2300", "SC2303", "SC2304",
        "SC2305", "SC2319",
    ];

    let notsh_rules = vec![
        "SC2124", // Array quoting
        "SC2292", // ${var:0:1} bash substring
        "SC2293", // Array += operator
        "SC2301", // [[ -v array[0] ]]
        "SC2302", // ${var//} bash expansion
    ];

    // Verify Universal rules
    for rule in &universal_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::Universal),
            "Batch 17 rule {} should be Universal",
            rule
        );
    }

    // Verify NotSh rules
    for rule in &notsh_rules {
        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "Batch 17 rule {} should be NotSh",
            rule
        );
    }

    // Verify counts
    assert_eq!(
        universal_rules.len(),
        16,
        "Batch 17 should have 16 Universal rules"
    );
    assert_eq!(notsh_rules.len(), 5, "Batch 17 should have 5 NotSh rules");

    // Total: 16 Universal + 5 NotSh = 21 rules
    // This brings total from 309 → 330 (92.4% coverage - 🎯🎯🎯 90% MILESTONE EXCEEDED! 🎯🎯🎯)
}
