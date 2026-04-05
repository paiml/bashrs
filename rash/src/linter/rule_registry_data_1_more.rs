//! Rule registry data — continuation of rule_registry_data_1.rs

use std::collections::HashMap;
use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;

pub(crate) fn register_more(registry: &mut HashMap<&'static str, RuleMetadata>) {
    registry.insert(
        "SC2085",
        RuleMetadata {
            id: "SC2085",
            name: "Local variable with arithmetic",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2133",
        RuleMetadata {
            id: "SC2133",
            name: "Unexpected tokens in arithmetic expansion",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2134",
        RuleMetadata {
            id: "SC2134",
            name: "Use (( )) for numeric tests",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2137",
        RuleMetadata {
            id: "SC2137",
            name: "Unnecessary braces in arithmetic",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Quoting and subshell rules (Universal - POSIX concepts)
    registry.insert(
        "SC2030",
        RuleMetadata {
            id: "SC2030",
            name: "Variable modified in subshell",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2031",
        RuleMetadata {
            id: "SC2031",
            name: "Variable was modified in subshell",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2032",
        RuleMetadata {
            id: "SC2032",
            name: "Variable in script with shebang",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2087",
        RuleMetadata {
            id: "SC2087",
            name: "Quote variables in sh -c",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2088",
        RuleMetadata {
            id: "SC2088",
            name: "Tilde expansion in quotes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2089",
        RuleMetadata {
            id: "SC2089",
            name: "Quotes in assignment treated literally",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2090",
        RuleMetadata {
            id: "SC2090",
            name: "Quotes in expansion treated literally",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2091",
        RuleMetadata {
            id: "SC2091",
            name: "Remove $() to avoid executing output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2092",
        RuleMetadata {
            id: "SC2092",
            name: "Remove backticks to avoid executing output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2093",
        RuleMetadata {
            id: "SC2093",
            name: "Remove exec if script should continue",
            compatibility: ShellCompatibility::Universal,
        },
    );
}
