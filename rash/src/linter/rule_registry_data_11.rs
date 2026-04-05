//! Rule registry entries — extracted from rule_registry.rs for file health.

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // === BATCH 3 CLASSIFICATIONS (30 rules) ===

    // Loop and iteration safety (Universal)
    registry.insert(
        "SC2038",
        RuleMetadata {
            id: "SC2038",
            name: "Use -print0/-0 or find -exec instead of for loop over find",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2040",
        RuleMetadata {
            id: "SC2040",
            name: "Avoid passing -o to other commands (shell option confusion)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2041",
        RuleMetadata {
            id: "SC2041",
            name: "Use while read, not read in for loop",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2042",
        RuleMetadata {
            id: "SC2042",
            name: "Use printf instead of echo with backslash escapes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2043",
        RuleMetadata {
            id: "SC2043",
            name: "This loop will only run once (for x in y without wildcards)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Test operators and conditionals (Universal)
    registry.insert(
        "SC2044",
        RuleMetadata {
            id: "SC2044",
            name: "For loops over find: use find -exec or process substitution",
            compatibility: ShellCompatibility::NotSh, // process substitution suggestion
        },
    );
    registry.insert(
        "SC2045",
        RuleMetadata {
            id: "SC2045",
            name: "Iterating over ls output is fragile",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2046",
        RuleMetadata {
            id: "SC2046",
            name: "Quote to prevent word splitting (CRITICAL)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2047",
        RuleMetadata {
            id: "SC2047",
            name: "Quote variables in [ ] to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2048",
        RuleMetadata {
            id: "SC2048",
            name: "Use \"$@\" (with quotes) to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2049",
        RuleMetadata {
            id: "SC2049",
            name: "Use =~ for regex matching (not = in [ ])",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2050",
        RuleMetadata {
            id: "SC2050",
            name: "This expression is constant (forgot $ on variable?)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2051",
        RuleMetadata {
            id: "SC2051",
            name: "Bash doesn't expand variables in brace ranges {$a..$b}",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Quoting and glob safety (mostly Universal, one NotSh)
    registry.insert(
        "SC2052",
        RuleMetadata {
            id: "SC2052",
            name: "Use [[ ]] instead of [ ] for glob patterns",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh
        },
    );
    registry.insert(
        "SC2053",
        RuleMetadata {
            id: "SC2053",
            name: "Quote RHS of = in [ ] to prevent glob matching",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2054",
        RuleMetadata {
            id: "SC2054",
            name: "Comma is just literal in [[ ]]; use array or separate comparison",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2055",
        RuleMetadata {
            id: "SC2055",
            name: "Deprecated -a operator in test (use &&)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2056",
        RuleMetadata {
            id: "SC2056",
            name: "Deprecated -o operator in test (use ||)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2057",
        RuleMetadata {
            id: "SC2057",
            name: "Unknown binary operator (===, =!, <>)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2058",
        RuleMetadata {
            id: "SC2058",
            name: "Unknown unary operator in test",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Command safety and redirection (Universal - CRITICAL security rules)
    registry.insert(
        "SC2059",
        RuleMetadata {
            id: "SC2059",
            name: "Printf format string injection (CRITICAL security)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2060",
        RuleMetadata {
            id: "SC2060",
            name: "Unquoted tr parameters (glob expansion)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2061",
        RuleMetadata {
            id: "SC2061",
            name: "Quote parameters to tr to prevent globbing",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2062",
        RuleMetadata {
            id: "SC2062",
            name: "Grep pattern glob expansion prevention",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Trap and signal handling (Universal - P0 timing issue)
    registry.insert(
        "SC2063",
        RuleMetadata {
            id: "SC2063",
            name: "Grep regex vs literal string matching",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2064",
        RuleMetadata {
            id: "SC2064",
            name: "Trap command quoting (P0 - timing issue)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2065",
        RuleMetadata {
            id: "SC2065",
            name: "Shell redirection interpretation in strings",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2066",
        RuleMetadata {
            id: "SC2066",
            name: "Missing semicolon before done in for loop",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 4 CLASSIFICATIONS (30 rules) ===

    // Variable and parameter safety (Universal)
    registry.insert(
        "SC2067",
        RuleMetadata {
            id: "SC2067",
            name: "Missing $ on array lookup",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2068",
        RuleMetadata {
            id: "SC2068",
            name: "Quote $@ to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2069",
        RuleMetadata {
            id: "SC2069",
            name: "To redirect stdout+stderr, use &> or 2>&1, not 1>&2",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2070",
        RuleMetadata {
            id: "SC2070",
            name: "-n doesn't work with unquoted arguments",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2071",
        RuleMetadata {
            id: "SC2071",
            name: "Arithmetic operators don't work in [ ]. Use [[ ]] or (( ))",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2072",
        RuleMetadata {
            id: "SC2072",
            name: "Lexicographic comparison in [ ]. Use -lt/-gt for numbers",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2073",
        RuleMetadata {
            id: "SC2073",
            name: "Escape \\d in character class or use [[:digit:]]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2074",
        RuleMetadata {
            id: "SC2074",
            name: "Can't use =~ in [ ]. Use [[ ]] instead",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Quote and expansion safety (Universal)
    registry.insert(
        "SC2075",
        RuleMetadata {
            id: "SC2075",
            name: "Escaping quotes in single quotes doesn't work",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2076",
        RuleMetadata {
            id: "SC2076",
            name: "Don't quote RHS of =~ in [[ ]]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2077",
        RuleMetadata {
            id: "SC2077",
            name: "Quote regex argument to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2078",
        RuleMetadata {
            id: "SC2078",
            name: "This expression is constant (forgot $ on variable?)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2081",
        RuleMetadata {
            id: "SC2081",
            name: "Escape [ in globs or use [[ ]]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2082",
        RuleMetadata {
            id: "SC2082",
            name: "Variable indirection with $$ (use ${!var})",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2083",
        RuleMetadata {
            id: "SC2083",
            name: "Don't add spaces after shebang",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Command and redirection safety (Universal - CRITICAL)
    registry.insert(
        "SC2094",
        RuleMetadata {
            id: "SC2094",
            name: "Don't use same file for input and output (will truncate)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2095",
        RuleMetadata {
            id: "SC2095",
            name: "ssh -t/-T in loops may consume stdin",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2096",
        RuleMetadata {
            id: "SC2096",
            name: "Use #! shebang, not just # comment",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2097",
        RuleMetadata {
            id: "SC2097",
            name: "Assign and use variable separately",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2098",
        RuleMetadata {
            id: "SC2098",
            name: "Variable assignment vs redirection confusion",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2103",
        RuleMetadata {
            id: "SC2103",
            name: "cd without error check (use cd ... || exit)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Test and conditional safety (Universal)
    registry.insert(
        "SC2104",
        RuleMetadata {
            id: "SC2104",
            name: "In [[ ]], == is literal. Use = or [[ ]]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2105",
        RuleMetadata {
            id: "SC2105",
            name: "Break outside loop",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2107",
        RuleMetadata {
            id: "SC2107",
            name: "Instead of [ a -o b ], use [ a ] || [ b ]",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Function and scope safety (Universal - CRITICAL dangerous rm)
    registry.insert(
        "SC2114",
        RuleMetadata {
            id: "SC2114",
            name: "Dangerous rm -rf without validation ($VAR might be empty)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2115",
        RuleMetadata {
            id: "SC2115",
            name: "Use ${var:?} to ensure var is set before rm -rf",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2116",
        RuleMetadata {
            id: "SC2116",
            name: "Useless echo $(cmd) - just use cmd",
            compatibility: ShellCompatibility::Universal,
        },
    );
}
