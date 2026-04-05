/// Lint a shell script with shell-type detection based on file path.
///
/// Automatically detects the shell type (POSIX sh, bash, etc.) from the file extension
/// and shebang, then applies only compatible rules. This prevents false positives when
/// linting POSIX sh scripts with bash-specific rules.
///
/// # Arguments
///
/// * `path` - File path used for shell type detection (`.sh`, `.bash`, etc.)
/// * `source` - The shell script source code to lint
///
/// # Returns
///
/// A [`LintResult`] containing diagnostics appropriate for the detected shell type.
///
/// # Examples
///
/// ## POSIX sh script
///
/// ```
/// use bashrs::linter::lint_shell_with_path;
/// use std::path::Path;
///
/// let path = Path::new("script.sh");
/// let script = "#!/bin/sh\necho hello";
/// let result = lint_shell_with_path(path, script);
/// // Should detect POSIX sh and skip bash-only rules
/// // Note: May have style suggestions, but no critical errors
/// let errors = result.diagnostics.iter()
///     .filter(|d| d.severity == bashrs::linter::Severity::Error)
///     .count();
/// assert_eq!(errors, 0);
/// ```
///
/// ## Bash-specific script
///
/// ```
/// use bashrs::linter::lint_shell_with_path;
/// use std::path::Path;
///
/// let path = Path::new("script.bash");
/// let script = "#!/bin/bash\necho hello";
/// let result = lint_shell_with_path(path, script);
/// // Bash-specific rules are applied
/// // Should complete successfully with minimal issues
/// let errors = result.diagnostics.iter()
///     .filter(|d| d.severity == bashrs::linter::Severity::Error)
///     .count();
/// assert_eq!(errors, 0);
/// ```
pub fn lint_shell_with_path(path: &std::path::Path, source: &str) -> LintResult {
    use crate::linter::shell_type::detect_shell_type;

    // Detect shell type from path and content
    let shell_type = detect_shell_type(path, source);

    // Run rules with shell-specific filtering
    lint_shell_filtered(source, shell_type)
}

/// Lint a shell script with shell-specific rule filtering
///
/// Conditionally applies rules based on shell type compatibility.
/// For example, bash-only array rules are skipped for POSIX sh files.
///
/// # Arguments
/// * `source` - Shell script source code
/// * `shell_type` - Detected or explicit shell type
///
/// # Returns
/// LintResult with filtered diagnostics
fn lint_shell_filtered(
    source: &str,
    shell_type: crate::linter::shell_type::ShellType,
) -> LintResult {
    use crate::linter::rule_registry::should_apply_rule;
    use crate::linter::suppression::SuppressionManager;

    let mut result = LintResult::new();

    // Helper macro to conditionally apply rules
    macro_rules! apply_rule {
        ($rule_id:expr, $check_fn:expr) => {
            if should_apply_rule($rule_id, shell_type) {
                result.merge($check_fn(source));
            }
        };
    }

    // Run SC1xxx rules (source code issues - Universal)
    apply_rule!("SC1003", sc1003::check);
    apply_rule!("SC1004", sc1004::check);
    apply_rule!("SC1007", sc1007::check);
    apply_rule!("SC1008", sc1008::check);
    apply_rule!("SC1009", sc1009::check);
    apply_rule!("SC1012", sc1012::check);
    apply_rule!("SC1014", sc1014::check);
    apply_rule!("SC1017", sc1017::check);
    apply_rule!("SC1018", sc1018::check);
    apply_rule!("SC1020", sc1020::check);
    apply_rule!("SC1026", sc1026::check);
    apply_rule!("SC1028", sc1028::check);
    apply_rule!("SC1035", sc1035::check);
    apply_rule!("SC1036", sc1036::check);
    apply_rule!("SC1037", sc1037::check);
    apply_rule!("SC1038", sc1038::check);
    apply_rule!("SC1040", sc1040::check);
    apply_rule!("SC1041", sc1041::check);
    apply_rule!("SC1044", sc1044::check);
    apply_rule!("SC1045", sc1045::check);
    apply_rule!("SC1065", sc1065::check);
    apply_rule!("SC1066", sc1066::check);
    apply_rule!("SC1068", sc1068::check);
    apply_rule!("SC1069", sc1069::check);
    apply_rule!("SC1075", sc1075::check);
    apply_rule!("SC1076", sc1076::check);
    apply_rule!("SC1078", sc1078::check);
    apply_rule!("SC1079", sc1079::check);
    apply_rule!("SC1082", sc1082::check);
    apply_rule!("SC1083", sc1083::check);
    apply_rule!("SC1084", sc1084::check);
    apply_rule!("SC1086", sc1086::check);
    apply_rule!("SC1087", sc1087::check);
    apply_rule!("SC1090", sc1090::check);
    apply_rule!("SC1091", sc1091::check);
    apply_rule!("SC1094", sc1094::check);
    apply_rule!("SC1095", sc1095::check);
    apply_rule!("SC1097", sc1097::check);
    apply_rule!("SC1098", sc1098::check);
    apply_rule!("SC1099", sc1099::check);
    apply_rule!("SC1100", sc1100::check);
    apply_rule!("SC1101", sc1101::check);
    apply_rule!("SC1104", sc1104::check);
    apply_rule!("SC1105", sc1105::check);
    apply_rule!("SC1106", sc1106::check);
    apply_rule!("SC1109", sc1109::check);
    apply_rule!("SC1110", sc1110::check);
    apply_rule!("SC1111", sc1111::check);
    apply_rule!("SC1113", sc1113::check);
    apply_rule!("SC1114", sc1114::check);
    apply_rule!("SC1115", sc1115::check);
    apply_rule!("SC1117", sc1117::check);
    apply_rule!("SC1120", sc1120::check);
    apply_rule!("SC1127", sc1127::check);
    apply_rule!("SC1128", sc1128::check);
    apply_rule!("SC1129", sc1129::check);
    apply_rule!("SC1131", sc1131::check);
    apply_rule!("SC1135", sc1135::check);
    apply_rule!("SC1139", sc1139::check);
    apply_rule!("SC1140", sc1140::check);

    // Run ShellCheck-equivalent rules with filtering
    apply_rule!("SC2001", sc2001::check);
    apply_rule!("SC2002", sc2002::check);
    apply_rule!("SC2003", sc2003::check);
    apply_rule!("SC2004", sc2004::check);
    apply_rule!("SC2005", sc2005::check);
    apply_rule!("SC2006", sc2006::check);
    apply_rule!("SC2007", sc2007::check);
    apply_rule!("SC2015", sc2015::check);
    apply_rule!("SC2016", sc2016::check);
    apply_rule!("SC2017", sc2017::check);
    apply_rule!("SC2018", sc2018::check);
    apply_rule!("SC2019", sc2019::check);
    apply_rule!("SC2020", sc2020::check);
    apply_rule!("SC2021", sc2021::check);
    apply_rule!("SC2022", sc2022::check);
    apply_rule!("SC2023", sc2023::check);
    apply_rule!("SC2024", sc2024::check);
    apply_rule!("SC2025", sc2025::check);
    apply_rule!("SC2026", sc2026::check);
    apply_rule!("SC2027", sc2027::check);
    apply_rule!("SC2028", sc2028::check); // Universal - echo may not expand \\n (use printf)
    apply_rule!("SC2029", sc2029::check);
    apply_rule!("SC2030", sc2030::check);
    apply_rule!("SC2031", sc2031::check); // Universal - subshell scope
    apply_rule!("SC2032", sc2032::check); // Universal - variable in shebang script

    // Add classified rules (SC2039 and SC2198-2201)
    apply_rule!("SC2039", sc2039::check); // NotSh - bash/zsh features

    // Batch 2: Arithmetic and quoting rules (Universal)
    apply_rule!("SC2079", sc2079::check); // Universal - decimals in arithmetic
    apply_rule!("SC2080", sc2080::check); // Universal - octal numbers
    apply_rule!("SC2084", sc2084::check); // Universal - arithmetic as command
    apply_rule!("SC2085", sc2085::check); // Universal - local with arithmetic
    apply_rule!("SC2087", sc2087::check); // Universal - quote in sh -c
    apply_rule!("SC2088", sc2088::check); // Universal - tilde expansion
    apply_rule!("SC2089", sc2089::check); // Universal - quotes in assignment
    apply_rule!("SC2090", sc2090::check); // Universal - quotes in expansion
    apply_rule!("SC2091", sc2091::check); // Universal - remove $() execution
    apply_rule!("SC2092", sc2092::check); // Universal - remove backticks execution
    apply_rule!("SC2093", sc2093::check); // Universal - remove exec

    // Batch 2: [[ ]] test syntax rules (NotSh - bash/zsh/ksh only)
    apply_rule!("SC2108", sc2108::check); // NotSh - [[ ]] use && not -a
    apply_rule!("SC2109", sc2109::check); // NotSh - [[ ]] use || not -o
    apply_rule!("SC2110", sc2110::check); // NotSh - don't mix && || with -a -o

    // Batch 2: function keyword rules (NotSh - bash/ksh only)
    apply_rule!("SC2111", sc2111::check); // NotSh - function keyword in sh
    apply_rule!("SC2112", sc2112::check); // NotSh - function keyword non-standard
    apply_rule!("SC2113", sc2113::check); // NotSh - function with () redundant

    // Batch 2: More arithmetic rules (Universal)
    apply_rule!("SC2133", sc2133::check); // Universal - unexpected tokens in arithmetic
    apply_rule!("SC2134", sc2134::check); // Universal - use (( )) for numeric tests
    apply_rule!("SC2137", sc2137::check); // Universal - unnecessary braces in arithmetic

    // Array rules (NotSh)
    apply_rule!("SC2198", sc2198::check); // NotSh - arrays
    apply_rule!("SC2199", sc2199::check); // NotSh - arrays
    apply_rule!("SC2200", sc2200::check); // NotSh - arrays
    apply_rule!("SC2201", sc2201::check); // NotSh - arrays

    // Batch 3: Loop and iteration safety (Universal - all shells)
    apply_rule!("SC2038", sc2038::check); // Universal - find loop safety
    apply_rule!("SC2040", sc2040::check); // Universal - avoid -o confusion
    apply_rule!("SC2041", sc2041::check); // Universal - read in for loop
    apply_rule!("SC2042", sc2042::check); // Universal - echo vs printf
    apply_rule!("SC2043", sc2043::check); // Universal - loop runs once

    // Batch 3: Test operators and word splitting (mostly Universal)
    apply_rule!("SC2044", sc2044::check); // NotSh - process substitution suggestion
    apply_rule!("SC2045", sc2045::check); // Universal - iterating over ls
    apply_rule!("SC2046", sc2046::check); // Universal - CRITICAL word splitting
    apply_rule!("SC2047", sc2047::check); // Universal - quote variables in [ ]
    apply_rule!("SC2048", sc2048::check); // Universal - quote "$@"
    apply_rule!("SC2049", sc2049::check); // Universal - =~ for regex
    apply_rule!("SC2050", sc2050::check); // Universal - constant expression
    apply_rule!("SC2051", sc2051::check); // Universal - brace range expansion

    // Batch 3: Quoting and glob safety (mostly Universal)
    apply_rule!("SC2052", sc2052::check); // NotSh - [[ ]] for glob patterns
    apply_rule!("SC2053", sc2053::check); // Universal - quote RHS in [ ]
    apply_rule!("SC2054", sc2054::check); // Universal - comma in [[ ]]
    apply_rule!("SC2055", sc2055::check); // Universal - deprecated -a
    apply_rule!("SC2056", sc2056::check); // Universal - deprecated -o
    apply_rule!("SC2057", sc2057::check); // Universal - unknown binary operator
    apply_rule!("SC2058", sc2058::check); // Universal - unknown unary operator

    // Batch 3: Command safety and security (Universal - CRITICAL)
    apply_rule!("SC2059", sc2059::check); // Universal - CRITICAL printf format injection
    apply_rule!("SC2060", sc2060::check); // Universal - unquoted tr params
    apply_rule!("SC2061", sc2061::check); // Universal - quote tr params
    apply_rule!("SC2062", sc2062::check); // Universal - grep pattern glob expansion

    // Batch 3: Trap and signal handling (Universal - CRITICAL)
    apply_rule!("SC2063", sc2063::check); // Universal - grep regex vs literal
    apply_rule!("SC2064", sc2064::check); // Universal - CRITICAL trap timing bug
    apply_rule!("SC2065", sc2065::check); // Universal - shell redirection interpretation
    apply_rule!("SC2066", sc2066::check); // Universal - missing semicolon before done

    // Batch 4: Variable and parameter safety (Universal - all shells)
    apply_rule!("SC2067", sc2067::check); // Universal - missing $ on array lookup
    apply_rule!("SC2068", sc2068::check); // Universal - quote $@ to prevent word splitting
    apply_rule!("SC2069", sc2069::check); // Universal - redirect stdout+stderr correctly
    apply_rule!("SC2070", sc2070::check); // Universal - -n doesn't work unquoted
    apply_rule!("SC2071", sc2071::check); // Universal - arithmetic operators in [ ]
    apply_rule!("SC2072", sc2072::check); // Universal - lexicographic vs numeric comparison
    apply_rule!("SC2073", sc2073::check); // Universal - escape \\d or use [[:digit:]]
    apply_rule!("SC2074", sc2074::check); // Universal - can't use =~ in [ ]

    // Batch 4: Quote and expansion safety (Universal)
    apply_rule!("SC2075", sc2075::check); // Universal - escaping quotes in single quotes
    apply_rule!("SC2076", sc2076::check); // Universal - don't quote RHS of =~ in [[ ]]
    apply_rule!("SC2077", sc2077::check); // Universal - quote regex to prevent word splitting
    apply_rule!("SC2078", sc2078::check); // Universal - constant expression (forgot $?)
    apply_rule!("SC2081", sc2081::check); // Universal - escape [ in globs
    apply_rule!("SC2082", sc2082::check); // Universal - variable indirection with $$
    apply_rule!("SC2083", sc2083::check); // Universal - don't add spaces after shebang

    // Batch 4: Command and redirection safety (Universal - CRITICAL)
    apply_rule!("SC2094", sc2094::check); // Universal - same file for input/output (truncate)
    apply_rule!("SC2095", sc2095::check); // Universal - ssh in loops may consume stdin
    apply_rule!("SC2096", sc2096::check); // Universal - use #! shebang
    apply_rule!("SC2097", sc2097::check); // Universal - assign and use variable separately
    apply_rule!("SC2098", sc2098::check); // Universal - variable assignment vs redirection
    apply_rule!("SC2103", sc2103::check); // Universal - cd without error check

    // Batch 4: Test and conditional safety (Universal)
    apply_rule!("SC2104", sc2104::check); // Universal - == is literal in [[ ]]
    apply_rule!("SC2105", sc2105::check); // Universal - break outside loop
    apply_rule!("SC2107", sc2107::check); // Universal - use [ a ] || [ b ] not [ a -o b ]

    // Batch 4: Function and scope safety (Universal - CRITICAL dangerous rm)
    apply_rule!("SC2114", sc2114::check); // Universal - CRITICAL dangerous rm -rf without validation
    apply_rule!("SC2115", sc2115::check); // Universal - CRITICAL use ${var:?} before rm -rf
    apply_rule!("SC2116", sc2116::check); // Universal - useless echo $(cmd)

    // Batch 4: Bash-specific function analysis (NotSh - bash/zsh/ksh only)
    // apply_rule!("SC2120", sc2120::check); // NotSh - function references $1 but none passed (TODO: has false positives)
    apply_rule!("SC2128", sc2128::check); // NotSh - array expansion without index

    // Batch 5: CRITICAL word splitting (Universal - HIGHEST PRIORITY)
    apply_rule!("SC2086", sc2086::check); // Universal - CRITICAL: Quote to prevent word splitting and globbing

    // TODO: Add remaining SC2xxx rules (~237 rules remaining, was ~257)
    // For now, fall back to lint_shell() for unclassified rules
    // This ensures backward compatibility while we incrementally classify

    // Determinism rules (Universal - always apply)
    result.merge(det001::check(source));
    result.merge(det002::check(source));
    result.merge(det003::check(source));
    result.merge(det004::check(source));

    // Idempotency rules (Universal - always apply)
    result.merge(idem001::check(source));
    result.merge(idem002::check(source));
    result.merge(idem003::check(source));

    // Best practice rules
    result.merge(bash001::check(source));
    result.merge(bash002::check(source));
    result.merge(bash003::check(source));
    result.merge(bash004::check(source));
    result.merge(bash005::check(source));
    result.merge(bash006::check(source));
    result.merge(bash007::check(source));
    result.merge(bash008::check(source));
    result.merge(bash009::check(source));
    result.merge(bash010::check(source));

    // Security rules (Universal - always apply)
    result.merge(sec001::check(source));
    result.merge(sec002::check(source));
    result.merge(sec003::check(source));
    result.merge(sec004::check(source));
    result.merge(sec005::check(source));
    result.merge(sec006::check(source));
    result.merge(sec007::check(source));
    result.merge(sec008::check(source));
    result.merge(sec009::check(source));
    result.merge(sec010::check(source));
    result.merge(sec011::check(source));
    result.merge(sec012::check(source));
    result.merge(sec013::check(source));
    result.merge(sec014::check(source));
    result.merge(sec015::check(source));
    result.merge(sec016::check(source));
    result.merge(sec017::check(source));
    result.merge(sec018::check(source));
    // SEC019 not dispatched: unquoted variable detection has false positives on
    // well-known shell variables ($HOME, $RANDOM), causing misclassification
    result.merge(sec020::check(source));
    result.merge(sec021::check(source));
    result.merge(sec022::check(source));
    result.merge(sec023::check(source));
    result.merge(sec024::check(source));

    // Performance rules
    apply_rule!("PERF001", perf001::check);
    apply_rule!("PERF002", perf002::check);
    apply_rule!("PERF003", perf003::check);
    apply_rule!("PERF004", perf004::check);
    apply_rule!("PERF005", perf005::check);

    // Portability rules
    apply_rule!("PORT001", port001::check);
    apply_rule!("PORT002", port002::check);
    apply_rule!("PORT003", port003::check);
    apply_rule!("PORT004", port004::check);
    apply_rule!("PORT005", port005::check);

    // Reliability rules
    apply_rule!("REL001", rel001::check);
    apply_rule!("REL002", rel002::check);
    apply_rule!("REL003", rel003::check);
    apply_rule!("REL004", rel004::check);
    apply_rule!("REL005", rel005::check);

    // Apply inline suppression filtering
    let suppression_manager = SuppressionManager::from_source(source);
    result
        .diagnostics
        .retain(|diag| !suppression_manager.is_suppressed(&diag.code, diag.span.start_line));

    result
}


include!("mod_lint.rs");
