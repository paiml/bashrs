//! Rule registry entries — extracted from rule_registry.rs for file health.

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register(registry: &mut HashMap<&'static str, RuleMetadata>) {
    register_batch_a(registry);
    register_batch_b(registry);
    register_batch_c(registry);
    super::rule_registry_data_3_more::register_more(registry);
}

fn register_batch_a(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // === BATCH 10 CLASSIFICATIONS (20 rules) ===

    // Batch 10: Command structure & ordering (Universal)
    registry.insert(
        "SC2202",
        RuleMetadata {
            id: "SC2202",
            name: "Order sensitivity (e.g., redirects)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2203",
        RuleMetadata {
            id: "SC2203",
            name: "Variable assignment order matters",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2204",
        RuleMetadata {
            id: "SC2204",
            name: "Exit traps must come before commands",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2205",
        RuleMetadata {
            id: "SC2205",
            name: "Command ordering with pipes",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 10: Array operations (NotSh - bash/zsh/ksh only)
    registry.insert(
        "SC2206",
        RuleMetadata {
            id: "SC2206",
            name: "Quote to prevent word splitting/globbing in arrays",
            compatibility: ShellCompatibility::NotSh,
        },
    );
    registry.insert(
        "SC2207",
        RuleMetadata {
            id: "SC2207",
            name: "Prefer mapfile or read -a to split command output",
            compatibility: ShellCompatibility::NotSh,
        },
    );

    // Batch 10: Command structure & find usage (Universal)
    registry.insert(
        "SC2208",
        RuleMetadata {
            id: "SC2208",
            name: "Command grouping issues",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2209",
        RuleMetadata {
            id: "SC2209",
            name: "Use single quotes for literal strings in find",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 10: Arithmetic operations (Universal)
    registry.insert(
        "SC2210",
        RuleMetadata {
            id: "SC2210",
            name: "Don't use arithmetic shortcuts like x=++y",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2211",
        RuleMetadata {
            id: "SC2211",
            name: "Arithmetic on variable without $(())",
            compatibility: ShellCompatibility::Universal,
        },
    );
}

fn register_batch_b(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // Batch 10: Control flow & test operators (Universal)
    registry.insert(
        "SC2212",
        RuleMetadata {
            id: "SC2212",
            name: "Use [ p ] || [ q ] instead of [ p -o q ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2213",
        RuleMetadata {
            id: "SC2213",
            name: "getopts requires argument variable",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2214",
        RuleMetadata {
            id: "SC2214",
            name: "Arithmetic comparison outside test",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2215",
        RuleMetadata {
            id: "SC2215",
            name: "Expression precedence issues",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2216",
        RuleMetadata {
            id: "SC2216",
            name: "Piping find to shell with ; instead of +",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2217",
        RuleMetadata {
            id: "SC2217",
            name: "Useless cat with find",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2218",
        RuleMetadata {
            id: "SC2218",
            name: "Useless return in command substitution",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2219",
        RuleMetadata {
            id: "SC2219",
            name: "Instead of let expr, use (( expr ))",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 10: Arithmetic syntax (Universal)
    registry.insert(
        "SC2220",
        RuleMetadata {
            id: "SC2220",
            name: "Invalid arithmetic expression",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2221",
        RuleMetadata {
            id: "SC2221",
            name: "Arithmetic syntax errors",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 11 CLASSIFICATIONS (20 rules) ===

    // Batch 11: Case statement syntax (Universal)
    registry.insert(
        "SC2222",
        RuleMetadata {
            id: "SC2222",
            name: "Lexical error in case statement syntax",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2223",
        RuleMetadata {
            id: "SC2223",
            name: "This default case is unreachable (previous pattern catches all)",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 11: Control flow & test operators (Universal)
    registry.insert(
        "SC2224",
        RuleMetadata {
            id: "SC2224",
            name: "Quote the word or use a glob",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2225",
        RuleMetadata {
            id: "SC2225",
            name: "Use : or true instead of /bin/true",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2226",
        RuleMetadata {
            id: "SC2226",
            name: "This expression is constant",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2227",
        RuleMetadata {
            id: "SC2227",
            name: "Redirection applies to the echo, not the assignment",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2228",
        RuleMetadata {
            id: "SC2228",
            name: "Declare -x is equivalent to export",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2229",
        RuleMetadata {
            id: "SC2229",
            name: "This does not read 'foo'. Remove $/${} for that",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 11: Command existence & portability (Universal)
    registry.insert(
        "SC2230",
        RuleMetadata {
            id: "SC2230",
            name: "which is non-standard, use command -v instead",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2231",
        RuleMetadata {
            id: "SC2231",
            name: "Quote expansions in this for loop glob to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2232",
        RuleMetadata {
            id: "SC2232",
            name: "Can't use sudo with builtins like cd",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2233",
        RuleMetadata {
            id: "SC2233",
            name: "Remove superfluous (..) around condition",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2234",
        RuleMetadata {
            id: "SC2234",
            name: "Remove superfluous () around here document",
            compatibility: ShellCompatibility::Universal,
        },
    );
}

fn register_batch_c(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // Batch 11: Quoting & expansion safety (Universal)
    registry.insert(
        "SC2235",
        RuleMetadata {
            id: "SC2235",
            name: "Quote arguments to unalias to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2236",
        RuleMetadata {
            id: "SC2236",
            name: "Use -n instead of ! -z",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2237",
        RuleMetadata {
            id: "SC2237",
            name: "Use [ ] instead of [[ ]] (for sh compatibility)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2238",
        RuleMetadata {
            id: "SC2238",
            name: "Prefer ${} over backticks (readability + nesting)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2239",
        RuleMetadata {
            id: "SC2239",
            name: "Ensure consistent quoting for redirects",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2240",
        RuleMetadata {
            id: "SC2240",
            name: "The dot command does not support arguments in sh",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2241",
        RuleMetadata {
            id: "SC2241",
            name: "Exit code is always overridden by following command",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 12 CLASSIFICATIONS (20 rules) ===

    // Batch 12: Control flow & case statements (Universal)
    registry.insert(
        "SC2242",
        RuleMetadata {
            id: "SC2242",
            name: "Can only break/continue from loops, not case",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2243",
        RuleMetadata {
            id: "SC2243",
            name: "Prefer explicit -n to check for output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2244",
        RuleMetadata {
            id: "SC2244",
            name: "Prefer explicit -n to check for output (variation)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2245",
        RuleMetadata {
            id: "SC2245",
            name: "-d test on assignment result",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2246",
        RuleMetadata {
            id: "SC2246",
            name: "This shebang was unrecognized",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 12: Test operators & efficiency (Universal)
    registry.insert(
        "SC2247",
        RuleMetadata {
            id: "SC2247",
            name: "Prefer [ p ] && [ q ] over [ p -a q ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2248",
        RuleMetadata {
            id: "SC2248",
            name: "Prefer explicit -n to check for output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2249",
        RuleMetadata {
            id: "SC2249",
            name: "Consider adding default case in case statement",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2250",
        RuleMetadata {
            id: "SC2250",
            name: "Prefer $((..)) over let for arithmetic",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2251",
        RuleMetadata {
            id: "SC2251",
            name: "This loop will only ever run once for constant",
            compatibility: ShellCompatibility::Universal,
        },
    );
}
