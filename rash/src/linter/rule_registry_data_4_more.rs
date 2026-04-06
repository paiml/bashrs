//! Rule registry data — continuation of rule_registry_data_4.rs

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register_more(registry: &mut HashMap<&'static str, RuleMetadata>) {
    registry.insert(
        "MAKE007",
        RuleMetadata {
            id: "MAKE007",
            name: "Silent recipe errors (missing @ prefix)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE008",
        RuleMetadata {
            id: "MAKE008",
            name: "Tab vs spaces in recipes (CRITICAL)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE009",
        RuleMetadata {
            id: "MAKE009",
            name: "Hardcoded paths (non-portable)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE010",
        RuleMetadata {
            id: "MAKE010",
            name: "Missing error handling (|| exit 1)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE011",
        RuleMetadata {
            id: "MAKE011",
            name: "Dangerous pattern rules",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE012",
        RuleMetadata {
            id: "MAKE012",
            name: "Recursive make considered harmful",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE013",
        RuleMetadata {
            id: "MAKE013",
            name: "Missing .SUFFIXES (performance issue)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE014",
        RuleMetadata {
            id: "MAKE014",
            name: "Inefficient shell invocation",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE015",
        RuleMetadata {
            id: "MAKE015",
            name: "Missing .DELETE_ON_ERROR",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE016",
        RuleMetadata {
            id: "MAKE016",
            name: "Unquoted variable in prerequisites",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE017",
        RuleMetadata {
            id: "MAKE017",
            name: "Missing .ONESHELL",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE018",
        RuleMetadata {
            id: "MAKE018",
            name: "Parallel-unsafe targets (race conditions)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE019",
        RuleMetadata {
            id: "MAKE019",
            name: "Environment variable pollution",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE020",
        RuleMetadata {
            id: "MAKE020",
            name: "Missing include guard",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Most other SC2xxx rules are Universal (quoting, syntax, etc.)
    // They represent bugs or issues that apply regardless of shell
    // Examples: SC2086 (quote variables), etc.
    // These will be added as "Universal" as we classify them

    // Performance rules (PERF001-PERF005) - Universal
    registry.insert(
        "PERF001",
        RuleMetadata {
            id: "PERF001",
            name: "Useless use of cat",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "PERF002",
        RuleMetadata {
            id: "PERF002",
            name: "Command substitution inside loop",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "PERF003",
        RuleMetadata {
            id: "PERF003",
            name: "Useless use of echo",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "PERF004",
        RuleMetadata {
            id: "PERF004",
            name: "find -exec with \\; instead of +",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "PERF005",
        RuleMetadata {
            id: "PERF005",
            name: "/bin/echo instead of builtin echo",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Portability rules (PORT001-PORT005) - POSIX-only (fires on #!/bin/sh)
    registry.insert(
        "PORT001",
        RuleMetadata {
            id: "PORT001",
            name: "Array syntax in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        },
    );
    registry.insert(
        "PORT002",
        RuleMetadata {
            id: "PORT002",
            name: "local keyword in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        },
    );
    registry.insert(
        "PORT003",
        RuleMetadata {
            id: "PORT003",
            name: "[[ ]] test in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        },
    );
    registry.insert(
        "PORT004",
        RuleMetadata {
            id: "PORT004",
            name: "Process substitution in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        },
    );
    registry.insert(
        "PORT005",
        RuleMetadata {
            id: "PORT005",
            name: "source instead of . in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        },
    );

    // Reliability rules (REL001-REL005) - Universal
    registry.insert(
        "REL001",
        RuleMetadata {
            id: "REL001",
            name: "Destructive command without error check",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "REL002",
        RuleMetadata {
            id: "REL002",
            name: "mktemp without trap cleanup",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "REL003",
        RuleMetadata {
            id: "REL003",
            name: "read without timeout",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "REL004",
        RuleMetadata {
            id: "REL004",
            name: "TOCTOU race condition",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "REL005",
        RuleMetadata {
            id: "REL005",
            name: "Predictable temp file name",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // SC1xxx rules (source code / portability issues)
    registry.insert(
        "SC1037",
        RuleMetadata {
            id: "SC1037",
            name: "Braces required for positional parameters beyond $9",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1076",
        RuleMetadata {
            id: "SC1076",
            name: "Deprecated $[...] arithmetic syntax",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1087",
        RuleMetadata {
            id: "SC1087",
            name: "Braces required for array access",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1105",
        RuleMetadata {
            id: "SC1105",
            name: "Space between $ and ( breaks command substitution",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1106",
        RuleMetadata {
            id: "SC1106",
            name: "Use -lt/-gt not </>  in single brackets",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1131",
        RuleMetadata {
            id: "SC1131",
            name: "Use elif instead of else followed by if",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1139",
        RuleMetadata {
            id: "SC1139",
            name: "Use || instead of -o in [[ ]]",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC1140",
        RuleMetadata {
            id: "SC1140",
            name: "Unexpected extra token after ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
}
