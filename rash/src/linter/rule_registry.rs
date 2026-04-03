//! Rule Registry - Central metadata for all linter rules.
//!
//! Provides a central registry of all linter rules with their metadata,
//! including shell compatibility information. Use this to query which rules
//! apply to specific shell types.
//!
//! # Examples
//!
//! ## Checking rule compatibility
//!
//! ```
//! use bashrs::linter::rule_registry;
//! use bashrs::linter::ShellType;
//!
//! // Check if a rule applies to bash
//! assert!(rule_registry::should_apply_rule("SEC001", ShellType::Bash));
//!
//! // Check if a rule applies to POSIX sh
//! assert!(rule_registry::should_apply_rule("IDEM001", ShellType::Sh));
//! ```
//!
//! ## Getting rule metadata
//!
//! ```
//! use bashrs::linter::rule_registry;
//!
//! if let Some(compat) = rule_registry::get_rule_compatibility("SEC001") {
//!     println!("SEC001 compatibility: {:?}", compat);
//! }
//! ```

use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

/// Metadata for a linter rule, including shell compatibility.
///
/// Each rule has a unique ID, descriptive name, and compatibility specification
/// indicating which shell types the rule applies to.
///
/// # Examples
///
/// ## Accessing metadata from registry
///
/// ```
/// use bashrs::linter::rule_registry;
///
/// // Get compatibility for a security rule
/// let compat = rule_registry::get_rule_compatibility("SEC001");
/// assert!(compat.is_some());
/// ```
///
/// # Fields
///
/// * `id` - Unique rule identifier (e.g., "SEC001", "DET001", "IDEM001")
/// * `name` - Human-readable rule description
/// * `compatibility` - Shell compatibility specification
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// Unique rule identifier (e.g., "SEC001", "DET001").
    pub id: &'static str,

    /// Human-readable description of the rule.
    pub name: &'static str,

    /// Shell compatibility specification.
    ///
    /// Determines which shell types this rule applies to:
    /// - `Universal`: Applies to all shells
    /// - `BashOnly`: Applies only to bash
    /// - `PosixOnly`: Applies only to POSIX sh
    /// - `BashAndZsh`: Applies to bash and zsh
    pub compatibility: ShellCompatibility,
}

/// Gets the shell compatibility for a specific rule ID.
///
/// Returns the compatibility specification if the rule exists in the registry.
///
/// # Arguments
///
/// * `rule_id` - The rule identifier (e.g., "SEC001", "DET001")
///
/// # Returns
///
/// * `Some(ShellCompatibility)` - If rule exists in registry
/// * `None` - If rule ID not found
///
/// # Examples
///
/// ## Check security rule compatibility
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellCompatibility;
///
/// let compat = rule_registry::get_rule_compatibility("SEC001");
/// assert_eq!(compat, Some(ShellCompatibility::Universal));
/// ```
///
/// ## Handle unknown rules
///
/// ```
/// use bashrs::linter::rule_registry;
///
/// let compat = rule_registry::get_rule_compatibility("UNKNOWN");
/// assert!(compat.is_none());
/// ```
pub fn get_rule_compatibility(rule_id: &str) -> Option<ShellCompatibility> {
    RULE_REGISTRY.get(rule_id).map(|meta| meta.compatibility)
}

/// Returns metadata for a specific rule by ID.
pub fn get_rule_metadata(rule_id: &str) -> Option<&RuleMetadata> {
    RULE_REGISTRY.get(rule_id)
}

/// Returns all rule metadata entries sorted by ID.
pub fn all_rules() -> Vec<&'static RuleMetadata> {
    let mut rules: Vec<&RuleMetadata> = RULE_REGISTRY.values().collect();
    rules.sort_by_key(|r| r.id);
    rules
}

/// Checks if a rule should be applied for a given shell type.
///
/// Queries the rule registry and checks if the rule's compatibility
/// specification matches the target shell type.
///
/// # Arguments
///
/// * `rule_id` - The rule identifier to check
/// * `shell` - The target shell type
///
/// # Returns
///
/// * `true` - If rule applies to the shell type (or rule not in registry)
/// * `false` - If rule explicitly doesn't apply to the shell type
///
/// # Conservative Default
///
/// If a rule is not found in the registry, this function returns `true`
/// (conservative approach - assume rule applies unless explicitly excluded).
///
/// # Examples
///
/// ## Security rules (universal)
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellType;
///
/// // Security rules apply to all shells
/// assert!(rule_registry::should_apply_rule("SEC001", ShellType::Bash));
/// assert!(rule_registry::should_apply_rule("SEC001", ShellType::Sh));
/// assert!(rule_registry::should_apply_rule("SEC001", ShellType::Zsh));
/// ```
///
/// ## Filtering by shell type
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellType;
///
/// // Only apply rules that match the target shell
/// let rules_to_check = vec!["SEC001", "DET001", "IDEM001"];
/// let bash_rules: Vec<_> = rules_to_check
///     .into_iter()
///     .filter(|&rule| rule_registry::should_apply_rule(rule, ShellType::Bash))
///     .collect();
///
/// assert_eq!(bash_rules.len(), 3); // All universal rules
/// ```
///
/// ## Unknown rules default to applying
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellType;
///
/// // Unknown rules conservatively apply
/// assert!(rule_registry::should_apply_rule("UNKNOWN", ShellType::Bash));
/// ```
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

        // SC1xxx Source Code Issue Rules - Universal
        registry.insert("SC1007", RuleMetadata {
            id: "SC1007",
            name: "Remove space after = in variable assignment",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1009", RuleMetadata {
            id: "SC1009",
            name: "Comment detected where command was expected",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1017", RuleMetadata {
            id: "SC1017",
            name: "Literal carriage return in source",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1018", RuleMetadata {
            id: "SC1018",
            name: "Unicode non-breaking space used",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1020", RuleMetadata {
            id: "SC1020",
            name: "Missing space before closing ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1035", RuleMetadata {
            id: "SC1035",
            name: "Missing space after keyword",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1068", RuleMetadata {
            id: "SC1068",
            name: "Don't put spaces around = in assignments",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1069", RuleMetadata {
            id: "SC1069",
            name: "Missing space before [ in test",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1082", RuleMetadata {
            id: "SC1082",
            name: "UTF-8 BOM detected",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1095", RuleMetadata {
            id: "SC1095",
            name: "Space between function name and () with function keyword",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC1099", RuleMetadata {
            id: "SC1099",
            name: "Missing space before # comment",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1100", RuleMetadata {
            id: "SC1100",
            name: "Unicode dash used instead of minus",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1101", RuleMetadata {
            id: "SC1101",
            name: "Trailing spaces after \\ line continuation",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1109", RuleMetadata {
            id: "SC1109",
            name: "Unquoted HTML entity in script",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1129", RuleMetadata {
            id: "SC1129",
            name: "Missing space before ! in negation",
            compatibility: ShellCompatibility::Universal,
        });

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
        registry.insert("SC2058", RuleMetadata {
            id: "SC2058",
            name: "Unknown unary operator in test",
            compatibility: ShellCompatibility::Universal,
        });

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

        // Trap and signal handling (Universal - P0 timing issue)
        registry.insert("SC2063", RuleMetadata {
            id: "SC2063",
            name: "Grep regex vs literal string matching",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2064", RuleMetadata {
            id: "SC2064",
            name: "Trap command quoting (P0 - timing issue)",
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

        // Batch 18: File handling and command best practices (Universal)
        registry.insert("SC2008", RuleMetadata {
            id: "SC2008",
            name: "echo doesn't read from stdin",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2009", RuleMetadata {
            id: "SC2009",
            name: "Consider using pgrep instead of grepping ps output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2010", RuleMetadata {
            id: "SC2010",
            name: "Don't use ls | grep, use glob or find",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2011", RuleMetadata {
            id: "SC2011",
            name: "Use find -print0 | xargs -0 instead of ls | xargs",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2012", RuleMetadata {
            id: "SC2012",
            name: "Use find instead of ls for non-alphanumeric filenames",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2013", RuleMetadata {
            id: "SC2013",
            name: "To read lines, pipe/redirect to 'while read' loop",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2014", RuleMetadata {
            id: "SC2014",
            name: "Variables don't expand before brace expansion",
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

        // === BATCH 8 CLASSIFICATIONS (20 rules) ===

        // Batch 8: Exit code & bracket safety (Universal)
        registry.insert("SC2158", RuleMetadata {
            id: "SC2158",
            name: "[ true ] evaluates as literal '[', not test command",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2159", RuleMetadata {
            id: "SC2159",
            name: "[ [ with space creates syntax error (double bracket mistake)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2160", RuleMetadata {
            id: "SC2160",
            name: "Instead of 'if var; then', use 'if [ -n \"$var\" ]; then'",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2161", RuleMetadata {
            id: "SC2161",
            name: "Provide explicit error handling for cd commands",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 8: read command safety (Universal)
        registry.insert("SC2162", RuleMetadata {
            id: "SC2162",
            name: "read without -r will mangle backslashes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2163", RuleMetadata {
            id: "SC2163",
            name: "export command with array syntax (non-portable)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2164", RuleMetadata {
            id: "SC2164",
            name: "cd without error check (use ||, &&, or if)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 8: Trap & signal handling (Mixed)
        registry.insert("SC2165", RuleMetadata {
            id: "SC2165",
            name: "Subshells don't inherit traps - use functions instead",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2166", RuleMetadata {
            id: "SC2166",
            name: "Prefer [ p ] && [ q ] over [ p -a q ] (POSIX portability)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2167", RuleMetadata {
            id: "SC2167",
            name: "Trap handler doesn't propagate to subshells",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2168", RuleMetadata {
            id: "SC2168",
            name: "'local' keyword is only valid in functions",
            compatibility: ShellCompatibility::NotSh, // local is bash/ksh/zsh specific
        });

        // Batch 8: Test operators & syntax (Universal)
        registry.insert("SC2169", RuleMetadata {
            id: "SC2169",
            name: "In dash/sh, -eq is undefined for strings (use = instead)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2170", RuleMetadata {
            id: "SC2170",
            name: "Numerical -eq comparison on non-numeric strings",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2171", RuleMetadata {
            id: "SC2171",
            name: "Found trailing ] on the line (syntax error)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2172", RuleMetadata {
            id: "SC2172",
            name: "Trapping signals by number is deprecated (use names)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2173", RuleMetadata {
            id: "SC2173",
            name: "Trying to trap untrappable signals (SIGKILL, SIGSTOP)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 8: Security & best practices (Universal)
        registry.insert("SC2174", RuleMetadata {
            id: "SC2174",
            name: "mkdir -p and chmod in one shot creates security race",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2175", RuleMetadata {
            id: "SC2175",
            name: "Quote this to prevent word splitting (placeholder check)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2176", RuleMetadata {
            id: "SC2176",
            name: "'time' keyword affects full pipeline (not just first command)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2177", RuleMetadata {
            id: "SC2177",
            name: "'time' only times the first command (placeholder check)",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 9 CLASSIFICATIONS (20 rules) ===

        // Batch 9: Array operations (NotSh - bash/zsh/ksh only)
        registry.insert("SC2178", RuleMetadata {
            id: "SC2178",
            name: "Variable was used as an array but is now assigned a string",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2179", RuleMetadata {
            id: "SC2179",
            name: "Use array+=(\"item\") to append items to an array",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2180", RuleMetadata {
            id: "SC2180",
            name: "Trying to use an array as a scalar (missing index)",
            compatibility: ShellCompatibility::NotSh,
        });

        // Batch 9: Exit code and printf patterns (Universal)
        registry.insert("SC2181", RuleMetadata {
            id: "SC2181",
            name: "Check exit code directly with if mycmd, not if [ $? -eq 0 ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2182", RuleMetadata {
            id: "SC2182",
            name: "This printf format string has no variables",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 9: Assignment and expansion safety (Universal)
        registry.insert("SC2183", RuleMetadata {
            id: "SC2183",
            name: "This value looks like a variable but won't be expanded",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2184", RuleMetadata {
            id: "SC2184",
            name: "Quote arguments to cd to avoid glob expansion",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2185", RuleMetadata {
            id: "SC2185",
            name: "Some SSH commands don't pass on their exit codes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2186", RuleMetadata {
            id: "SC2186",
            name: "mktemp argument may be evaluated as template",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 9: Shell directives and redirection (Mixed)
        registry.insert("SC2187", RuleMetadata {
            id: "SC2187",
            name: "Ash scripts will be checked as Dash (use #!/bin/dash)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2188", RuleMetadata {
            id: "SC2188",
            name: "This redirection doesn't have a command",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2189", RuleMetadata {
            id: "SC2189",
            name: "Zsh directive will be checked as sh (use #!/bin/zsh)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 9: Associative arrays (NotSh - bash 4+ / zsh)
        registry.insert("SC2190", RuleMetadata {
            id: "SC2190",
            name: "Elements in associative arrays need index",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2191", RuleMetadata {
            id: "SC2191",
            name: "Trying to use an associative array without index",
            compatibility: ShellCompatibility::NotSh,
        });

        // Batch 9: Command composition and regex (Universal)
        registry.insert("SC2192", RuleMetadata {
            id: "SC2192",
            name: "Piping to sudo: only last command will run as root",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2193", RuleMetadata {
            id: "SC2193",
            name: "RHS of regexes must be unquoted in [[]]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2194", RuleMetadata {
            id: "SC2194",
            name: "This word is constant - did you forget $ or ()?",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2195", RuleMetadata {
            id: "SC2195",
            name: "Use single quotes to pass literal regex to grep",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2196", RuleMetadata {
            id: "SC2196",
            name: "Prefer explicit -n to check output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2197", RuleMetadata {
            id: "SC2197",
            name: "Don't compare globs in []; use [[ ]] or case",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 10 CLASSIFICATIONS (20 rules) ===

        // Batch 10: Command structure & ordering (Universal)
        registry.insert("SC2202", RuleMetadata {
            id: "SC2202",
            name: "Order sensitivity (e.g., redirects)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2203", RuleMetadata {
            id: "SC2203",
            name: "Variable assignment order matters",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2204", RuleMetadata {
            id: "SC2204",
            name: "Exit traps must come before commands",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2205", RuleMetadata {
            id: "SC2205",
            name: "Command ordering with pipes",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 10: Array operations (NotSh - bash/zsh/ksh only)
        registry.insert("SC2206", RuleMetadata {
            id: "SC2206",
            name: "Quote to prevent word splitting/globbing in arrays",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC2207", RuleMetadata {
            id: "SC2207",
            name: "Prefer mapfile or read -a to split command output",
            compatibility: ShellCompatibility::NotSh,
        });

        // Batch 10: Command structure & find usage (Universal)
        registry.insert("SC2208", RuleMetadata {
            id: "SC2208",
            name: "Command grouping issues",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2209", RuleMetadata {
            id: "SC2209",
            name: "Use single quotes for literal strings in find",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 10: Arithmetic operations (Universal)
        registry.insert("SC2210", RuleMetadata {
            id: "SC2210",
            name: "Don't use arithmetic shortcuts like x=++y",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2211", RuleMetadata {
            id: "SC2211",
            name: "Arithmetic on variable without $(())",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 10: Control flow & test operators (Universal)
        registry.insert("SC2212", RuleMetadata {
            id: "SC2212",
            name: "Use [ p ] || [ q ] instead of [ p -o q ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2213", RuleMetadata {
            id: "SC2213",
            name: "getopts requires argument variable",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2214", RuleMetadata {
            id: "SC2214",
            name: "Arithmetic comparison outside test",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2215", RuleMetadata {
            id: "SC2215",
            name: "Expression precedence issues",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2216", RuleMetadata {
            id: "SC2216",
            name: "Piping find to shell with ; instead of +",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2217", RuleMetadata {
            id: "SC2217",
            name: "Useless cat with find",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2218", RuleMetadata {
            id: "SC2218",
            name: "Useless return in command substitution",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2219", RuleMetadata {
            id: "SC2219",
            name: "Instead of let expr, use (( expr ))",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 10: Arithmetic syntax (Universal)
        registry.insert("SC2220", RuleMetadata {
            id: "SC2220",
            name: "Invalid arithmetic expression",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2221", RuleMetadata {
            id: "SC2221",
            name: "Arithmetic syntax errors",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 11 CLASSIFICATIONS (20 rules) ===

        // Batch 11: Case statement syntax (Universal)
        registry.insert("SC2222", RuleMetadata {
            id: "SC2222",
            name: "Lexical error in case statement syntax",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2223", RuleMetadata {
            id: "SC2223",
            name: "This default case is unreachable (previous pattern catches all)",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 11: Control flow & test operators (Universal)
        registry.insert("SC2224", RuleMetadata {
            id: "SC2224",
            name: "Quote the word or use a glob",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2225", RuleMetadata {
            id: "SC2225",
            name: "Use : or true instead of /bin/true",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2226", RuleMetadata {
            id: "SC2226",
            name: "This expression is constant",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2227", RuleMetadata {
            id: "SC2227",
            name: "Redirection applies to the echo, not the assignment",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2228", RuleMetadata {
            id: "SC2228",
            name: "Declare -x is equivalent to export",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2229", RuleMetadata {
            id: "SC2229",
            name: "This does not read 'foo'. Remove $/${} for that",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 11: Command existence & portability (Universal)
        registry.insert("SC2230", RuleMetadata {
            id: "SC2230",
            name: "which is non-standard, use command -v instead",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2231", RuleMetadata {
            id: "SC2231",
            name: "Quote expansions in this for loop glob to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2232", RuleMetadata {
            id: "SC2232",
            name: "Can't use sudo with builtins like cd",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2233", RuleMetadata {
            id: "SC2233",
            name: "Remove superfluous (..) around condition",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2234", RuleMetadata {
            id: "SC2234",
            name: "Remove superfluous () around here document",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 11: Quoting & expansion safety (Universal)
        registry.insert("SC2235", RuleMetadata {
            id: "SC2235",
            name: "Quote arguments to unalias to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2236", RuleMetadata {
            id: "SC2236",
            name: "Use -n instead of ! -z",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2237", RuleMetadata {
            id: "SC2237",
            name: "Use [ ] instead of [[ ]] (for sh compatibility)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2238", RuleMetadata {
            id: "SC2238",
            name: "Prefer ${} over backticks (readability + nesting)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2239", RuleMetadata {
            id: "SC2239",
            name: "Ensure consistent quoting for redirects",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2240", RuleMetadata {
            id: "SC2240",
            name: "The dot command does not support arguments in sh",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2241", RuleMetadata {
            id: "SC2241",
            name: "Exit code is always overridden by following command",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 12 CLASSIFICATIONS (20 rules) ===

        // Batch 12: Control flow & case statements (Universal)
        registry.insert("SC2242", RuleMetadata {
            id: "SC2242",
            name: "Can only break/continue from loops, not case",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2243", RuleMetadata {
            id: "SC2243",
            name: "Prefer explicit -n to check for output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2244", RuleMetadata {
            id: "SC2244",
            name: "Prefer explicit -n to check for output (variation)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2245", RuleMetadata {
            id: "SC2245",
            name: "-d test on assignment result",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2246", RuleMetadata {
            id: "SC2246",
            name: "This shebang was unrecognized",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 12: Test operators & efficiency (Universal)
        registry.insert("SC2247", RuleMetadata {
            id: "SC2247",
            name: "Prefer [ p ] && [ q ] over [ p -a q ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2248", RuleMetadata {
            id: "SC2248",
            name: "Prefer explicit -n to check for output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2249", RuleMetadata {
            id: "SC2249",
            name: "Consider adding default case in case statement",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2250", RuleMetadata {
            id: "SC2250",
            name: "Prefer $((..)) over let for arithmetic",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2251", RuleMetadata {
            id: "SC2251",
            name: "This loop will only ever run once for constant",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 12: Loop & case patterns (Universal)
        registry.insert("SC2252", RuleMetadata {
            id: "SC2252",
            name: "You probably wanted && here, not a second [",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2253", RuleMetadata {
            id: "SC2253",
            name: "Quote the RHS of = in [[ ]] to prevent glob matching",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2254", RuleMetadata {
            id: "SC2254",
            name: "Quote expansions in case patterns to prevent word splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2255", RuleMetadata {
            id: "SC2255",
            name: "This [ .. ] is true whenever str is non-empty",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2256", RuleMetadata {
            id: "SC2256",
            name: "Prefer -n/-z over comparison with empty string",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 12: Command safety & quoting (Universal)
        registry.insert("SC2257", RuleMetadata {
            id: "SC2257",
            name: "Prefer explicit -n to check non-empty string",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2258", RuleMetadata {
            id: "SC2258",
            name: "Prefer explicit -n to check output",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2259", RuleMetadata {
            id: "SC2259",
            name: "This assumes $RANDOM is always positive",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2260", RuleMetadata {
            id: "SC2260",
            name: "Fix $((..)) arithmetic so [[ ]] can interpret it",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2261", RuleMetadata {
            id: "SC2261",
            name: "Unquoted operand will be glob expanded",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 13 CLASSIFICATIONS (20 rules) ===

        // Batch 13: Quoting & parameter safety (Universal)
        registry.insert("SC2262", RuleMetadata {
            id: "SC2262",
            name: "This command may need quoting (context sensitive)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2263", RuleMetadata {
            id: "SC2263",
            name: "Use cd ... || exit to handle cd failures",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2264", RuleMetadata {
            id: "SC2264",
            name: "Prefer [ p ] && [ q ] over [ p -a q ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2265", RuleMetadata {
            id: "SC2265",
            name: "Use ${var:?} to ensure this never expands to /* /",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2266", RuleMetadata {
            id: "SC2266",
            name: "Prefer [ p ] || [ q ] over [ p -o q ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2267", RuleMetadata {
            id: "SC2267",
            name: "Use ${var:?} to ensure variable is set",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2268", RuleMetadata {
            id: "SC2268",
            name: "Avoid x-prefix in comparisons",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2269", RuleMetadata {
            id: "SC2269",
            name: "This regex should be put in a variable",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 13: Argument parsing & best practices (Universal)
        registry.insert("SC2270", RuleMetadata {
            id: "SC2270",
            name: "Prefer getopts over manual argument parsing",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2271", RuleMetadata {
            id: "SC2271",
            name: "Prefer printf over echo for non-trivial formatting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2272", RuleMetadata {
            id: "SC2272",
            name: "This is a constant, not a variable",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2273", RuleMetadata {
            id: "SC2273",
            name: "Use ${var:?} if this should never be empty",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2274", RuleMetadata {
            id: "SC2274",
            name: "Quote the RHS of = in [ ] to prevent globbing",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 13: Word splitting & expansion safety (Universal)
        registry.insert("SC2275", RuleMetadata {
            id: "SC2275",
            name: "Use ${var} to avoid field splitting",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2276", RuleMetadata {
            id: "SC2276",
            name: "Prefer explicit -n to check non-empty",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2277", RuleMetadata {
            id: "SC2277",
            name: "Use || instead of -o for test operators",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2278", RuleMetadata {
            id: "SC2278",
            name: "Use [[ ]] instead of deprecated syntax",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2279", RuleMetadata {
            id: "SC2279",
            name: "Use [[ < instead of [ <",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2280", RuleMetadata {
            id: "SC2280",
            name: "Remove redundant (..) or use 'if .. then'",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2281", RuleMetadata {
            id: "SC2281",
            name: "Don't use $@ in double quotes, it breaks word splitting",
            compatibility: ShellCompatibility::Universal,
        });

        // === BATCH 14 CLASSIFICATIONS (10 rules) ===

        // Batch 14: Parameter expansion & safety (Universal)
        registry.insert("SC2282", RuleMetadata {
            id: "SC2282",
            name: "Use ${var:?} to require variables to be set",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2283", RuleMetadata {
            id: "SC2283",
            name: "Remove extra spaces after ! in test expressions",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2284", RuleMetadata {
            id: "SC2284",
            name: "Use ${var:+value} for conditional value assignment",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2285", RuleMetadata {
            id: "SC2285",
            name: "Remove $ from variables in arithmetic contexts",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 14: Bash-specific features (NotSh - bash/zsh/ksh only)
        registry.insert("SC2286", RuleMetadata {
            id: "SC2286",
            name: "Prefer mapfile/readarray over read loops",
            compatibility: ShellCompatibility::NotSh, // mapfile/readarray are bash 4+ builtins
        });
        registry.insert("SC2287", RuleMetadata {
            id: "SC2287",
            name: "Use [[ -v var ]] to check if variable is set",
            compatibility: ShellCompatibility::NotSh, // [[ -v ]] is bash/zsh/ksh specific
        });

        // Batch 14: Best practices & style (Universal)
        registry.insert("SC2288", RuleMetadata {
            id: "SC2288",
            name: "Use true/false directly instead of [ 1 = 1 ]",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC2289", RuleMetadata {
            id: "SC2289",
            name: "Use ${#var} instead of expr length for string length",
            compatibility: ShellCompatibility::Universal,
        });

        // Batch 14: Bash arrays (NotSh - bash/zsh/ksh only)
        registry.insert("SC2290", RuleMetadata {
            id: "SC2290",
            name: "Remove $ from array index: ${array[i]} not ${array[$i]}",
            compatibility: ShellCompatibility::NotSh, // Arrays are bash-specific
        });
        registry.insert("SC2291", RuleMetadata {
            id: "SC2291",
            name: "Use [[ ! -v var ]] to check if variable is unset",
            compatibility: ShellCompatibility::NotSh, // [[ ! -v ]] is bash/zsh/ksh specific
        });

        // === BATCH 15 CLASSIFICATIONS (13 rules) ===

        // Batch 15: Bash-specific parameter expansion (NotSh)
        registry.insert("SC2306", RuleMetadata {
            id: "SC2306",
            name: "Use ${var//old/new} instead of sed for simple substitutions",
            compatibility: ShellCompatibility::NotSh, // ${var//} is bash parameter expansion
        });

        // Batch 15: POSIX parameter expansion (Universal)
        registry.insert("SC2307", RuleMetadata {
            id: "SC2307",
            name: "Use ${var#prefix} to remove prefix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        });
        registry.insert("SC2308", RuleMetadata {
            id: "SC2308",
            name: "Use ${var%suffix} to remove suffix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        });
        registry.insert("SC2309", RuleMetadata {
            id: "SC2309",
            name: "Use ${var##prefix} to remove longest prefix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        });
        registry.insert("SC2311", RuleMetadata {
            id: "SC2311",
            name: "Use ${var%%suffix} to remove longest suffix",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        });
        registry.insert("SC2315", RuleMetadata {
            id: "SC2315",
            name: "Use ${var:+replacement} for conditional replacement",
            compatibility: ShellCompatibility::Universal, // POSIX ${var:+value}
        });

        // Batch 15: set -e behavior & control flow (Universal)
        registry.insert("SC2310", RuleMetadata {
            id: "SC2310",
            name: "Function in condition - set -e doesn't apply",
            compatibility: ShellCompatibility::Universal, // POSIX set -e behavior
        });
        registry.insert("SC2316", RuleMetadata {
            id: "SC2316",
            name: "Command group and precedence issues",
            compatibility: ShellCompatibility::Universal, // POSIX control flow
        });
        registry.insert("SC2317", RuleMetadata {
            id: "SC2317",
            name: "Unreachable code detection",
            compatibility: ShellCompatibility::Universal, // Universal logic
        });

        // Batch 15: Deprecated syntax warnings (Universal)
        registry.insert("SC2312", RuleMetadata {
            id: "SC2312",
            name: "Deprecated local -x syntax",
            compatibility: ShellCompatibility::Universal, // Universal portability warning
        });
        registry.insert("SC2313", RuleMetadata {
            id: "SC2313",
            name: "Use $(( )) for arithmetic",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic
        });
        registry.insert("SC2318", RuleMetadata {
            id: "SC2318",
            name: "Deprecated $[ ] syntax - use $(( ))",
            compatibility: ShellCompatibility::Universal, // Universal deprecation warning
        });

        // Batch 15: Pattern matching (NotSh - if suggests [[]] specifically)
        registry.insert("SC2314", RuleMetadata {
            id: "SC2314",
            name: "Use [[ ]] for pattern matching",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh specific
        });

        // === BATCH 16 CLASSIFICATIONS (6 rules) ===

        // Batch 16: Positional parameters & arithmetic (Universal)
        registry.insert("SC2320", RuleMetadata {
            id: "SC2320",
            name: "This $N expands to the parameter, not a separate word",
            compatibility: ShellCompatibility::Universal, // POSIX positional parameters
        });
        registry.insert("SC2322", RuleMetadata {
            id: "SC2322",
            name: "Arithmetic operations don't accept this argument count",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic
        });
        registry.insert("SC2323", RuleMetadata {
            id: "SC2323",
            name: "Arithmetic equality uses = not ==",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic style
        });
        registry.insert("SC2324", RuleMetadata {
            id: "SC2324",
            name: "Use ${var:+value} for conditional value based on isset",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion
        });
        registry.insert("SC2325", RuleMetadata {
            id: "SC2325",
            name: "Use $var instead of ${var} in arithmetic contexts",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic style
        });

        // Batch 16: [[ ]] specific (NotSh)
        registry.insert("SC2321", RuleMetadata {
            id: "SC2321",
            name: "This && is not a logical AND but part of [[ ]]",
            compatibility: ShellCompatibility::NotSh, // [[ ]] is bash/zsh/ksh specific
        });

        // === BATCH 17 CLASSIFICATIONS (21 rules - ALL REMAINING UNCLASSIFIED) ===
        // This batch completes 100% of implemented rules - 🎯🎯🎯 90% MILESTONE! 🎯🎯🎯

        // Batch 17: Backtick & Command Substitution (Universal)
        registry.insert("SC2036", RuleMetadata {
            id: "SC2036",
            name: "Quotes in backticks need escaping. Use $( ) instead",
            compatibility: ShellCompatibility::Universal, // POSIX backticks
        });
        registry.insert("SC2037", RuleMetadata {
            id: "SC2037",
            name: "To assign command output, use var=$(cmd), not cmd > $var",
            compatibility: ShellCompatibility::Universal, // POSIX redirection vs command substitution
        });

        // Batch 17: Function & Parameter Usage (Universal + NotSh)
        registry.insert("SC2119", RuleMetadata {
            id: "SC2119",
            name: "Use foo \"$@\" if function's $1 should mean script's $1",
            compatibility: ShellCompatibility::Universal, // POSIX positional parameters
        });
        registry.insert("SC2123", RuleMetadata {
            id: "SC2123",
            name: "PATH is the shell search path. Assign to path instead",
            compatibility: ShellCompatibility::Universal, // POSIX PATH variable
        });
        registry.insert("SC2124", RuleMetadata {
            id: "SC2124",
            name: "Use \"${var[@]}\" to prevent word splitting",
            compatibility: ShellCompatibility::NotSh, // Arrays are bash/zsh/ksh specific
        });
        registry.insert("SC2125", RuleMetadata {
            id: "SC2125",
            name: "Brace expansion doesn't happen in [[ ]]",
            compatibility: ShellCompatibility::Universal, // Brace expansion behavior is consistent
        });

        // Batch 17: Parameter Expansion & Command Optimization (Mixed)
        registry.insert("SC2292", RuleMetadata {
            id: "SC2292",
            name: "Prefer ${var:0:1} over expr substr for single character",
            compatibility: ShellCompatibility::NotSh, // ${var:pos:len} is bash substring expansion
        });
        registry.insert("SC2293", RuleMetadata {
            id: "SC2293",
            name: "Use += to append to arrays",
            compatibility: ShellCompatibility::NotSh, // Array += is bash/zsh/ksh specific
        });
        registry.insert("SC2294", RuleMetadata {
            id: "SC2294",
            name: "Use arithmetic expansion ((...)) for simple assignments",
            compatibility: ShellCompatibility::Universal, // POSIX $(( )) arithmetic
        });
        registry.insert("SC2295", RuleMetadata {
            id: "SC2295",
            name: "Expansions inside ${} need to be quoted separately",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion quoting
        });
        registry.insert("SC2296", RuleMetadata {
            id: "SC2296",
            name: "Parameter expansions can't be nested",
            compatibility: ShellCompatibility::Universal, // POSIX limitation
        });
        registry.insert("SC2297", RuleMetadata {
            id: "SC2297",
            name: "Redirect before pipe",
            compatibility: ShellCompatibility::Universal, // POSIX shell pipeline ordering
        });
        registry.insert("SC2298", RuleMetadata {
            id: "SC2298",
            name: "Useless use of cat before pipe",
            compatibility: ShellCompatibility::Universal, // Universal anti-pattern
        });
        registry.insert("SC2299", RuleMetadata {
            id: "SC2299",
            name: "Parameter expansion only allows literals here",
            compatibility: ShellCompatibility::Universal, // POSIX parameter expansion restrictions
        });
        registry.insert("SC2300", RuleMetadata {
            id: "SC2300",
            name: "Use ${var:?} for required environment variables",
            compatibility: ShellCompatibility::Universal, // POSIX ${var:?} parameter expansion
        });
        registry.insert("SC2301", RuleMetadata {
            id: "SC2301",
            name: "Use [[ -v array[0] ]] to check if array element exists",
            compatibility: ShellCompatibility::NotSh, // Arrays and [[ -v ]] are bash/zsh/ksh specific
        });
        registry.insert("SC2302", RuleMetadata {
            id: "SC2302",
            name: "Prefer ${var// /} over tr for simple substitution",
            compatibility: ShellCompatibility::NotSh, // ${var//pattern/replacement} is bash specific
        });
        registry.insert("SC2303", RuleMetadata {
            id: "SC2303",
            name: "Arithmetic base only allowed in assignments",
            compatibility: ShellCompatibility::Universal, // POSIX arithmetic base restrictions
        });
        registry.insert("SC2304", RuleMetadata {
            id: "SC2304",
            name: "Command appears to be undefined",
            compatibility: ShellCompatibility::Universal, // Universal command validation
        });
        registry.insert("SC2305", RuleMetadata {
            id: "SC2305",
            name: "Use ${var:=value} to assign default value",
            compatibility: ShellCompatibility::Universal, // POSIX ${var:=value} parameter expansion
        });

        // Batch 17: Exit Code Usage (Universal)
        registry.insert("SC2319", RuleMetadata {
            id: "SC2319",
            name: "This $? refers to a condition, not the previous command",
            compatibility: ShellCompatibility::Universal, // POSIX $? behavior
        });

        // Makefile Rules (20 rules) - Universal (applies to all Make implementations)
        registry.insert("MAKE001", RuleMetadata {
            id: "MAKE001",
            name: "Non-deterministic wildcard usage in Makefiles",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE002", RuleMetadata {
            id: "MAKE002",
            name: "Non-idempotent mkdir in Makefile recipes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE003", RuleMetadata {
            id: "MAKE003",
            name: "Unsafe variable expansion in Makefile recipes",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE004", RuleMetadata {
            id: "MAKE004",
            name: "Missing .PHONY declaration for non-file targets",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE005", RuleMetadata {
            id: "MAKE005",
            name: "Recursive variable assignment in Makefiles",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE006", RuleMetadata {
            id: "MAKE006",
            name: "Missing target dependencies",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE007", RuleMetadata {
            id: "MAKE007",
            name: "Silent recipe errors (missing @ prefix)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE008", RuleMetadata {
            id: "MAKE008",
            name: "Tab vs spaces in recipes (CRITICAL)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE009", RuleMetadata {
            id: "MAKE009",
            name: "Hardcoded paths (non-portable)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE010", RuleMetadata {
            id: "MAKE010",
            name: "Missing error handling (|| exit 1)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE011", RuleMetadata {
            id: "MAKE011",
            name: "Dangerous pattern rules",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE012", RuleMetadata {
            id: "MAKE012",
            name: "Recursive make considered harmful",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE013", RuleMetadata {
            id: "MAKE013",
            name: "Missing .SUFFIXES (performance issue)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE014", RuleMetadata {
            id: "MAKE014",
            name: "Inefficient shell invocation",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE015", RuleMetadata {
            id: "MAKE015",
            name: "Missing .DELETE_ON_ERROR",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE016", RuleMetadata {
            id: "MAKE016",
            name: "Unquoted variable in prerequisites",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE017", RuleMetadata {
            id: "MAKE017",
            name: "Missing .ONESHELL",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE018", RuleMetadata {
            id: "MAKE018",
            name: "Parallel-unsafe targets (race conditions)",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE019", RuleMetadata {
            id: "MAKE019",
            name: "Environment variable pollution",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("MAKE020", RuleMetadata {
            id: "MAKE020",
            name: "Missing include guard",
            compatibility: ShellCompatibility::Universal,
        });

        // Most other SC2xxx rules are Universal (quoting, syntax, etc.)
        // They represent bugs or issues that apply regardless of shell
        // Examples: SC2086 (quote variables), etc.
        // These will be added as "Universal" as we classify them

        // Performance rules (PERF001-PERF005) - Universal
        registry.insert("PERF001", RuleMetadata {
            id: "PERF001",
            name: "Useless use of cat",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("PERF002", RuleMetadata {
            id: "PERF002",
            name: "Command substitution inside loop",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("PERF003", RuleMetadata {
            id: "PERF003",
            name: "Useless use of echo",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("PERF004", RuleMetadata {
            id: "PERF004",
            name: "find -exec with \\; instead of +",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("PERF005", RuleMetadata {
            id: "PERF005",
            name: "/bin/echo instead of builtin echo",
            compatibility: ShellCompatibility::Universal,
        });

        // Portability rules (PORT001-PORT005) - POSIX-only (fires on #!/bin/sh)
        registry.insert("PORT001", RuleMetadata {
            id: "PORT001",
            name: "Array syntax in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        });
        registry.insert("PORT002", RuleMetadata {
            id: "PORT002",
            name: "local keyword in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        });
        registry.insert("PORT003", RuleMetadata {
            id: "PORT003",
            name: "[[ ]] test in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        });
        registry.insert("PORT004", RuleMetadata {
            id: "PORT004",
            name: "Process substitution in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        });
        registry.insert("PORT005", RuleMetadata {
            id: "PORT005",
            name: "source instead of . in POSIX sh",
            compatibility: ShellCompatibility::ShOnly,
        });

        // Reliability rules (REL001-REL005) - Universal
        registry.insert("REL001", RuleMetadata {
            id: "REL001",
            name: "Destructive command without error check",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("REL002", RuleMetadata {
            id: "REL002",
            name: "mktemp without trap cleanup",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("REL003", RuleMetadata {
            id: "REL003",
            name: "read without timeout",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("REL004", RuleMetadata {
            id: "REL004",
            name: "TOCTOU race condition",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("REL005", RuleMetadata {
            id: "REL005",
            name: "Predictable temp file name",
            compatibility: ShellCompatibility::Universal,
        });

        // SC1xxx rules (source code / portability issues)
        registry.insert("SC1037", RuleMetadata {
            id: "SC1037",
            name: "Braces required for positional parameters beyond $9",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1076", RuleMetadata {
            id: "SC1076",
            name: "Deprecated $[...] arithmetic syntax",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1087", RuleMetadata {
            id: "SC1087",
            name: "Braces required for array access",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1105", RuleMetadata {
            id: "SC1105",
            name: "Space between $ and ( breaks command substitution",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1106", RuleMetadata {
            id: "SC1106",
            name: "Use -lt/-gt not </>  in single brackets",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1131", RuleMetadata {
            id: "SC1131",
            name: "Use elif instead of else followed by if",
            compatibility: ShellCompatibility::Universal,
        });
        registry.insert("SC1139", RuleMetadata {
            id: "SC1139",
            name: "Use || instead of -o in [[ ]]",
            compatibility: ShellCompatibility::NotSh,
        });
        registry.insert("SC1140", RuleMetadata {
            id: "SC1140",
            name: "Unexpected extra token after ]",
            compatibility: ShellCompatibility::Universal,
        });

        registry
    };
}

#[cfg(test)]
#[path = "rule_registry_tests.rs"]
mod tests;
