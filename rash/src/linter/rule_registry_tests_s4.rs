use super::*;
use crate::linter::shell_type::ShellType;

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
