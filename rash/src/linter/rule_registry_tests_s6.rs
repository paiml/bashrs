use super::*;
use crate::linter::shell_type::ShellType;

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
