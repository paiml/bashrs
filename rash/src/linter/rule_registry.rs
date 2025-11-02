// Rule Registry
// Central metadata registry for all linter rules with shell compatibility

use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

/// Rule metadata including shell compatibility
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub compatibility: ShellCompatibility,
}

/// Get compatibility for a specific rule ID
pub fn get_rule_compatibility(rule_id: &str) -> Option<ShellCompatibility> {
    RULE_REGISTRY.get(rule_id).map(|meta| meta.compatibility)
}

/// Check if a rule should be applied for the given shell type
pub fn should_apply_rule(rule_id: &str, shell: crate::linter::shell_type::ShellType) -> bool {
    if let Some(compat) = get_rule_compatibility(rule_id) {
        compat.applies_to(shell)
    } else {
        // If rule not in registry, assume universal (conservative approach)
        true
    }
}

lazy_static::lazy_static! {
    /// Central registry of all linter rules with their compatibility
    static ref RULE_REGISTRY: HashMap<&'static str, RuleMetadata> = {
        let mut registry = HashMap::new();

        // Security Rules (8 rules) - Universal
        registry.insert("SEC001", RuleMetadata {
            id: "SEC001",
            name: "Command injection vulnerability",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC002", RuleMetadata {
            id: "SEC002",
            name: "Unsafe eval usage",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC003", RuleMetadata {
            id: "SEC003",
            name: "Unquoted variables (injection risk)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC004", RuleMetadata {
            id: "SEC004",
            name: "User input in commands",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC005", RuleMetadata {
            id: "SEC005",
            name: "Unsafe PATH modification",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC006", RuleMetadata {
            id: "SEC006",
            name: "Dangerous rm patterns",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC007", RuleMetadata {
            id: "SEC007",
            name: "Insecure temp file creation",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SEC008", RuleMetadata {
            id: "SEC008",
            name: "Source untrusted files",
            compatibility: ShellCompatibility::Universal,
        });

        // Determinism Rules (3 rules) - Universal
        registry.insert("DET001", RuleMetadata {
            id: "DET001",
            name: "$RANDOM usage (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("DET002", RuleMetadata {
            id: "DET002",
            name: "Timestamp usage (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("DET003", RuleMetadata {
            id: "DET003",
            name: "Wildcard ordering (non-deterministic)",
            compatibility: ShellCompatibility::Universal,
        });

        // Idempotency Rules (3 rules) - Universal
        registry.insert("IDEM001", RuleMetadata {
            id: "IDEM001",
            name: "mkdir without -p (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("IDEM002", RuleMetadata {
            id: "IDEM002",
            name: "rm without -f (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("IDEM003", RuleMetadata {
            id: "IDEM003",
            name: "ln without -sf (non-idempotent)",
            compatibility: ShellCompatibility::Universal,
        });

        // Bash-only rules: Arrays, [[]], process substitution, etc.
        // These should not fire on POSIX sh or pure zsh scripts

        // SC2039: Features undefined in POSIX sh (bash/zsh specific)
        registry.insert("SC2039", RuleMetadata {
            id: "SC2039",
            name: "Bash features undefined in POSIX sh",
            compatibility: ShellCompatibility::NotSh, // Works in bash/zsh/ksh but not sh
        });

        // SC2198: Arrays are bash-specific
        registry.insert("SC2198", RuleMetadata {
            id: "SC2198",
            name: "Array syntax (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // SC2199: Arrays are bash-specific
        registry.insert("SC2199", RuleMetadata {
            id: "SC2199",
            name: "Array expansion (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // SC2200: Arrays are bash-specific
        registry.insert("SC2200", RuleMetadata {
            id: "SC2200",
            name: "Array iteration (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // SC2201: Arrays are bash-specific
        registry.insert("SC2201", RuleMetadata {
            id: "SC2201",
            name: "Array assignment (bash-specific)",
            compatibility: ShellCompatibility::NotSh,
        });

        // Process substitution rules (bash/zsh, not POSIX sh)
        registry.insert("SC2002", RuleMetadata {
            id: "SC2002",
            name: "Useless cat (can use process substitution in bash/zsh)",
            compatibility: ShellCompatibility::NotSh,
        });

        // === BATCH 2 CLASSIFICATIONS (25 rules) ===

        // [[ ]] test syntax rules (NotSh - bash/zsh/ksh only)
        registry.insert("SC2108", RuleMetadata {
            id: "SC2108",
            name: "In [[ ]], use && instead of -a",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2109", RuleMetadata {
            id: "SC2109",
            name: "In [[ ]], use || instead of -o",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2110", RuleMetadata {
            id: "SC2110",
            name: "Don't mix && and || with -a and -o in [[ ]]",
            compatibility: ShellCompatibility::NotSh,
        });

        // function keyword rules (NotSh - bash/ksh only, not POSIX)
        registry.insert("SC2111", RuleMetadata {
            id: "SC2111",
            name: "'function' keyword not supported in sh",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2112", RuleMetadata {
            id: "SC2112",
            name: "'function' keyword is non-standard",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2113", RuleMetadata {
            id: "SC2113",
            name: "'function' keyword with () is redundant",
            compatibility: ShellCompatibility::NotSh,
        });

        // Arithmetic expansion rules (Universal - $((...)) is POSIX)
        registry.insert("SC2003", RuleMetadata {
            id: "SC2003",
            name: "expr is antiquated. Use $((...))",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2004", RuleMetadata {
            id: "SC2004",
            name: "$/${} unnecessary on arithmetic variables",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2079", RuleMetadata {
            id: "SC2079",
            name: "Decimals not supported in (( ))",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2080", RuleMetadata {
            id: "SC2080",
            name: "Leading zero interpreted as octal",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2084", RuleMetadata {
            id: "SC2084",
            name: "Arithmetic expansion as command",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2085", RuleMetadata {
            id: "SC2085",
            name: "Local variable with arithmetic",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2133", RuleMetadata {
            id: "SC2133",
            name: "Unexpected tokens in arithmetic expansion",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2134", RuleMetadata {
            id: "SC2134",
            name: "Use (( )) for numeric tests",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2137", RuleMetadata {
            id: "SC2137",
            name: "Unnecessary braces in arithmetic",
            compatibility: ShellCompatibility::Universal,
        });

        // Quoting and subshell rules (Universal - POSIX concepts)
        registry.insert("SC2030", RuleMetadata {
            id: "SC2030",
            name: "Variable modified in subshell",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2031", RuleMetadata {
            id: "SC2031",
            name: "Variable was modified in subshell",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2032", RuleMetadata {
            id: "SC2032",
            name: "Variable in script with shebang",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2087", RuleMetadata {
            id: "SC2087",
            name: "Quote variables in sh -c",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2088", RuleMetadata {
            id: "SC2088",
            name: "Tilde expansion in quotes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2089", RuleMetadata {
            id: "SC2089",
            name: "Quotes in assignment treated literally",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2090", RuleMetadata {
            id: "SC2090",
            name: "Quotes in expansion treated literally",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2091", RuleMetadata {
            id: "SC2091",
            name: "Remove $() to avoid executing output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2092", RuleMetadata {
            id: "SC2092",
            name: "Remove backticks to avoid executing output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2093", RuleMetadata {
            id: "SC2093",
            name: "Remove exec if script should continue",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 3 CLASSIFICATIONS (30 rules) ===

        // Loop and iteration safety (Universal)
        registry.insert("SC2038", RuleMetadata {
            id: "SC2038",
            name: "Use -print0/-0 or find -exec instead of for loop over find",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2040", RuleMetadata {
            id: "SC2040",
            name: "Avoid passing -o to other commands (shell option confusion)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2041", RuleMetadata {
            id: "SC2041",
            name: "Use while read, not read in for loop",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2042", RuleMetadata {
            id: "SC2042",
            name: "Use printf instead of echo with backslash escapes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2043", RuleMetadata {
            id: "SC2043",
            name: "This loop will only run once (for x in y without wildcards)",
            compatibility: ShellCompatibility::Universal,
        });

        // Test operators and conditionals (Universal)
        registry.insert("SC2044", RuleMetadata {
            id: "SC2044",
            name: "For loops over find: use find -exec or process substitution",
            compatibility: ShellCompatibility::NotSh, // process substitution suggestion
        });
        registry.insert("SC2045", RuleMetadata {
            id: "SC2045",
            name: "Iterating over ls output is fragile",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2046", RuleMetadata {
            id: "SC2046",
            name: "Quote to prevent word splitting (CRITICAL)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2047", RuleMetadata {
            id: "SC2047",
            name: "Quote variables in [ ] to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2048", RuleMetadata {
            id: "SC2048",
            name: "Use \"$@\" (with quotes) to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2049", RuleMetadata {
            id: "SC2049",
            name: "Use =~ for regex matching (not = in [ ])",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2050", RuleMetadata {
            id: "SC2050",
            name: "This expression is constant (forgot $ on variable?)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2051", RuleMetadata {
            id: "SC2051",
            name: "Bash doesn't expand variables in brace ranges {$a..$b}",
            compatibility: ShellCompatibility::Universal,
        });

        // Quoting and glob safety (mostly Universal, one NotSh)
        registry.insert("SC2052", RuleMetadata {
            id: "SC2052",
            name: "Use [[ ]] instead of [ ] for glob patterns",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh
        });
        registry.insert("SC2053", RuleMetadata {
            id: "SC2053",
            name: "Quote RHS of = in [ ] to prevent glob matching",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2054", RuleMetadata {
            id: "SC2054",
            name: "Comma is just literal in [[ ]]; use array or separate comparison",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2055", RuleMetadata {
            id: "SC2055",
            name: "Deprecated -a operator in test (use &&)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2056", RuleMetadata {
            id: "SC2056",
            name: "Deprecated -o operator in test (use ||)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2057", RuleMetadata {
            id: "SC2057",
            name: "Unknown binary operator (===, =!, <>)",
            compatibility: ShellCompatibility::Universal,
        });
        // SC2058: Not implemented yet (unknown unary operator)
        // registry.insert("SC2058", RuleMetadata {
        //     id: "SC2058",
        //     name: "Unknown unary operator in test",
        //     compatibility: ShellCompatibility::Universal,
        // });

        // Command safety and redirection (Universal - CRITICAL security rules)
        registry.insert("SC2059", RuleMetadata {
            id: "SC2059",
            name: "Printf format string injection (CRITICAL security)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2060", RuleMetadata {
            id: "SC2060",
            name: "Unquoted tr parameters (glob expansion)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2061", RuleMetadata {
            id: "SC2061",
            name: "Quote parameters to tr to prevent globbing",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2062", RuleMetadata {
            id: "SC2062",
            name: "Grep pattern glob expansion prevention",
            compatibility: ShellCompatibility::Universal,
        });

        // Trap and signal handling (Universal - including CRITICAL timing bug)
        registry.insert("SC2063", RuleMetadata {
            id: "SC2063",
            name: "Grep regex vs literal string matching",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2064", RuleMetadata {
            id: "SC2064",
            name: "Trap command quoting (CRITICAL - timing bug)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2065", RuleMetadata {
            id: "SC2065",
            name: "Shell redirection interpretation in strings",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2066", RuleMetadata {
            id: "SC2066",
            name: "Missing semicolon before done in for loop",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 4 CLASSIFICATIONS (30 rules) ===

        // Variable and parameter safety (Universal)
        registry.insert("SC2067", RuleMetadata {
            id: "SC2067",
            name: "Missing $ on array lookup",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2068", RuleMetadata {
            id: "SC2068",
            name: "Quote $@ to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2069", RuleMetadata {
            id: "SC2069",
            name: "To redirect stdout+stderr, use &> or 2>&1, not 1>&2",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2070", RuleMetadata {
            id: "SC2070",
            name: "-n doesn't work with unquoted arguments",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2071", RuleMetadata {
            id: "SC2071",
            name: "Arithmetic operators don't work in [ ]. Use [[ ]] or (( ))",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2072", RuleMetadata {
            id: "SC2072",
            name: "Lexicographic comparison in [ ]. Use -lt/-gt for numbers",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2073", RuleMetadata {
            id: "SC2073",
            name: "Escape \\d in character class or use [[:digit:]]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2074", RuleMetadata {
            id: "SC2074",
            name: "Can't use =~ in [ ]. Use [[ ]] instead",
            compatibility: ShellCompatibility::Universal,
        });

        // Quote and expansion safety (Universal)
        registry.insert("SC2075", RuleMetadata {
            id: "SC2075",
            name: "Escaping quotes in single quotes doesn't work",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2076", RuleMetadata {
            id: "SC2076",
            name: "Don't quote RHS of =~ in [[ ]]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2077", RuleMetadata {
            id: "SC2077",
            name: "Quote regex argument to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2078", RuleMetadata {
            id: "SC2078",
            name: "This expression is constant (forgot $ on variable?)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2081", RuleMetadata {
            id: "SC2081",
            name: "Escape [ in globs or use [[ ]]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2082", RuleMetadata {
            id: "SC2082",
            name: "Variable indirection with $$ (use ${!var})",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2083", RuleMetadata {
            id: "SC2083",
            name: "Don't add spaces after shebang",
            compatibility: ShellCompatibility::Universal,
        });

        // Command and redirection safety (Universal - CRITICAL)
        registry.insert("SC2094", RuleMetadata {
            id: "SC2094",
            name: "Don't use same file for input and output (will truncate)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2095", RuleMetadata {
            id: "SC2095",
            name: "ssh -t/-T in loops may consume stdin",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2096", RuleMetadata {
            id: "SC2096",
            name: "Use #! shebang, not just # comment",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2097", RuleMetadata {
            id: "SC2097",
            name: "Assign and use variable separately",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2098", RuleMetadata {
            id: "SC2098",
            name: "Variable assignment vs redirection confusion",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2103", RuleMetadata {
            id: "SC2103",
            name: "cd without error check (use cd ... || exit)",
            compatibility: ShellCompatibility::Universal,
        });

        // Test and conditional safety (Universal)
        registry.insert("SC2104", RuleMetadata {
            id: "SC2104",
            name: "In [[ ]], == is literal. Use = or [[ ]]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2105", RuleMetadata {
            id: "SC2105",
            name: "Break outside loop",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2107", RuleMetadata {
            id: "SC2107",
            name: "Instead of [ a -o b ], use [ a ] || [ b ]",
            compatibility: ShellCompatibility::Universal,
        });

        // Function and scope safety (Universal - CRITICAL dangerous rm)
        registry.insert("SC2114", RuleMetadata {
            id: "SC2114",
            name: "Dangerous rm -rf without validation ($VAR might be empty)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2115", RuleMetadata {
            id: "SC2115",
            name: "Use ${var:?} to ensure var is set before rm -rf",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2116", RuleMetadata {
            id: "SC2116",
            name: "Useless echo $(cmd) - just use cmd",
            compatibility: ShellCompatibility::Universal,
        });

        // Bash-specific rules (NotSh)
        // SC2120: Not enabled yet (has false positives requiring AST parsing)
        // registry.insert("SC2120", RuleMetadata {
        //     id: "SC2120",
        //     name: "Function references $1 but none passed",
        //     compatibility: ShellCompatibility::NotSh, // Requires bash function analysis
        // });
        registry.insert("SC2128", RuleMetadata {
            id: "SC2128",
            name: "Expanding array without index in bash",
            compatibility: ShellCompatibility::NotSh,
        });

        // === BATCH 5 CLASSIFICATIONS (20 rules) ===

        // Batch 5: Command optimization and best practices (Universal)
        registry.insert("SC2001", RuleMetadata {
            id: "SC2001",
            name: "Use ${var//pattern/replacement} instead of sed",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2005", RuleMetadata {
            id: "SC2005",
            name: "Useless echo instead of bare command",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2006", RuleMetadata {
            id: "SC2006",
            name: "Use $(...) instead of deprecated backticks",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2007", RuleMetadata {
            id: "SC2007",
            name: "Use $((..)) instead of deprecated expr",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 5: Logic and quoting safety (Universal)
        registry.insert("SC2015", RuleMetadata {
            id: "SC2015",
            name: "Note && and || precedence (use explicit grouping)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2016", RuleMetadata {
            id: "SC2016",
            name: "Expressions don't expand in single quotes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2017", RuleMetadata {
            id: "SC2017",
            name: "Increase precision by replacing bc/awk with arithmetic",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 5: tr character classes (Universal)
        registry.insert("SC2018", RuleMetadata {
            id: "SC2018",
            name: "Use [:upper:] instead of [A-Z] for tr",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2019", RuleMetadata {
            id: "SC2019",
            name: "Use [:lower:] instead of [a-z] for tr",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2020", RuleMetadata {
            id: "SC2020",
            name: "tr replaces sets of chars, not strings",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2021", RuleMetadata {
            id: "SC2021",
            name: "Don't use [] around classes in tr",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 5: SSH and command safety (Universal)
        registry.insert("SC2022", RuleMetadata {
            id: "SC2022",
            name: "Note: set -x only affects the current shell",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2023", RuleMetadata {
            id: "SC2023",
            name: "Brace expansion doesn't happen in [[ ]]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2024", RuleMetadata {
            id: "SC2024",
            name: "sudo only affects the command, not the redirection",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2025", RuleMetadata {
            id: "SC2025",
            name: "Note: set -e only affects the current shell",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2026", RuleMetadata {
            id: "SC2026",
            name: "Word splitting occurs in the variable",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 5: Quoting and echo safety (Universal)
        registry.insert("SC2027", RuleMetadata {
            id: "SC2027",
            name: "Quote or escape $ in double quotes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2028", RuleMetadata {
            id: "SC2028",
            name: "echo may not expand \\n (use printf)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2029", RuleMetadata {
            id: "SC2029",
            name: "Variables must be local in remote SSH command",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 5: CRITICAL word splitting (Universal)
        registry.insert("SC2086", RuleMetadata {
            id: "SC2086",
            name: "CRITICAL: Quote to prevent word splitting and globbing",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 6 CLASSIFICATIONS (20 rules) ===

        // Batch 6: Variable and function safety (Universal)
        registry.insert("SC2033", RuleMetadata {
            id: "SC2033",
            name: "Shell functions can't be exported (use scripts or ENV)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2034", RuleMetadata {
            id: "SC2034",
            name: "Variable appears unused (verify with shellcheck)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2035", RuleMetadata {
            id: "SC2035",
            name: "Use ./*glob* or -- *glob* to match files starting with -",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 6: Command best practices (Universal)
        registry.insert("SC2099", RuleMetadata {
            id: "SC2099",
            name: "Use $(...) instead of deprecated backticks",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2100", RuleMetadata {
            id: "SC2100",
            name: "Use $((..)) instead of deprecated expr",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2101", RuleMetadata {
            id: "SC2101",
            name: "Named POSIX class needs outer [] (e.g., [[:digit:]])",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2102", RuleMetadata {
            id: "SC2102",
            name: "Ranges only work with single chars (not regex +)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2106", RuleMetadata {
            id: "SC2106",
            name: "Consider using pgrep instead of ps | grep",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2117", RuleMetadata {
            id: "SC2117",
            name: "Unreachable code after exit or return",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 6: Ksh-specific (NotSh)
        registry.insert("SC2118", RuleMetadata {
            id: "SC2118",
            name: "Ksh-specific set -A won't work in sh",
            compatibility: ShellCompatibility::NotSh,
        });

        // Batch 6: Assignment and operator safety (Universal)
        registry.insert("SC2121", RuleMetadata {
            id: "SC2121",
            name: "Don't use $ on left side of assignment",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2122", RuleMetadata {
            id: "SC2122",
            name: ">= not valid in [ ]. Use -ge for numeric comparison",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 6: Code quality and efficiency (Universal)
        registry.insert("SC2126", RuleMetadata {
            id: "SC2126",
            name: "Use grep -c instead of grep | wc -l (efficiency)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2127", RuleMetadata {
            id: "SC2127",
            name: "Constant comparison in [ ] (always true/false)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2129", RuleMetadata {
            id: "SC2129",
            name: "Use >> instead of repeated > redirects to same file",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2130", RuleMetadata {
            id: "SC2130",
            name: "-e flag usage clarification (valid file test)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2131", RuleMetadata {
            id: "SC2131",
            name: "Backslashes in single quotes are literal (no escaping)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2132", RuleMetadata {
            id: "SC2132",
            name: "Readonly variable used in for loop (will fail)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 6: Control flow safety (Universal)
        registry.insert("SC2135", RuleMetadata {
            id: "SC2135",
            name: "Unexpected 'then' after condition (missing semicolon or wrong keyword)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2136", RuleMetadata {
            id: "SC2136",
            name: "Unexpected 'do' in 'if' statement (should be 'then')",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 7 CLASSIFICATIONS (20 rules) ===

        // Batch 7: Alias and function context safety (Universal)
        registry.insert("SC2138", RuleMetadata {
            id: "SC2138",
            name: "Function defined in wrong context (if/loop) or reserved name",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2139", RuleMetadata {
            id: "SC2139",
            name: "Alias variable expands at definition time (not invocation)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2140", RuleMetadata {
            id: "SC2140",
            name: "Malformed quote concatenation (unquoted words between quotes)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2141", RuleMetadata {
            id: "SC2141",
            name: "Command receives stdin but ignores it (find, ls, echo, sudo)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2142", RuleMetadata {
            id: "SC2142",
            name: "Aliases can't use positional parameters (use functions instead)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 7: Find and glob efficiency (Universal)
        registry.insert("SC2143", RuleMetadata {
            id: "SC2143",
            name: "Use grep -q for efficiency (exits on first match)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2144", RuleMetadata {
            id: "SC2144",
            name: "-e test on glob that never matches (glob safety)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2145", RuleMetadata {
            id: "SC2145",
            name: "Argument mixin in arrays ($@ or $* unquoted in quotes)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2146", RuleMetadata {
            id: "SC2146",
            name: "find -o action grouping needs parentheses",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2147", RuleMetadata {
            id: "SC2147",
            name: "Literal tilde in PATH doesn't expand (use $HOME)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2148", RuleMetadata {
            id: "SC2148",
            name: "Add shebang to indicate interpreter (portability)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2149", RuleMetadata {
            id: "SC2149",
            name: "Remove quotes from unset variable names",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2150", RuleMetadata {
            id: "SC2150",
            name: "Use find -exec + instead of \\; for batch processing (efficiency)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 7: Return/exit code and control flow safety (Universal)
        registry.insert("SC2151", RuleMetadata {
            id: "SC2151",
            name: "Return code should be 0-255 (POSIX requirement)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2152", RuleMetadata {
            id: "SC2152",
            name: "Exit code should be 0-255 (POSIX requirement)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2153", RuleMetadata {
            id: "SC2153",
            name: "Possible misspelling: var=$VAR1, but only $VAR2 is defined",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2154", RuleMetadata {
            id: "SC2154",
            name: "Variable is referenced but not assigned (may be external)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2155", RuleMetadata {
            id: "SC2155",
            name: "Declare and assign separately to preserve exit code",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2156", RuleMetadata {
            id: "SC2156",
            name: "Injected filenames can cause command injection ($() in filenames)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2157", RuleMetadata {
            id: "SC2157",
            name: "Argument to [ -z/-n ] is always false due to literal strings",
            compatibility: ShellCompatibility::Universal,
        });

        // Most other SC2xxx rules are Universal (quoting, syntax, etc.)
        // They represent bugs or issues that apply regardless of shell
        // Examples: SC2086 (quote variables), etc.
        // These will be added as "Universal" as we classify them

        registry
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::shell_type::ShellType;

    #[test]
    fn test_sec_rules_are_universal() {
        for i in 1..=8 {
            let rule_id = format!("SEC{:03}", i);
            let compat = get_rule_compatibility(&rule_id);
            assert_eq!(compat, Some(ShellCompatibility::Universal));
        }
    }

    #[test]
    fn test_det_rules_are_universal() {
        for i in 1..=3 {
            let rule_id = format!("DET{:03}", i);
            let compat = get_rule_compatibility(&rule_id);
            assert_eq!(compat, Some(ShellCompatibility::Universal));
        }
    }

    #[test]
    fn test_idem_rules_are_universal() {
        for i in 1..=3 {
            let rule_id = format!("IDEM{:03}", i);
            let compat = get_rule_compatibility(&rule_id);
            assert_eq!(compat, Some(ShellCompatibility::Universal));
        }
    }

    #[test]
    fn test_should_apply_universal_rules_to_all_shells() {
        let shells = vec![
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Sh,
            ShellType::Ksh,
            ShellType::Auto,
        ];

        for shell in shells {
            assert!(should_apply_rule("SEC001", shell));
            assert!(should_apply_rule("DET001", shell));
            assert!(should_apply_rule("IDEM001", shell));
        }
    }

    #[test]
    fn test_unknown_rule_defaults_to_universal() {
        // Unknown rules default to universal (conservative)
        assert!(should_apply_rule("UNKNOWN999", ShellType::Bash));
        assert!(should_apply_rule("UNKNOWN999", ShellType::Zsh));
        assert!(should_apply_rule("UNKNOWN999", ShellType::Sh));
    }

    #[test]
    fn test_registry_has_160_rules() {
        // Batch 1: 8 SEC + 3 DET + 3 IDEM + 6 SC2xxx = 20 rules
        // Batch 2: 6 NotSh + 19 Universal = 25 rules
        // Batch 3: 2 NotSh + 25 Universal = 27 rules (SC2058 not implemented yet)
        // Batch 4: 1 NotSh + 27 Universal = 28 rules (SC2120 has false positives, not enabled)
        // Batch 5: 0 NotSh + 20 Universal = 20 rules
        // Batch 6: 1 NotSh + 19 Universal = 20 rules
        // Batch 7: 0 NotSh + 20 Universal = 20 rules
        // Total: 160 rules (44.8% of 357 total)
        assert_eq!(RULE_REGISTRY.len(), 160);
    }

    #[test]
    fn test_bash_specific_rules_not_sh() {
        // Array and process substitution rules should be NotSh
        assert_eq!(
            get_rule_compatibility("SC2198"),
            Some(ShellCompatibility::NotSh)
        );
        assert_eq!(
            get_rule_compatibility("SC2199"),
            Some(ShellCompatibility::NotSh)
        );
        assert_eq!(
            get_rule_compatibility("SC2039"),
            Some(ShellCompatibility::NotSh)
        );
    }

    #[test]
    fn test_should_skip_bash_rules_for_sh() {
        // Bash-specific rules should not apply to POSIX sh
        assert!(!should_apply_rule("SC2198", ShellType::Sh));
        assert!(!should_apply_rule("SC2199", ShellType::Sh));

        // But should apply to bash and zsh
        assert!(should_apply_rule("SC2198", ShellType::Bash));
        assert!(should_apply_rule("SC2198", ShellType::Zsh));
    }

    // === Batch 2 Classification Tests ===

    #[test]
    fn test_double_bracket_rules_not_sh() {
        // [[ ]] syntax rules (SC2108-SC2110) should be NotSh
        let double_bracket_rules = vec!["SC2108", "SC2109", "SC2110"];

        for rule in double_bracket_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh),
                "{} should be NotSh",
                rule
            );

            // Should NOT apply to POSIX sh
            assert!(
                !should_apply_rule(rule, ShellType::Sh),
                "{} should not apply to sh",
                rule
            );

            // But SHOULD apply to bash and zsh
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Zsh),
                "{} should apply to zsh",
                rule
            );
        }
    }

    #[test]
    fn test_function_keyword_rules_not_sh() {
        // function keyword rules (SC2111-SC2113) should be NotSh
        let function_rules = vec!["SC2111", "SC2112", "SC2113"];

        for rule in function_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh),
                "{} should be NotSh",
                rule
            );

            // Should NOT apply to POSIX sh
            assert!(
                !should_apply_rule(rule, ShellType::Sh),
                "{} should not apply to sh",
                rule
            );

            // But SHOULD apply to bash and zsh
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
        }
    }

    #[test]
    fn test_arithmetic_rules_universal() {
        // Arithmetic rules (SC2003, SC2004, SC2079, SC2080, SC2084, SC2085, SC2133, SC2134, SC2137)
        let arithmetic_rules = vec![
            "SC2003", "SC2004", "SC2079", "SC2080", "SC2084", "SC2085", "SC2133", "SC2134",
            "SC2137",
        ];

        for rule in arithmetic_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );

            // Should apply to ALL shells
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Zsh),
                "{} should apply to zsh",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Sh),
                "{} should apply to sh",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Ksh),
                "{} should apply to ksh",
                rule
            );
        }
    }

    #[test]
    fn test_quoting_rules_universal() {
        // Quoting and subshell rules (SC2030, SC2031, SC2032, SC2087-SC2093)
        let quoting_rules = vec![
            "SC2030", "SC2031", "SC2032", "SC2087", "SC2088", "SC2089", "SC2090", "SC2091",
            "SC2092", "SC2093",
        ];

        for rule in quoting_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch2_notsh_count() {
        // Batch 2 should have 6 NotSh rules
        let notsh_rules = vec![
            "SC2108", "SC2109", "SC2110", // [[ ]]
            "SC2111", "SC2112", "SC2113", // function keyword
        ];

        for rule in notsh_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh)
            );
        }
    }

    #[test]
    fn test_batch2_universal_count() {
        // Batch 2 should have 19 Universal rules
        let universal_rules = vec![
            // Arithmetic (9 rules)
            "SC2003", "SC2004", "SC2079", "SC2080", "SC2084", "SC2085", "SC2133", "SC2134",
            "SC2137", // Quoting (10 rules)
            "SC2030", "SC2031", "SC2032", "SC2087", "SC2088", "SC2089", "SC2090", "SC2091",
            "SC2092", "SC2093",
        ];

        assert_eq!(universal_rules.len(), 19);

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal)
            );
        }
    }

    // === Batch 3 Classification Tests ===

    #[test]
    fn test_batch3_loop_safety_rules_universal() {
        // Loop safety rules (SC2038, SC2040-SC2043) should be Universal
        let loop_rules = vec!["SC2038", "SC2040", "SC2041", "SC2042", "SC2043"];

        for rule in loop_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch3_test_operators_mostly_universal() {
        // Most test operator rules are Universal
        let universal_test_rules = vec![
            "SC2045", "SC2046", "SC2047", "SC2048", "SC2049", "SC2050", "SC2051",
        ];

        for rule in universal_test_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }

        // SC2044 and SC2052 are NotSh (process substitution / [[ ]])
        assert_eq!(
            get_rule_compatibility("SC2044"),
            Some(ShellCompatibility::NotSh),
            "SC2044 should be NotSh (process substitution)"
        );
        assert_eq!(
            get_rule_compatibility("SC2052"),
            Some(ShellCompatibility::NotSh),
            "SC2052 should be NotSh ([[ ]] syntax)"
        );
    }

    #[test]
    fn test_batch3_critical_security_rules_universal() {
        // CRITICAL security rules must be Universal
        let critical_rules = vec![
            ("SC2059", "Printf format string injection"),
            ("SC2064", "Trap command timing bug"),
        ];

        for (rule, description) in critical_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} ({}) should be Universal - applies to all shells",
                rule,
                description
            );

            // Must apply to ALL shells
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
    fn test_batch3_quoting_rules_universal() {
        // Quoting and glob safety rules (SC2053-SC2057, SC2060-SC2063, SC2065-SC2066) - SC2058 not implemented
        let quoting_rules = vec![
            "SC2053", "SC2054", "SC2055", "SC2056", "SC2057", "SC2060", "SC2061", "SC2062",
            "SC2063", "SC2065", "SC2066",
        ];

        for rule in quoting_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }

    #[test]
    fn test_batch3_notsh_count() {
        // Batch 3 should have 2 NotSh rules
        let notsh_rules = vec![
            "SC2044", // process substitution suggestion
            "SC2052", // [[ ]] for globs
        ];

        for rule in notsh_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh)
            );

            // Should NOT apply to POSIX sh
            assert!(!should_apply_rule(rule, ShellType::Sh));

            // But SHOULD apply to bash/zsh
            assert!(should_apply_rule(rule, ShellType::Bash));
            assert!(should_apply_rule(rule, ShellType::Zsh));
        }
    }

    #[test]
    fn test_batch3_universal_count() {
        // Batch 3 should have 25 Universal rules (SC2044 and SC2052 are NotSh, SC2058 not implemented)
        let universal_rules = vec![
            // Loop safety (5)
            "SC2038", "SC2040", "SC2041", "SC2042",
            "SC2043", // Test operators (7, excluding SC2044 which is NotSh)
            "SC2045", "SC2046", "SC2047", "SC2048", "SC2049", "SC2050", "SC2051",
            // Quoting/glob (9, excluding SC2052 NotSh and SC2058 not implemented)
            "SC2053", "SC2054", "SC2055", "SC2056", "SC2057", "SC2060", "SC2061", "SC2062",
            "SC2063", // Security and trap (4)
            "SC2059", // format injection
            "SC2064", // trap timing
            "SC2065", // shell redirection
            "SC2066", // missing semicolon
        ];

        // Should be 25 unique rules
        let unique_count = universal_rules.len();
        assert_eq!(unique_count, 25, "Batch 3 should have 25 Universal rules");

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }

    // === Batch 4 Classification Tests ===

    #[test]
    fn test_batch4_variable_safety_universal() {
        // Variable and parameter safety rules (SC2067-SC2074) should be Universal
        let variable_rules = vec![
            "SC2067", "SC2068", "SC2069", "SC2070", "SC2071", "SC2072", "SC2073", "SC2074",
        ];

        for rule in variable_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch4_quoting_safety_universal() {
        // Quote and expansion safety rules should be Universal
        let quoting_rules = vec![
            "SC2075", "SC2076", "SC2077", "SC2078", "SC2081", "SC2082", "SC2083",
        ];

        for rule in quoting_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch4_command_safety_universal() {
        // Command and redirection safety rules should be Universal
        let command_rules = vec!["SC2094", "SC2095", "SC2096", "SC2097", "SC2098", "SC2103"];

        for rule in command_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch4_critical_dangerous_rm_universal() {
        // CRITICAL: Dangerous rm -rf rules (SC2114, SC2115) MUST be Universal
        let critical_rules = vec![
            ("SC2114", "Dangerous rm -rf without validation"),
            ("SC2115", "Use ${var:?} to ensure var is set before rm -rf"),
        ];

        for (rule, description) in critical_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} ({}) should be Universal - applies to all shells",
                rule,
                description
            );

            // Must apply to ALL shells (CRITICAL safety)
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
    fn test_batch4_notsh_count() {
        // Batch 4 should have 1 NotSh rule (SC2120 has false positives, not enabled)
        let notsh_rules = vec![
            // "SC2120", // Function parameter analysis (has false positives, not enabled)
            "SC2128", // Array expansion without index
        ];

        for rule in notsh_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::NotSh)
            );

            // Should NOT apply to POSIX sh
            assert!(!should_apply_rule(rule, ShellType::Sh));

            // But SHOULD apply to bash/zsh
            assert!(should_apply_rule(rule, ShellType::Bash));
            assert!(should_apply_rule(rule, ShellType::Zsh));
        }
    }

    #[test]
    fn test_batch4_universal_count() {
        // Batch 4 should have 27 Universal rules
        let universal_rules = vec![
            // Variable safety (8)
            "SC2067", "SC2068", "SC2069", "SC2070", "SC2071", "SC2072", "SC2073", "SC2074",
            // Quoting safety (7)
            "SC2075", "SC2076", "SC2077", "SC2078", "SC2081", "SC2082", "SC2083",
            // Command safety (6)
            "SC2094", "SC2095", "SC2096", "SC2097", "SC2098", "SC2103",
            // Test safety (3)
            "SC2104", "SC2105", "SC2107", // CRITICAL dangerous rm (2)
            "SC2114", "SC2115", // Echo safety (1)
            "SC2116",
        ];

        // Total: 8+7+6+3+2+1 = 27 Universal rules
        let unique_count = universal_rules.len();
        assert_eq!(unique_count, 27, "Batch 4 should have 27 Universal rules");

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }

    // === Batch 5 Classification Tests ===

    #[test]
    fn test_batch5_command_optimization_universal() {
        // Command optimization rules (SC2001, SC2005-2007) should be Universal
        let command_rules = vec!["SC2001", "SC2005", "SC2006", "SC2007"];

        for rule in command_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch5_logic_and_tr_universal() {
        // Logic, quoting, and tr character class rules should be Universal
        let logic_and_tr_rules = vec![
            // Logic (3)
            "SC2015", "SC2016", "SC2017", // tr character classes (4)
            "SC2018", "SC2019", "SC2020", "SC2021",
        ];

        for rule in logic_and_tr_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch5_ssh_and_quoting_universal() {
        // SSH, sudo, quoting, and echo safety rules should be Universal
        let ssh_and_quoting_rules = vec![
            // SSH and command safety (5)
            "SC2022", "SC2023", "SC2024", "SC2025", "SC2026", // Quoting and echo (3)
            "SC2027", "SC2028", "SC2029",
        ];

        for rule in ssh_and_quoting_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch5_critical_word_splitting_universal() {
        // CRITICAL: SC2086 (quote to prevent word splitting) MUST be Universal
        let critical_rule = "SC2086";

        assert_eq!(
            get_rule_compatibility(critical_rule),
            Some(ShellCompatibility::Universal),
            "SC2086 (CRITICAL word splitting) should be Universal"
        );

        // Must apply to ALL shells (CRITICAL safety)
        for shell in [
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Sh,
            ShellType::Ksh,
        ] {
            assert!(
                should_apply_rule(critical_rule, shell),
                "SC2086 should apply to {:?}",
                shell
            );
        }
    }

    #[test]
    fn test_batch5_universal_count() {
        // Batch 5 should have 20 Universal rules
        let universal_rules = vec![
            // Command optimization (4)
            "SC2001", "SC2005", "SC2006", "SC2007", // Logic and quoting (3)
            "SC2015", "SC2016", "SC2017", // tr character classes (4)
            "SC2018", "SC2019", "SC2020", "SC2021", // SSH and command safety (5)
            "SC2022", "SC2023", "SC2024", "SC2025", "SC2026", // Quoting and echo (3)
            "SC2027", "SC2028", "SC2029", // CRITICAL word splitting (1)
            "SC2086",
        ];

        // Total: 4+3+4+5+3+1 = 20 Universal rules
        let unique_count = universal_rules.len();
        assert_eq!(unique_count, 20, "Batch 5 should have 20 Universal rules");

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }

    // === Batch 6 Classification Tests ===

    #[test]
    fn test_batch6_variable_function_safety_universal() {
        // Variable and function safety rules (SC2033-2035) should be Universal
        let variable_rules = vec!["SC2033", "SC2034", "SC2035"];

        for rule in variable_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch6_command_best_practices_universal() {
        // Command best practices (SC2099-2102, SC2106, SC2117) should be Universal
        let command_rules = vec!["SC2099", "SC2100", "SC2101", "SC2102", "SC2106", "SC2117"];

        for rule in command_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch6_ksh_specific_notsh() {
        // SC2118 (ksh set -A arrays) should be NotSh
        let rule = "SC2118";

        assert_eq!(
            get_rule_compatibility(rule),
            Some(ShellCompatibility::NotSh),
            "{} should be NotSh (ksh-specific)",
            rule
        );

        // Should NOT apply to POSIX sh
        assert!(
            !should_apply_rule(rule, ShellType::Sh),
            "{} should not apply to sh",
            rule
        );

        // But SHOULD apply to bash/zsh/ksh
        assert!(
            should_apply_rule(rule, ShellType::Bash),
            "{} should apply to bash",
            rule
        );
        assert!(
            should_apply_rule(rule, ShellType::Zsh),
            "{} should apply to zsh",
            rule
        );
    }

    #[test]
    fn test_batch6_quality_efficiency_universal() {
        // Quality/efficiency rules should be Universal
        let quality_rules = vec![
            "SC2121", "SC2122", "SC2126", "SC2127", "SC2129", "SC2130", "SC2131", "SC2132",
            "SC2135", "SC2136",
        ];

        for rule in quality_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch6_universal_count() {
        // Batch 6 should have 20 rules total (19 Universal + 1 NotSh)
        let universal_rules = vec![
            // Variable/function safety (3)
            "SC2033", "SC2034", "SC2035", // Command best practices (6)
            "SC2099", "SC2100", "SC2101", "SC2102", "SC2106", "SC2117",
            // Quality/efficiency (10)
            "SC2121", "SC2122", "SC2126", "SC2127", "SC2129", "SC2130", "SC2131", "SC2132",
            "SC2135", "SC2136",
        ];

        // 19 Universal rules
        let unique_count = universal_rules.len();
        assert_eq!(unique_count, 19, "Batch 6 should have 19 Universal rules");

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }

        // 1 NotSh rule (ksh-specific)
        assert_eq!(
            get_rule_compatibility("SC2118"),
            Some(ShellCompatibility::NotSh),
            "SC2118 should be NotSh"
        );
    }

    // === Batch 7 Classification Tests ===

    #[test]
    fn test_batch7_alias_function_context_universal() {
        // Alias and function context safety rules (SC2138-SC2142) should be Universal
        let alias_function_rules = vec!["SC2138", "SC2139", "SC2140", "SC2141", "SC2142"];

        for rule in alias_function_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch7_find_glob_efficiency_universal() {
        // Find and glob efficiency rules (SC2143-SC2150) should be Universal
        let find_glob_rules = vec![
            "SC2143", "SC2144", "SC2145", "SC2146", "SC2147", "SC2148", "SC2149", "SC2150",
        ];

        for rule in find_glob_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch7_exit_code_safety_universal() {
        // Return/exit code and control flow safety rules (SC2151-SC2157) should be Universal
        let exit_code_rules = vec![
            "SC2151", "SC2152", "SC2153", "SC2154", "SC2155", "SC2156", "SC2157",
        ];

        for rule in exit_code_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
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
        }
    }

    #[test]
    fn test_batch7_universal_count() {
        // Batch 7 should have 20 rules total (all Universal)
        let universal_rules = vec![
            // Alias/function context (5)
            "SC2138", "SC2139", "SC2140", "SC2141", "SC2142",
            // Find/glob efficiency (8)
            "SC2143", "SC2144", "SC2145", "SC2146", "SC2147", "SC2148", "SC2149", "SC2150",
            // Return/exit codes (7)
            "SC2151", "SC2152", "SC2153", "SC2154", "SC2155", "SC2156", "SC2157",
        ];

        // 20 Universal rules
        let unique_count = universal_rules.len();
        assert_eq!(unique_count, 20, "Batch 7 should have 20 Universal rules");

        for rule in universal_rules {
            assert_eq!(
                get_rule_compatibility(rule),
                Some(ShellCompatibility::Universal),
                "{} should be Universal",
                rule
            );
        }
    }

    #[test]
    fn test_batch7_no_notsh_rules() {
        // Batch 7 should have NO NotSh rules (all Universal)
        let batch7_rules = vec![
            "SC2138", "SC2139", "SC2140", "SC2141", "SC2142", "SC2143", "SC2144", "SC2145",
            "SC2146", "SC2147", "SC2148", "SC2149", "SC2150", "SC2151", "SC2152", "SC2153",
            "SC2154", "SC2155", "SC2156", "SC2157",
        ];

        for rule in batch7_rules {
            let compat = get_rule_compatibility(rule);
            assert_eq!(
                compat,
                Some(ShellCompatibility::Universal),
                "{} should be Universal (not NotSh)",
                rule
            );

            // Should apply to ALL shells including sh
            assert!(
                should_apply_rule(rule, ShellType::Sh),
                "{} should apply to sh",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Bash),
                "{} should apply to bash",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Zsh),
                "{} should apply to zsh",
                rule
            );
            assert!(
                should_apply_rule(rule, ShellType::Ksh),
                "{} should apply to ksh",
                rule
            );
        }
    }
}
