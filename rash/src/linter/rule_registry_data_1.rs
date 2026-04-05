//! Rule registry entries — extracted from rule_registry.rs for file health.

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register(registry: &mut HashMap<&'static str, RuleMetadata>) {
    registry.insert(
        "SC1007",
        RuleMetadata {
            id: "SC1007",
            name: "Remove space after = in variable assignment",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1009",
        RuleMetadata {
            id: "SC1009",
            name: "Comment detected where command was expected",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1017",
        RuleMetadata {
            id: "SC1017",
            name: "Literal carriage return in source",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1018",
        RuleMetadata {
            id: "SC1018",
            name: "Unicode non-breaking space used",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1020",
        RuleMetadata {
            id: "SC1020",
            name: "Missing space before closing ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1035",
        RuleMetadata {
            id: "SC1035",
            name: "Missing space after keyword",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1068",
        RuleMetadata {
            id: "SC1068",
            name: "Don't put spaces around = in assignments",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1069",
        RuleMetadata {
            id: "SC1069",
            name: "Missing space before [ in test",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1082",
        RuleMetadata {
            id: "SC1082",
            name: "UTF-8 BOM detected",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1095",
        RuleMetadata {
            id: "SC1095",
            name: "Space between function name and () with function keyword",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC1099",
        RuleMetadata {
            id: "SC1099",
            name: "Missing space before # comment",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1100",
        RuleMetadata {
            id: "SC1100",
            name: "Unicode dash used instead of minus",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1101",
        RuleMetadata {
            id: "SC1101",
            name: "Trailing spaces after \\ line continuation",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1109",
        RuleMetadata {
            id: "SC1109",
            name: "Unquoted HTML entity in script",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC1129",
        RuleMetadata {
            id: "SC1129",
            name: "Missing space before ! in negation",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Security Rules (8 rules) - Universal
    registry.insert(
        "SEC001",
        RuleMetadata {
            id: "SEC001",
            name: "Command injection vulnerability",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC002",
        RuleMetadata {
            id: "SEC002",
            name: "Unsafe eval usage",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC003",
        RuleMetadata {
            id: "SEC003",
            name: "Unquoted variables (injection risk)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC004",
        RuleMetadata {
            id: "SEC004",
            name: "User input in commands",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC005",
        RuleMetadata {
            id: "SEC005",
            name: "Unsafe PATH modification",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC006",
        RuleMetadata {
            id: "SEC006",
            name: "Dangerous rm patterns",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC007",
        RuleMetadata {
            id: "SEC007",
            name: "Insecure temp file creation",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SEC008",
        RuleMetadata {
            id: "SEC008",
            name: "Source untrusted files",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Determinism Rules (3 rules) - Universal
    registry.insert(
        "DET001",
        RuleMetadata {
            id: "DET001",
            name: "$RANDOM usage (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "DET002",
        RuleMetadata {
            id: "DET002",
            name: "Timestamp usage (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "DET003",
        RuleMetadata {
            id: "DET003",
            name: "Wildcard ordering (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Idempotency Rules (3 rules) - Universal
    registry.insert(
        "IDEM001",
        RuleMetadata {
            id: "IDEM001",
            name: "mkdir without -p (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "IDEM002",
        RuleMetadata {
            id: "IDEM002",
            name: "rm without -f (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "IDEM003",
        RuleMetadata {
            id: "IDEM003",
            name: "ln without -sf (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Bash-only rules: Arrays, [[]], process substitution, etc.
    // These should not fire on POSIX sh or pure zsh scripts

    // SC2039: Features undefined in POSIX sh (bash/zsh specific)
    registry.insert(
        "SC2039",
        RuleMetadata {
            id: "SC2039",
            name: "Bash features undefined in POSIX sh",
            compatibility: ShellCompatibility::NotSh, // Works in bash/zsh/ksh but not sh
        },
    );

    // SC2198: Arrays are bash-specific
    registry.insert(
        "SC2198",
        RuleMetadata {
            id: "SC2198",
            name: "Array syntax (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // SC2199: Arrays are bash-specific
    registry.insert(
        "SC2199",
        RuleMetadata {
            id: "SC2199",
            name: "Array expansion (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // SC2200: Arrays are bash-specific
    registry.insert(
        "SC2200",
        RuleMetadata {
            id: "SC2200",
            name: "Array iteration (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // SC2201: Arrays are bash-specific
    registry.insert(
        "SC2201",
        RuleMetadata {
            id: "SC2201",
            name: "Array assignment (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // Process substitution rules (bash/zsh, not POSIX sh)
    registry.insert(
        "SC2002",
        RuleMetadata {
            id: "SC2002",
            name: "Useless cat (can use process substitution in bash/zsh)",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // === BATCH 2 CLASSIFICATIONS (25 rules) ===

    // [[ ]] test syntax rules (NotSh - bash/zsh/ksh only)
    registry.insert(
        "SC2108",
        RuleMetadata {
            id: "SC2108",
            name: "In [[ ]], use && instead of -a",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2109",
        RuleMetadata {
            id: "SC2109",
            name: "In [[ ]], use || instead of -o",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2110",
        RuleMetadata {
            id: "SC2110",
            name: "Don't mix && and || with -a and -o in [[ ]]",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // function keyword rules (NotSh - bash/ksh only, not POSIX)
    registry.insert(
        "SC2111",
        RuleMetadata {
            id: "SC2111",
            name: "'function' keyword not supported in sh",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2112",
        RuleMetadata {
            id: "SC2112",
            name: "'function' keyword is non-standard",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2113",
        RuleMetadata {
            id: "SC2113",
            name: "'function' keyword with () is redundant",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // Arithmetic expansion rules (Universal - $((...)) is POSIX)
    registry.insert(
        "SC2003",
        RuleMetadata {
            id: "SC2003",
            name: "expr is antiquated. Use $((...))",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2004",
        RuleMetadata {
            id: "SC2004",
            name: "$/${} unnecessary on arithmetic variables",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2079",
        RuleMetadata {
            id: "SC2079",
            name: "Decimals not supported in (( ))",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2080",
        RuleMetadata {
            id: "SC2080",
            name: "Leading zero interpreted as octal",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2084",
        RuleMetadata {
            id: "SC2084",
            name: "Arithmetic expansion as command",
            compatibility: ShellCompatibility::Universal,
        },
    );
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
