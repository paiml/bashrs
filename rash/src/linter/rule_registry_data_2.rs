//! Rule registry entries — extracted from rule_registry.rs for file health.

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // Bash-specific rules (NotSh)
    // SC2120: Not enabled yet (has false positives requiring AST parsing)
    // registry.insert("SC2120", RuleMetadata {
    //     id: "SC2120",
    //     name: "Function references $1 but none passed",
    //     compatibility: ShellCompatibility::NotSh, // Requires bash function analysis
    // });
    registry.insert(
        "SC2128",
        RuleMetadata {
            id: "SC2128",
            name: "Expanding array without index in bash",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // === BATCH 5 CLASSIFICATIONS (20 rules) ===

    // Batch 5: Command optimization and best practices (Universal)
    registry.insert(
        "SC2001",
        RuleMetadata {
            id: "SC2001",
            name: "Use ${var//pattern/replacement} instead of sed",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2005",
        RuleMetadata {
            id: "SC2005",
            name: "Useless echo instead of bare command",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2006",
        RuleMetadata {
            id: "SC2006",
            name: "Use $(...) instead of deprecated backticks",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2007",
        RuleMetadata {
            id: "SC2007",
            name: "Use $((..)) instead of deprecated expr",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 18: File handling and command best practices (Universal)
    registry.insert(
        "SC2008",
        RuleMetadata {
            id: "SC2008",
            name: "echo doesn't read from stdin",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2009",
        RuleMetadata {
            id: "SC2009",
            name: "Consider using pgrep instead of grepping ps output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2010",
        RuleMetadata {
            id: "SC2010",
            name: "Don't use ls | grep, use glob or find",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2011",
        RuleMetadata {
            id: "SC2011",
            name: "Use find -print0 | xargs -0 instead of ls | xargs",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2012",
        RuleMetadata {
            id: "SC2012",
            name: "Use find instead of ls for non-alphanumeric filenames",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2013",
        RuleMetadata {
            id: "SC2013",
            name: "To read lines, pipe/redirect to 'while read' loop",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2014",
        RuleMetadata {
            id: "SC2014",
            name: "Variables don't expand before brace expansion",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 5: Logic and quoting safety (Universal)
    registry.insert(
        "SC2015",
        RuleMetadata {
            id: "SC2015",
            name: "Note && and || precedence (use explicit grouping)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2016",
        RuleMetadata {
            id: "SC2016",
            name: "Expressions don't expand in single quotes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2017",
        RuleMetadata {
            id: "SC2017",
            name: "Increase precision by replacing bc/awk with arithmetic",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 5: tr character classes (Universal)
    registry.insert(
        "SC2018",
        RuleMetadata {
            id: "SC2018",
            name: "Use [:upper:] instead of [A-Z] for tr",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2019",
        RuleMetadata {
            id: "SC2019",
            name: "Use [:lower:] instead of [a-z] for tr",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2020",
        RuleMetadata {
            id: "SC2020",
            name: "tr replaces sets of chars, not strings",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2021",
        RuleMetadata {
            id: "SC2021",
            name: "Don't use [] around classes in tr",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 5: SSH and command safety (Universal)
    registry.insert(
        "SC2022",
        RuleMetadata {
            id: "SC2022",
            name: "Note: set -x only affects the current shell",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2023",
        RuleMetadata {
            id: "SC2023",
            name: "Brace expansion doesn't happen in [[ ]]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2024",
        RuleMetadata {
            id: "SC2024",
            name: "sudo only affects the command, not the redirection",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2025",
        RuleMetadata {
            id: "SC2025",
            name: "Note: set -e only affects the current shell",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2026",
        RuleMetadata {
            id: "SC2026",
            name: "Word splitting occurs in the variable",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 5: Quoting and echo safety (Universal)
    registry.insert(
        "SC2027",
        RuleMetadata {
            id: "SC2027",
            name: "Quote or escape $ in double quotes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2028",
        RuleMetadata {
            id: "SC2028",
            name: "echo may not expand \\n (use printf)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2029",
        RuleMetadata {
            id: "SC2029",
            name: "Variables must be local in remote SSH command",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 5: CRITICAL word splitting (Universal)
    registry.insert(
        "SC2086",
        RuleMetadata {
            id: "SC2086",
            name: "CRITICAL: Quote to prevent word splitting and globbing",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 6 CLASSIFICATIONS (20 rules) ===

    // Batch 6: Variable and function safety (Universal)
    registry.insert(
        "SC2033",
        RuleMetadata {
            id: "SC2033",
            name: "Shell functions can't be exported (use scripts or ENV)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2034",
        RuleMetadata {
            id: "SC2034",
            name: "Variable appears unused (verify with shellcheck)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2035",
        RuleMetadata {
            id: "SC2035",
            name: "Use ./*glob* or -- *glob* to match files starting with -",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 6: Command best practices (Universal)
    registry.insert(
        "SC2099",
        RuleMetadata {
            id: "SC2099",
            name: "Use $(...) instead of deprecated backticks",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2100",
        RuleMetadata {
            id: "SC2100",
            name: "Use $((..)) instead of deprecated expr",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2101",
        RuleMetadata {
            id: "SC2101",
            name: "Named POSIX class needs outer [] (e.g., [[:digit:]])",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2102",
        RuleMetadata {
            id: "SC2102",
            name: "Ranges only work with single chars (not regex +)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2106",
        RuleMetadata {
            id: "SC2106",
            name: "Consider using pgrep instead of ps | grep",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2117",
        RuleMetadata {
            id: "SC2117",
            name: "Unreachable code after exit or return",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 6: Ksh-specific (NotSh)
    registry.insert(
        "SC2118",
        RuleMetadata {
            id: "SC2118",
            name: "Ksh-specific set -A won't work in sh",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // Batch 6: Assignment and operator safety (Universal)
    registry.insert(
        "SC2121",
        RuleMetadata {
            id: "SC2121",
            name: "Don't use $ on left side of assignment",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2122",
        RuleMetadata {
            id: "SC2122",
            name: ">= not valid in [ ]. Use -ge for numeric comparison",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 6: Code quality and efficiency (Universal)
    registry.insert(
        "SC2126",
        RuleMetadata {
            id: "SC2126",
            name: "Use grep -c instead of grep | wc -l (efficiency)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2127",
        RuleMetadata {
            id: "SC2127",
            name: "Constant comparison in [ ] (always true/false)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2129",
        RuleMetadata {
            id: "SC2129",
            name: "Use >> instead of repeated > redirects to same file",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2130",
        RuleMetadata {
            id: "SC2130",
            name: "-e flag usage clarification (valid file test)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2131",
        RuleMetadata {
            id: "SC2131",
            name: "Backslashes in single quotes are literal (no escaping)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2132",
        RuleMetadata {
            id: "SC2132",
            name: "Readonly variable used in for loop (will fail)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 6: Control flow safety (Universal)
    registry.insert(
        "SC2135",
        RuleMetadata {
            id: "SC2135",
            name: "Unexpected 'then' after condition (missing semicolon or wrong keyword)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2136",
        RuleMetadata {
            id: "SC2136",
            name: "Unexpected 'do' in 'if' statement (should be 'then')",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 7 CLASSIFICATIONS (20 rules) ===

    // Batch 7: Alias and function context safety (Universal)
    registry.insert(
        "SC2138",
        RuleMetadata {
            id: "SC2138",
            name: "Function defined in wrong context (if/loop) or reserved name",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2139",
        RuleMetadata {
            id: "SC2139",
            name: "Alias variable expands at definition time (not invocation)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2140",
        RuleMetadata {
            id: "SC2140",
            name: "Malformed quote concatenation (unquoted words between quotes)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2141",
        RuleMetadata {
            id: "SC2141",
            name: "Command receives stdin but ignores it (find, ls, echo, sudo)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2142",
        RuleMetadata {
            id: "SC2142",
            name: "Aliases can't use positional parameters (use functions instead)",
            compatibility: ShellCompatibility::Universal,
        },
    );
}
