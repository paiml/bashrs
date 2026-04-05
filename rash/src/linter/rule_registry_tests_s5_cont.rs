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
