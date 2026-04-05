use super::*;
use crate::linter::shell_type::ShellType;

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
    }
}
