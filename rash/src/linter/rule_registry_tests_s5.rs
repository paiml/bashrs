use super::*;
use crate::linter::shell_type::ShellType;

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

include!("rule_registry_tests_s5_incl2.rs");
