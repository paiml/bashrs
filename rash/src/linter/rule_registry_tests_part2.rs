use super::*;
use crate::linter::shell_type::ShellType;

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


include!("rule_registry_tests_part2_incl2.rs");
