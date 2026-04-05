//! Rule registry entries — extracted from rule_registry.rs for file health.

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // Batch 7: Find and glob efficiency (Universal)
    registry.insert(
        "SC2143",
        RuleMetadata {
            id: "SC2143",
            name: "Use grep -q for efficiency (exits on first match)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2144",
        RuleMetadata {
            id: "SC2144",
            name: "-e test on glob that never matches (glob safety)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2145",
        RuleMetadata {
            id: "SC2145",
            name: "Argument mixin in arrays ($@ or $* unquoted in quotes)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2146",
        RuleMetadata {
            id: "SC2146",
            name: "find -o action grouping needs parentheses",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2147",
        RuleMetadata {
            id: "SC2147",
            name: "Literal tilde in PATH doesn't expand (use $HOME)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2148",
        RuleMetadata {
            id: "SC2148",
            name: "Add shebang to indicate interpreter (portability)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2149",
        RuleMetadata {
            id: "SC2149",
            name: "Remove quotes from unset variable names",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2150",
        RuleMetadata {
            id: "SC2150",
            name: "Use find -exec + instead of \\; for batch processing (efficiency)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 7: Return/exit code and control flow safety (Universal)
    registry.insert(
        "SC2151",
        RuleMetadata {
            id: "SC2151",
            name: "Return code should be 0-255 (POSIX requirement)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2152",
        RuleMetadata {
            id: "SC2152",
            name: "Exit code should be 0-255 (POSIX requirement)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2153",
        RuleMetadata {
            id: "SC2153",
            name: "Possible misspelling: var=$VAR1, but only $VAR2 is defined",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2154",
        RuleMetadata {
            id: "SC2154",
            name: "Variable is referenced but not assigned (may be external)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2155",
        RuleMetadata {
            id: "SC2155",
            name: "Declare and assign separately to preserve exit code",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2156",
        RuleMetadata {
            id: "SC2156",
            name: "Injected filenames can cause command injection ($() in filenames)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2157",
        RuleMetadata {
            id: "SC2157",
            name: "Argument to [ -z/-n ] is always false due to literal strings",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 8 CLASSIFICATIONS (20 rules) ===

    // Batch 8: Exit code & bracket safety (Universal)
    registry.insert(
        "SC2158",
        RuleMetadata {
            id: "SC2158",
            name: "[ true ] evaluates as literal '[', not test command",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2159",
        RuleMetadata {
            id: "SC2159",
            name: "[ [ with space creates syntax error (double bracket mistake)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2160",
        RuleMetadata {
            id: "SC2160",
            name: "Instead of 'if var; then', use 'if [ -n \"$var\" ]; then'",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2161",
        RuleMetadata {
            id: "SC2161",
            name: "Provide explicit error handling for cd commands",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 8: read command safety (Universal)
    registry.insert(
        "SC2162",
        RuleMetadata {
            id: "SC2162",
            name: "read without -r will mangle backslashes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2163",
        RuleMetadata {
            id: "SC2163",
            name: "export command with array syntax (non-portable)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2164",
        RuleMetadata {
            id: "SC2164",
            name: "cd without error check (use ||, &&, or if)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 8: Trap & signal handling (Mixed)
    registry.insert(
        "SC2165",
        RuleMetadata {
            id: "SC2165",
            name: "Subshells don't inherit traps - use functions instead",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2166",
        RuleMetadata {
            id: "SC2166",
            name: "Prefer [ p ] && [ q ] over [ p -a q ] (POSIX portability)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2167",
        RuleMetadata {
            id: "SC2167",
            name: "Trap handler doesn't propagate to subshells",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2168",
        RuleMetadata {
            id: "SC2168",
            name: "'local' keyword is only valid in functions",
            compatibility: ShellCompatibility::NotSh, // local is bash/ksh/zsh specific
        },
    );

    // Batch 8: Test operators & syntax (Universal)
    registry.insert(
        "SC2169",
        RuleMetadata {
            id: "SC2169",
            name: "In dash/sh, -eq is undefined for strings (use = instead)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2170",
        RuleMetadata {
            id: "SC2170",
            name: "Numerical -eq comparison on non-numeric strings",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2171",
        RuleMetadata {
            id: "SC2171",
            name: "Found trailing ] on the line (syntax error)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2172",
        RuleMetadata {
            id: "SC2172",
            name: "Trapping signals by number is deprecated (use names)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2173",
        RuleMetadata {
            id: "SC2173",
            name: "Trying to trap untrappable signals (SIGKILL, SIGSTOP)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 8: Security & best practices (Universal)
    registry.insert(
        "SC2174",
        RuleMetadata {
            id: "SC2174",
            name: "mkdir -p and chmod in one shot creates security race",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2175",
        RuleMetadata {
            id: "SC2175",
            name: "Quote this to prevent word splitting (placeholder check)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2176",
        RuleMetadata {
            id: "SC2176",
            name: "'time' keyword affects full pipeline (not just first command)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2177",
        RuleMetadata {
            id: "SC2177",
            name: "'time' only times the first command (placeholder check)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 9 CLASSIFICATIONS (20 rules) ===

    // Batch 9: Array operations (NotSh - bash/zsh/ksh only)
    registry.insert(
        "SC2178",
        RuleMetadata {
            id: "SC2178",
            name: "Variable was used as an array but is now assigned a string",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2179",
        RuleMetadata {
            id: "SC2179",
            name: "Use array+=(\"item\") to append items to an array",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2180",
        RuleMetadata {
            id: "SC2180",
            name: "Trying to use an array as a scalar (missing index)",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // Batch 9: Exit code and printf patterns (Universal)
    registry.insert(
        "SC2181",
        RuleMetadata {
            id: "SC2181",
            name: "Check exit code directly with if mycmd, not if [ $? -eq 0 ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2182",
        RuleMetadata {
            id: "SC2182",
            name: "This printf format string has no variables",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 9: Assignment and expansion safety (Universal)
    registry.insert(
        "SC2183",
        RuleMetadata {
            id: "SC2183",
            name: "This value looks like a variable but won't be expanded",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2184",
        RuleMetadata {
            id: "SC2184",
            name: "Quote arguments to cd to avoid glob expansion",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2185",
        RuleMetadata {
            id: "SC2185",
            name: "Some SSH commands don't pass on their exit codes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2186",
        RuleMetadata {
            id: "SC2186",
            name: "mktemp argument may be evaluated as template",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 9: Shell directives and redirection (Mixed)
    registry.insert(
        "SC2187",
        RuleMetadata {
            id: "SC2187",
            name: "Ash scripts will be checked as Dash (use #!/bin/dash)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2188",
        RuleMetadata {
            id: "SC2188",
            name: "This redirection doesn't have a command",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2189",
        RuleMetadata {
            id: "SC2189",
            name: "Zsh directive will be checked as sh (use #!/bin/zsh)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 9: Associative arrays (NotSh - bash 4+ / zsh)
    registry.insert(
        "SC2190",
        RuleMetadata {
            id: "SC2190",
            name: "Elements in associative arrays need index",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2191",
        RuleMetadata {
            id: "SC2191",
            name: "Trying to use an associative array without index",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // Batch 9: Command composition and regex (Universal)
    registry.insert(
        "SC2192",
        RuleMetadata {
            id: "SC2192",
            name: "Piping to sudo: only last command will run as root",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2193",
        RuleMetadata {
            id: "SC2193",
            name: "RHS of regexes must be unquoted in [[]]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2194",
        RuleMetadata {
            id: "SC2194",
            name: "This word is constant - did you forget $ or ()?",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2195",
        RuleMetadata {
            id: "SC2195",
            name: "Use single quotes to pass literal regex to grep",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2196",
        RuleMetadata {
            id: "SC2196",
            name: "Prefer explicit -n to check output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2197",
        RuleMetadata {
            id: "SC2197",
            name: "Don't compare globs in []; use [[ ]] or case",
            compatibility: ShellCompatibility::Universal,
        },
    );
}
