use super::*;
use crate::linter::shell_type::ShellType;

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
