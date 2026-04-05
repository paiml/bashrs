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
