//! Rule registry entries — extracted from rule_registry.rs for file health.

use super::rule_registry::RuleMetadata;
use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

pub(crate) fn register(registry: &mut HashMap<&'static str, RuleMetadata>) {
    // === BATCH 15 CLASSIFICATIONS (13 rules) ===

    // Batch 15: Bash-specific parameter expansion (NotSh)
    registry.insert(
        "SC2306",
        RuleMetadata {
            id: "SC2306",
            name: "Use ${var//old/new} instead of sed for simple substitutions",
            compatibility: ShellCompatibility::NotSh, // ${var//} is bash parameter expansion
        },
    );

    // Batch 15: POSIX parameter expansion (Universal)
    registry.insert(
        "SC2307",
        RuleMetadata {
            id: "SC2307",
            name: "Use ${var#prefix} to remove prefix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        },
    );
    registry.insert(
        "SC2308",
        RuleMetadata {
            id: "SC2308",
            name: "Use ${var%suffix} to remove suffix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        },
    );
    registry.insert(
        "SC2309",
        RuleMetadata {
            id: "SC2309",
            name: "Use ${var##prefix} to remove longest prefix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        },
    );
    registry.insert(
        "SC2311",
        RuleMetadata {
            id: "SC2311",
            name: "Use ${var%%suffix} to remove longest suffix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        },
    );
    registry.insert(
        "SC2315",
        RuleMetadata {
            id: "SC2315",
            name: "Use ${var:+replacement} for conditional replacement",
            compatibility: ShellCompatibility::Universal, // POSIX ${var:+value}
        },
    );

    // Batch 15: set -e behavior & control flow (Universal)
    registry.insert(
        "SC2310",
        RuleMetadata {
            id: "SC2310",
            name: "Function in condition - set -e doesn't apply",
            compatibility: ShellCompatibility::Universal, // POSIX set -e behavior
        },
    );
    registry.insert(
        "SC2316",
        RuleMetadata {
            id: "SC2316",
            name: "Command group and precedence issues",
            compatibility: ShellCompatibility::Universal, // POSIX control flow
        },
    );
    registry.insert(
        "SC2317",
        RuleMetadata {
            id: "SC2317",
            name: "Unreachable code detection",
            compatibility: ShellCompatibility::Universal, // Universal logic
        },
    );

    // Batch 15: Deprecated syntax warnings (Universal)
    registry.insert(
        "SC2312",
        RuleMetadata {
            id: "SC2312",
            name: "Deprecated local -x syntax",
            compatibility: ShellCompatibility::Universal, // Universal portability warning
        },
    );
    registry.insert(
        "SC2313",
        RuleMetadata {
            id: "SC2313",
            name: "Use $(( )) for arithmetic",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic
        },
    );
    registry.insert(
        "SC2318",
        RuleMetadata {
            id: "SC2318",
            name: "Deprecated $[ ] syntax - use $(( ))",
            compatibility: ShellCompatibility::Universal, // Universal deprecation warning
        },
    );

    // Batch 15: Pattern matching (NotSh - if suggests [[]] specifically)
    registry.insert(
        "SC2314",
        RuleMetadata {
            id: "SC2314",
            name: "Use [[ ]] for pattern matching",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh specific
        },
    );

    // === BATCH 16 CLASSIFICATIONS (6 rules) ===

    // Batch 16: Positional parameters & arithmetic (Universal)
    registry.insert(
        "SC2320",
        RuleMetadata {
            id: "SC2320",
            name: "This $N expands to the parameter, not a separate word",
            compatibility: ShellCompatibility::Universal, // POSIX positional parameters
        },
    );
    registry.insert(
        "SC2322",
        RuleMetadata {
            id: "SC2322",
            name: "Arithmetic operations don't accept this argument count",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic
        },
    );
    registry.insert(
        "SC2323",
        RuleMetadata {
            id: "SC2323",
            name: "Arithmetic equality uses = not ==",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic style
        },
    );
    registry.insert(
        "SC2324",
        RuleMetadata {
            id: "SC2324",
            name: "Use ${var:+value} for conditional value based on isset",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        },
    );
    registry.insert(
        "SC2325",
        RuleMetadata {
            id: "SC2325",
            name: "Use $var instead of ${var} in arithmetic contexts",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic style
        },
    );

    // Batch 16: [[ ]] specific (NotSh)
    registry.insert(
        "SC2321",
        RuleMetadata {
            id: "SC2321",
            name: "This && is not a logical AND but part of [[ ]]",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh specific
        },
    );

    // === BATCH 17 CLASSIFICATIONS (21 rules - ALL REMAINING UNCLASSIFIED) ===
    // This batch completes 100% of implemented rules - 🎯🎯🎯 90% MILESTONE! 🎯🎯🎯

    // Batch 17: Backtick & Command Substitution (Universal)
    registry.insert(
        "SC2036",
        RuleMetadata {
            id: "SC2036",
            name: "Quotes in backticks need escaping. Use $( ) instead",
            compatibility: ShellCompatibility::Universal, // POSIX backticks
        },
    );
    registry.insert(
        "SC2037",
        RuleMetadata {
            id: "SC2037",
            name: "To assign command output, use var=$(cmd), not cmd > $var",
            compatibility: ShellCompatibility::Universal, // POSIX redirection vs command substitution
        },
    );

    // Batch 17: Function & Parameter Usage (Universal + NotSh)
    registry.insert(
        "SC2119",
        RuleMetadata {
            id: "SC2119",
            name: "Use foo \"$@\" if function's $1 should mean script's $1",
            compatibility: ShellCompatibility::Universal, // POSIX positional parameters
        },
    );
    registry.insert(
        "SC2123",
        RuleMetadata {
            id: "SC2123",
            name: "PATH is the shell search path. Assign to path instead",
            compatibility: ShellCompatibility::Universal, // POSIX PATH variable
        },
    );
    registry.insert(
        "SC2124",
        RuleMetadata {
            id: "SC2124",
            name: "Use \"${var[@]}\" to prevent word splitting",
            compatibility: ShellCompatibility::NotSh, // Arrays are bash/zsh/ksh specific
        },
    );
    registry.insert(
        "SC2125",
        RuleMetadata {
            id: "SC2125",
            name: "Brace expansion doesn't happen in [[ ]]",
            compatibility: ShellCompatibility::Universal, // Brace expansion behavior is consistent
        },
    );

    // Batch 17: Parameter Expansion & Command Optimization (Mixed)
    registry.insert(
        "SC2292",
        RuleMetadata {
            id: "SC2292",
            name: "Prefer ${var:0:1} over expr substr for single character",
            compatibility: ShellCompatibility::NotSh, // ${var:pos:len} is bash substring expansion
        },
    );
    registry.insert(
        "SC2293",
        RuleMetadata {
            id: "SC2293",
            name: "Use += to append to arrays",
            compatibility: ShellCompatibility::NotSh, // Array += is bash/zsh/ksh specific
        },
    );
    registry.insert(
        "SC2294",
        RuleMetadata {
            id: "SC2294",
            name: "Use arithmetic expansion ((...)) for simple assignments",
            compatibility: ShellCompatibility::Universal, // POSIX $(( )) arithmetic
        },
    );
    registry.insert(
        "SC2295",
        RuleMetadata {
            id: "SC2295",
            name: "Expansions inside ${} need to be quoted separately",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion quoting
        },
    );
    registry.insert(
        "SC2296",
        RuleMetadata {
            id: "SC2296",
            name: "Parameter expansions can't be nested",
            compatibility: ShellCompatibility::Universal, // POSIX limitation
        },
    );
    registry.insert(
        "SC2297",
        RuleMetadata {
            id: "SC2297",
            name: "Redirect before pipe",
            compatibility: ShellCompatibility::Universal, // POSIX shell pipeline ordering
        },
    );
    registry.insert(
        "SC2298",
        RuleMetadata {
            id: "SC2298",
            name: "Useless use of cat before pipe",
            compatibility: ShellCompatibility::Universal, // Universal anti-pattern
        },
    );
    registry.insert(
        "SC2299",
        RuleMetadata {
            id: "SC2299",
            name: "Parameter expansion only allows literals here",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion restrictions
        },
    );
    registry.insert(
        "SC2300",
        RuleMetadata {
            id: "SC2300",
            name: "Use ${var:?} for required environment variables",
            compatibility: ShellCompatibility::Universal, // POSIX ${var:?} parameter expansion
        },
    );
    registry.insert(
        "SC2301",
        RuleMetadata {
            id: "SC2301",
            name: "Use [[ -v array[0] ]] to check if array element exists",
            compatibility: ShellCompatibility::NotSh, // Arrays and [[ -v ]] are bash/zsh/ksh specific
        },
    );
    registry.insert(
        "SC2302",
        RuleMetadata {
            id: "SC2302",
            name: "Prefer ${var// /} over tr for simple substitution",
            compatibility: ShellCompatibility::NotSh, // ${var//pattern/replacement} is bash specific
        },
    );
    registry.insert(
        "SC2303",
        RuleMetadata {
            id: "SC2303",
            name: "Arithmetic base only allowed in assignments",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic base restrictions
        },
    );
    registry.insert(
        "SC2304",
        RuleMetadata {
            id: "SC2304",
            name: "Command appears to be undefined",
            compatibility: ShellCompatibility::Universal, // Universal command validation
        },
    );
    registry.insert(
        "SC2305",
        RuleMetadata {
            id: "SC2305",
            name: "Use ${var:=value} to assign default value",
            compatibility: ShellCompatibility::Universal, // POSIX ${var:=value} parameter expansion
        },
    );

    // Batch 17: Exit Code Usage (Universal)
    registry.insert(
        "SC2319",
        RuleMetadata {
            id: "SC2319",
            name: "This $? refers to a condition, not the previous command",
            compatibility: ShellCompatibility::Universal, // POSIX $? behavior
        },
    );

    // Makefile Rules (20 rules) - Universal (applies to all Make implementations)
    registry.insert(
        "MAKE001",
        RuleMetadata {
            id: "MAKE001",
            name: "Non-deterministic wildcard usage in Makefiles",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE002",
        RuleMetadata {
            id: "MAKE002",
            name: "Non-idempotent mkdir in Makefile recipes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE003",
        RuleMetadata {
            id: "MAKE003",
            name: "Unsafe variable expansion in Makefile recipes",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE004",
        RuleMetadata {
            id: "MAKE004",
            name: "Missing .PHONY declaration for non-file targets",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE005",
        RuleMetadata {
            id: "MAKE005",
            name: "Recursive variable assignment in Makefiles",
            compatibility: ShellCompatibility::Universal,
        },
    );
    registry.insert(
        "MAKE006",
        RuleMetadata {
            id: "MAKE006",
            name: "Missing target dependencies",
            compatibility: ShellCompatibility::Universal,
        },
    );
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
