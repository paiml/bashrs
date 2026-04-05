//! Rule registry data — continuation of rule_registry_data_3.rs

use std::collections::HashMap;
use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;

pub(crate) fn register_more(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // Batch 12: Loop & case patterns (Universal)
    registry.insert(
        "SC2252",
        RuleMetadata {
            id: "SC2252",
            name: "You probably wanted && here, not a second [",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2253",
        RuleMetadata {
            id: "SC2253",
            name: "Quote the RHS of = in [[ ]] to prevent glob matching",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2254",
        RuleMetadata {
            id: "SC2254",
            name: "Quote expansions in case patterns to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2255",
        RuleMetadata {
            id: "SC2255",
            name: "This [ .. ] is true whenever str is non-empty",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2256",
        RuleMetadata {
            id: "SC2256",
            name: "Prefer -n/-z over comparison with empty string",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 12: Command safety & quoting (Universal)
    registry.insert(
        "SC2257",
        RuleMetadata {
            id: "SC2257",
            name: "Prefer explicit -n to check non-empty string",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2258",
        RuleMetadata {
            id: "SC2258",
            name: "Prefer explicit -n to check output",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2259",
        RuleMetadata {
            id: "SC2259",
            name: "This assumes $RANDOM is always positive",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2260",
        RuleMetadata {
            id: "SC2260",
            name: "Fix $((..)) arithmetic so [[ ]] can interpret it",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2261",
        RuleMetadata {
            id: "SC2261",
            name: "Unquoted operand will be glob expanded",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 13 CLASSIFICATIONS (20 rules) ===

    // Batch 13: Quoting & parameter safety (Universal)
    registry.insert(
        "SC2262",
        RuleMetadata {
            id: "SC2262",
            name: "This command may need quoting (context sensitive)",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2263",
        RuleMetadata {
            id: "SC2263",
            name: "Use cd ... || exit to handle cd failures",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2264",
        RuleMetadata {
            id: "SC2264",
            name: "Prefer [ p ] && [ q ] over [ p -a q ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2265",
        RuleMetadata {
            id: "SC2265",
            name: "Use ${var:?} to ensure this never expands to /* /",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2266",
        RuleMetadata {
            id: "SC2266",
            name: "Prefer [ p ] || [ q ] over [ p -o q ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2267",
        RuleMetadata {
            id: "SC2267",
            name: "Use ${var:?} to ensure variable is set",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2268",
        RuleMetadata {
            id: "SC2268",
            name: "Avoid x-prefix in comparisons",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2269",
        RuleMetadata {
            id: "SC2269",
            name: "This regex should be put in a variable",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 13: Argument parsing & best practices (Universal)
    registry.insert(
        "SC2270",
        RuleMetadata {
            id: "SC2270",
            name: "Prefer getopts over manual argument parsing",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2271",
        RuleMetadata {
            id: "SC2271",
            name: "Prefer printf over echo for non-trivial formatting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2272",
        RuleMetadata {
            id: "SC2272",
            name: "This is a constant, not a variable",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2273",
        RuleMetadata {
            id: "SC2273",
            name: "Use ${var:?} if this should never be empty",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2274",
        RuleMetadata {
            id: "SC2274",
            name: "Quote the RHS of = in [ ] to prevent globbing",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 13: Word splitting & expansion safety (Universal)
    registry.insert(
        "SC2275",
        RuleMetadata {
            id: "SC2275",
            name: "Use ${var} to avoid field splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2276",
        RuleMetadata {
            id: "SC2276",
            name: "Prefer explicit -n to check non-empty",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2277",
        RuleMetadata {
            id: "SC2277",
            name: "Use || instead of -o for test operators",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2278",
        RuleMetadata {
            id: "SC2278",
            name: "Use [[ ]] instead of deprecated syntax",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2279",
        RuleMetadata {
            id: "SC2279",
            name: "Use [[ < instead of [ <",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2280",
        RuleMetadata {
            id: "SC2280",
            name: "Remove redundant (..) or use 'if .. then'",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2281",
        RuleMetadata {
            id: "SC2281",
            name: "Don't use $@ in double quotes, it breaks word splitting",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // === BATCH 14 CLASSIFICATIONS (10 rules) ===

    // Batch 14: Parameter expansion & safety (Universal)
    registry.insert(
        "SC2282",
        RuleMetadata {
            id: "SC2282",
            name: "Use ${var:?} to require variables to be set",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2283",
        RuleMetadata {
            id: "SC2283",
            name: "Remove extra spaces after ! in test expressions",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2284",
        RuleMetadata {
            id: "SC2284",
            name: "Use ${var:+value} for conditional value assignment",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2285",
        RuleMetadata {
            id: "SC2285",
            name: "Remove $ from variables in arithmetic contexts",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 14: Bash-specific features (NotSh - bash/zsh/ksh only)
    registry.insert(
        "SC2286",
        RuleMetadata {
            id: "SC2286",
            name: "Prefer mapfile/readarray over read loops",
            compatibility: ShellCompatibility::NotSh, // mapfile/readarray are bash 4+ builtins
        },
    );
    registry.insert(
        "SC2287",
        RuleMetadata {
            id: "SC2287",
            name: "Use [[ -v var ]] to check if variable is set",
            compatibility: ShellCompatibility::NotSh, // [[ -v ]] is bash/zsh/ksh specific
        },
    );

    // Batch 14: Best practices & style (Universal)
    registry.insert(
        "SC2288",
        RuleMetadata {
            id: "SC2288",
            name: "Use true/false directly instead of [ 1 = 1 ]",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "SC2289",
        RuleMetadata {
            id: "SC2289",
            name: "Use ${#var} instead of expr length for string length",
            compatibility: ShellCompatibility::Universal,
        },
    );

    // Batch 14: Bash arrays (NotSh - bash/zsh/ksh only)
    registry.insert(
        "SC2290",
        RuleMetadata {
            id: "SC2290",
            name: "Remove $ from array index: ${array[i]} not ${array[$i]}",
            compatibility: ShellCompatibility::NotSh, // Arrays are bash-specific
        },
    );
    registry.insert(
        "SC2291",
        RuleMetadata {
            id: "SC2291",
            name: "Use [[ ! -v var ]] to check if variable is unset",
            compatibility: ShellCompatibility::NotSh, // [[ ! -v ]] is bash/zsh/ksh specific
        },
    );
}
