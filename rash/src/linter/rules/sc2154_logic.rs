//! SC2154 Pure Logic - Variable reference validation
//!
//! Extracted for EXTREME TDD testability.

use regex::Regex;
use std::collections::HashSet;

/// Get set of built-in/environment variables to skip
/// This includes POSIX standard variables and bash-specific builtins
pub fn get_builtins() -> HashSet<&'static str> {
    [
        // POSIX standard environment variables
        "HOME",
        "PATH",
        "PWD",
        "USER",
        "SHELL",
        "TERM",
        "LANG",
        "LC_ALL",
        "OLDPWD",
        "IFS",
        "OPTARG",
        "OPTIND",
        "PPID",
        "CDPATH",
        "MAILCHECK",
        "PS1",
        "PS2",
        "PS3",
        "PS4",
        "ENV",
        "FCEDIT",
        "HISTFILE",
        "HISTSIZE",
        "MAIL",
        "MAILPATH",
        "NLSPATH",
        "TMOUT",
        "COLUMNS",
        "LINES",
        // Bash specific - User/System info
        "EUID",
        "UID",
        "GROUPS",
        "HOSTNAME",
        "HOSTTYPE",
        "OSTYPE",
        "MACHTYPE",
        // Bash specific - Version info
        "BASH",
        "BASH_VERSION",
        "BASH_VERSINFO",
        "BASH_SUBSHELL",
        "BASHPID",
        // Bash specific - Special runtime variables
        "RANDOM",
        "SECONDS",
        "LINENO",
        "SHLVL",
        "REPLY",
        "EPOCHSECONDS",
        "EPOCHREALTIME",
        "SRANDOM",
        // Bash specific - Function/script context
        "FUNCNAME",
        "BASH_SOURCE",
        "BASH_LINENO",
        "FUNCNEST",
        // Bash specific - Command/execution context
        "BASH_COMMAND",
        "BASH_EXECUTION_STRING",
        "BASH_ARGC",
        "BASH_ARGV",
        "BASH_ARGV0",
        "BASH_REMATCH",
        "MAPFILE",
        "READLINE_LINE",
        "READLINE_POINT",
        "READLINE_MARK",
        // Bash specific - Pipeline/job status
        "PIPESTATUS",
        // Bash specific - Completion
        "COMP_WORDS",
        "COMP_CWORD",
        "COMP_LINE",
        "COMP_POINT",
        "COMP_TYPE",
        "COMP_KEY",
        "COMPREPLY",
        // Bash specific - Options
        "SHELLOPTS",
        "BASHOPTS",
        "BASH_COMPAT",
        // Bash specific - History
        "HISTCMD",
        "HISTCONTROL",
        "HISTIGNORE",
        "HISTTIMEFORMAT",
        // Bash specific - Directory stack
        "DIRSTACK",
        // Bash specific - Coprocesses
        "COPROC",
        // Common environment variables (widely used)
        "TMPDIR",
        "TEMP",
        "TMP",
        "EDITOR",
        "VISUAL",
        "PAGER",
        "BROWSER",
        "DISPLAY",
        "XAUTHORITY",
        "DBUS_SESSION_BUS_ADDRESS",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_CACHE_HOME",
        "XDG_RUNTIME_DIR",
        "XDG_SESSION_TYPE",
        "XDG_CURRENT_DESKTOP",
        "LOGNAME",
        "HOSTNAME",
        "HOSTFILE",
        "INPUTRC",
        // Terminal/locale
        "COLORTERM",
        "TERM_PROGRAM",
        "LC_CTYPE",
        "LC_MESSAGES",
        "LC_NUMERIC",
        "LC_TIME",
        "LC_COLLATE",
        "LC_MONETARY",
    ]
    .iter()
    .copied()
    .collect()
}

/// Check if variable is special or builtin (should be skipped)
pub fn is_special_or_builtin(var_name: &str, builtins: &HashSet<&str>) -> bool {
    // Skip if in builtins
    if builtins.contains(var_name) {
        return true;
    }

    // Skip numeric variables (positional parameters)
    if var_name.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // Skip special variables
    if ["@", "*", "#", "?", "$", "!", "0", "-"].contains(&var_name) {
        return true;
    }

    false
}

/// Check if script sources external files
/// If source/. commands are found, we're more lenient with undefined variables
pub fn has_source_commands(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        // Match: source file, . file, source "file", . "file"
        if trimmed.starts_with("source ") || trimmed.starts_with(". ") {
            return true;
        }
        // Also check for source/. after semicolon or &&/||
        if trimmed.contains("; source ")
            || trimmed.contains("; . ")
            || trimmed.contains("&& source ")
            || trimmed.contains("&& . ")
            || trimmed.contains("|| source ")
            || trimmed.contains("|| . ")
        {
            return true;
        }
    }
    false
}

/// Check if line is a comment
pub fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if variable is all uppercase (likely environment var)
pub fn is_uppercase_var(var_name: &str) -> bool {
    var_name.chars().all(|c| c.is_uppercase() || c == '_')
}

/// Check if a variable reference uses parameter expansion operators
/// ${VAR:-}, ${VAR:=}, ${VAR:+}, ${VAR:?} are intentional patterns
/// that should not trigger "undefined variable" warnings
///
/// Issue #132: Variables like ${BASHRS_TEST:-} are intentional env var checks
pub fn is_parameter_expansion_with_operator(line: &str, match_end: usize) -> bool {
    // Check what follows the variable name in the line
    let remaining = &line[match_end..];

    // Parameter expansion operators are :-, :=, :+, :?
    // Also handle the non-colon variants -, =, +, ?
    if remaining.starts_with(":-")
        || remaining.starts_with(":=")
        || remaining.starts_with(":+")
        || remaining.starts_with(":?")
        || remaining.starts_with('-')
        || remaining.starts_with('=')
        || remaining.starts_with('+')
        || remaining.starts_with('?')
    {
        return true;
    }

    false
}

/// Find variables assigned inside case statements with default branches
/// If a case has a *) default branch, variables assigned in ANY branch are considered defined
#[allow(clippy::expect_used)] // Compile-time regex
pub fn collect_case_statement_variables(source: &str) -> HashSet<String> {
    let mut case_vars: HashSet<String> = HashSet::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut in_case = false;
    let mut case_start = 0;
    let mut case_depth = 0;

    let assign_pattern =
        Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=").expect("valid assignment regex pattern");

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("case ") && trimmed.contains(" in") {
            if !in_case {
                in_case = true;
                case_start = i;
            }
            case_depth += 1;
        }

        if trimmed == "esac" || trimmed.starts_with("esac;") || trimmed.starts_with("esac ") {
            if case_depth > 0 {
                case_depth -= 1;
            }
            if case_depth == 0 && in_case {
                let case_block: Vec<&str> = lines[case_start..=i].to_vec();
                let has_default = case_block.iter().any(|l| {
                    let t = l.trim();
                    t.starts_with("*)") || t.starts_with("* )") || t.contains("*)")
                });

                if has_default {
                    for case_line in &case_block {
                        let t = case_line.trim();
                        if t.ends_with(')') && !t.contains('=') {
                            continue;
                        }
                        for cap in assign_pattern.captures_iter(case_line) {
                            if let Some(var) = cap.get(1) {
                                case_vars.insert(var.as_str().to_string());
                            }
                        }
                    }
                }
                in_case = false;
            }
        }
    }

    case_vars
}

/// Check if line is a case expression start
pub fn is_case_start(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("case ") && trimmed.contains(" in")
}

/// Check if line is case end
pub fn is_case_end(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "esac" || trimmed.starts_with("esac;") || trimmed.starts_with("esac ")
}

/// Check if case block has default branch
pub fn case_has_default(block: &[&str]) -> bool {
    block.iter().any(|l| {
        let t = l.trim();
        t.starts_with("*)") || t.starts_with("* )") || t.contains("*)")
    })
}

/// Check if line is a pattern line in case (like "a)" or "*)")
pub fn is_case_pattern_line(line: &str) -> bool {
    let t = line.trim();
    t.ends_with(')') && !t.contains('=')
}

/// Extract variable names from read command in a line
pub fn extract_read_variables(line: &str) -> Vec<String> {
    let mut vars = Vec::new();
    if let Some(read_pos) = line.find("read ") {
        let after_read = &line[read_pos + 5..];
        let parts: Vec<&str> = after_read.split_whitespace().collect();
        let mut i = 0;
        // Skip flags
        while i < parts.len() {
            let part = parts[i];
            if part.starts_with('-') {
                i += 1;
                if matches!(part, "-p" | "-a" | "-d" | "-n" | "-t" | "-u") {
                    i += 1;
                }
            } else {
                break;
            }
        }
        // Remaining parts are variable names
        while i < parts.len() {
            let var_name = parts[i].trim_end_matches(';');
            if var_name
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
                && var_name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                vars.push(var_name.to_string());
                i += 1;
            } else {
                break;
            }
        }
    }
    vars
}

/// Patterns for variable detection
pub struct Patterns {
    pub assign: Regex,
    pub use_: Regex,
    pub for_loop: Regex,
    pub c_style_for: Regex,
    pub case_expr: Regex,
}

/// Create regex patterns for variable detection
#[allow(clippy::unwrap_used)] // Compile-time regex
pub fn create_patterns() -> Patterns {
    Patterns {
        assign: Regex::new(
            r"^\s*(?:(?:local|readonly|export|declare|typeset)(?:\s+-[a-zA-Z]+)?\s+)?([A-Za-z_][A-Za-z0-9_]*)=",
        ).unwrap(),
        use_: Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap(),
        for_loop: Regex::new(r"\bfor\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\b").unwrap(),
        c_style_for: Regex::new(r"\bfor\s*\(\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap(),
        case_expr: Regex::new(r"\bcase\s+\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?\s+in\b").unwrap(),
    }
}

/// Collect variable assignments and uses from source
#[allow(clippy::unwrap_used)] // Regex captures in known patterns
pub fn collect_variable_info(
    source: &str,
    patterns: &Patterns,
) -> (HashSet<String>, Vec<(String, usize, usize)>) {
    let mut assigned: HashSet<String> = HashSet::new();
    let mut used_vars: Vec<(String, usize, usize)> = Vec::new();
    let has_sources = has_source_commands(source);

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if is_comment_line(line) {
            continue;
        }

        for cap in patterns.assign.captures_iter(line) {
            assigned.insert(cap.get(1).unwrap().as_str().to_string());
        }
        for cap in patterns.for_loop.captures_iter(line) {
            assigned.insert(cap.get(1).unwrap().as_str().to_string());
        }
        for cap in patterns.c_style_for.captures_iter(line) {
            assigned.insert(cap.get(1).unwrap().as_str().to_string());
        }
        for cap in patterns.case_expr.captures_iter(line) {
            assigned.insert(cap.get(1).unwrap().as_str().to_string());
        }
        for var in extract_read_variables(line) {
            assigned.insert(var);
        }
        for cap in patterns.use_.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap();
            let col = full_match.start() + 1;

            // Issue #132: Skip variables with parameter expansion operators
            // ${VAR:-}, ${VAR:=}, ${VAR:+}, ${VAR:?} are intentional default/check patterns
            if is_parameter_expansion_with_operator(line, full_match.end()) {
                continue;
            }

            if has_sources && is_uppercase_var(var_name) {
                continue;
            }
            used_vars.push((var_name.to_string(), line_num, col));
        }
    }
    (assigned, used_vars)
}

/// Validate undefined variables and return diagnostics info
pub fn find_undefined_variables(
    assigned: &HashSet<String>,
    used_vars: &[(String, usize, usize)],
    builtins: &HashSet<&str>,
) -> Vec<(String, usize, usize)> {
    let mut undefined = Vec::new();
    for (var_name, line_num, col) in used_vars {
        if assigned.contains(var_name) {
            continue;
        }
        if is_special_or_builtin(var_name, builtins) {
            continue;
        }
        undefined.push((var_name.clone(), *line_num, *col));
    }
    undefined
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== GET BUILTINS =====

    #[test]
    fn test_get_builtins_contains_posix() {
        let builtins = get_builtins();
        assert!(builtins.contains("HOME"));
        assert!(builtins.contains("PATH"));
        assert!(builtins.contains("PWD"));
        assert!(builtins.contains("USER"));
    }

    #[test]
    fn test_get_builtins_contains_bash() {
        let builtins = get_builtins();
        assert!(builtins.contains("BASH_VERSION"));
        assert!(builtins.contains("RANDOM"));
        assert!(builtins.contains("LINENO"));
    }

    #[test]
    fn test_get_builtins_not_empty() {
        let builtins = get_builtins();
        assert!(builtins.len() > 50);
    }

    // ===== IS SPECIAL OR BUILTIN =====

    #[test]
    fn test_is_special_or_builtin_builtin() {
        let builtins = get_builtins();
        assert!(is_special_or_builtin("HOME", &builtins));
        assert!(is_special_or_builtin("PATH", &builtins));
    }

    #[test]
    fn test_is_special_or_builtin_numeric() {
        let builtins = get_builtins();
        assert!(is_special_or_builtin("1", &builtins));
        assert!(is_special_or_builtin("2", &builtins));
        assert!(is_special_or_builtin("10", &builtins));
    }

    #[test]
    fn test_is_special_or_builtin_special() {
        let builtins = get_builtins();
        assert!(is_special_or_builtin("@", &builtins));
        assert!(is_special_or_builtin("*", &builtins));
        assert!(is_special_or_builtin("#", &builtins));
        assert!(is_special_or_builtin("?", &builtins));
    }

    #[test]
    fn test_is_special_or_builtin_regular() {
        let builtins = get_builtins();
        assert!(!is_special_or_builtin("my_var", &builtins));
        assert!(!is_special_or_builtin("foo", &builtins));
    }

    // ===== HAS SOURCE COMMANDS =====

    #[test]
    fn test_has_source_commands_source() {
        assert!(has_source_commands("source /etc/profile"));
        assert!(has_source_commands("source ./env.sh"));
    }

    #[test]
    fn test_has_source_commands_dot() {
        assert!(has_source_commands(". /etc/profile"));
        assert!(has_source_commands(". ./env.sh"));
    }

    #[test]
    fn test_has_source_commands_in_line() {
        assert!(has_source_commands("cd /tmp; source ./env.sh"));
        assert!(has_source_commands("test && source ./env.sh"));
    }

    #[test]
    fn test_has_source_commands_none() {
        assert!(!has_source_commands("echo hello"));
        assert!(!has_source_commands("# source comment"));
    }

    // ===== IS COMMENT LINE =====

    #[test]
    fn test_is_comment_line_true() {
        assert!(is_comment_line("# comment"));
        assert!(is_comment_line("  # indented"));
    }

    #[test]
    fn test_is_comment_line_false() {
        assert!(!is_comment_line("echo hello"));
        assert!(!is_comment_line(""));
    }

    // ===== IS UPPERCASE VAR =====

    #[test]
    fn test_is_uppercase_var_true() {
        assert!(is_uppercase_var("HOME"));
        assert!(is_uppercase_var("MY_VAR"));
        assert!(is_uppercase_var("FOO_BAR_BAZ"));
    }

    #[test]
    fn test_is_uppercase_var_false() {
        assert!(!is_uppercase_var("home"));
        assert!(!is_uppercase_var("myVar"));
        assert!(!is_uppercase_var("Home"));
    }

    // ===== CASE STATEMENT HELPERS =====

    #[test]
    fn test_is_case_start_true() {
        assert!(is_case_start("case $x in"));
        assert!(is_case_start("  case \"$var\" in"));
    }

    #[test]
    fn test_is_case_start_false() {
        assert!(!is_case_start("esac"));
        assert!(!is_case_start("echo case"));
    }

    #[test]
    fn test_is_case_end_true() {
        assert!(is_case_end("esac"));
        assert!(is_case_end("esac;"));
        assert!(is_case_end("esac "));
    }

    #[test]
    fn test_is_case_end_false() {
        assert!(!is_case_end("case $x in"));
    }

    #[test]
    fn test_case_has_default_true() {
        let block = vec!["case $x in", "a) echo a;;", "*) echo default;;", "esac"];
        assert!(case_has_default(&block));
    }

    #[test]
    fn test_case_has_default_false() {
        let block = vec!["case $x in", "a) echo a;;", "b) echo b;;", "esac"];
        assert!(!case_has_default(&block));
    }

    #[test]
    fn test_is_case_pattern_line() {
        assert!(is_case_pattern_line("a)"));
        assert!(is_case_pattern_line("*)"));
        assert!(is_case_pattern_line("  a | b)"));
        assert!(!is_case_pattern_line("var=value)"));
    }

    // ===== COLLECT CASE STATEMENT VARIABLES =====

    #[test]
    fn test_collect_case_statement_variables_with_default() {
        let source = "case $x in\na) foo=1;;\n*) bar=2;;\nesac";
        let vars = collect_case_statement_variables(source);
        assert!(vars.contains("foo"));
        assert!(vars.contains("bar"));
    }

    #[test]
    fn test_collect_case_statement_variables_no_default() {
        let source = "case $x in\na) foo=1;;\nb) bar=2;;\nesac";
        let vars = collect_case_statement_variables(source);
        // No default branch, so no vars collected
        assert!(vars.is_empty());
    }

    // ===== EXTRACT READ VARIABLES =====

    #[test]
    fn test_extract_read_variables_simple() {
        let vars = extract_read_variables("read foo");
        assert_eq!(vars, vec!["foo"]);
    }

    #[test]
    fn test_extract_read_variables_multiple() {
        let vars = extract_read_variables("read a b c");
        assert_eq!(vars, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_extract_read_variables_with_flags() {
        // Simple case: -r flag without argument
        let vars = extract_read_variables("read -r foo");
        assert_eq!(vars, vec!["foo"]);
    }

    #[test]
    fn test_extract_read_variables_with_prompt() {
        // -p with prompt, then variable (simplified without spaces in prompt)
        let vars = extract_read_variables("read -p prompt foo");
        assert_eq!(vars, vec!["foo"]);
    }

    #[test]
    fn test_extract_read_variables_none() {
        let vars = extract_read_variables("echo hello");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_extract_read_variables_with_array_flag() {
        // -a flag takes array name as next argument
        let vars = extract_read_variables("read -a arr line");
        assert_eq!(vars, vec!["line"]);
    }

    #[test]
    fn test_extract_read_variables_with_delimiter_flag() {
        // -d flag takes delimiter as next argument
        let vars = extract_read_variables("read -d : foo");
        assert_eq!(vars, vec!["foo"]);
    }

    // ===== HAS SOURCE COMMANDS EDGE CASES =====

    #[test]
    fn test_has_source_commands_or_chain() {
        assert!(has_source_commands("test -f x || source default.sh"));
        assert!(has_source_commands("test -f x || . default.sh"));
    }

    // ===== PATTERNS AND VARIABLE INFO =====

    #[test]
    fn test_create_patterns_valid() {
        let p = create_patterns();
        assert!(p.assign.is_match("local foo="));
        assert!(p.use_.is_match("$var"));
        assert!(p.for_loop.is_match("for x in"));
    }

    #[test]
    fn test_collect_variable_info_basic() {
        let p = create_patterns();
        let source = "foo=bar\necho $foo";
        let (assigned, used) = collect_variable_info(source, &p);
        assert!(assigned.contains("foo"));
        assert!(used.iter().any(|(v, _, _)| v == "foo"));
    }

    #[test]
    fn test_collect_variable_info_for_loop() {
        let p = create_patterns();
        let source = "for x in *.txt; do echo $x; done";
        let (assigned, _) = collect_variable_info(source, &p);
        assert!(assigned.contains("x"));
    }

    #[test]
    fn test_collect_variable_info_c_style_for() {
        let p = create_patterns();
        let source = "for ((i=0; i<10; i++)); do echo $i; done";
        let (assigned, _) = collect_variable_info(source, &p);
        assert!(assigned.contains("i"));
    }

    #[test]
    fn test_collect_variable_info_case_expr() {
        let p = create_patterns();
        let source = "case $mode in a) ;; esac";
        let (assigned, _) = collect_variable_info(source, &p);
        assert!(assigned.contains("mode"));
    }

    #[test]
    fn test_collect_variable_info_read_command() {
        let p = create_patterns();
        let source = "read line\necho $line";
        let (assigned, _) = collect_variable_info(source, &p);
        assert!(assigned.contains("line"));
    }

    #[test]
    fn test_collect_variable_info_skips_comments() {
        let p = create_patterns();
        let source = "# foo=bar\necho $foo";
        let (assigned, used) = collect_variable_info(source, &p);
        assert!(!assigned.contains("foo"));
        assert!(used.iter().any(|(v, _, _)| v == "foo"));
    }

    #[test]
    fn test_collect_variable_info_with_source_skips_uppercase() {
        let p = create_patterns();
        let source = "source config.sh\necho $CONFIG_VAR";
        let (_, used) = collect_variable_info(source, &p);
        // CONFIG_VAR is uppercase and script has source - should be skipped
        assert!(!used.iter().any(|(v, _, _)| v == "CONFIG_VAR"));
    }

    // ===== FIND UNDEFINED VARIABLES =====

    #[test]
    fn test_find_undefined_variables_none() {
        let builtins = get_builtins();
        let assigned: HashSet<String> = ["foo".to_string()].into_iter().collect();
        let used = vec![("foo".to_string(), 1, 1)];
        let undef = find_undefined_variables(&assigned, &used, &builtins);
        assert!(undef.is_empty());
    }

    #[test]
    fn test_find_undefined_variables_found() {
        let builtins = get_builtins();
        let assigned: HashSet<String> = HashSet::new();
        let used = vec![("undefined_var".to_string(), 1, 1)];
        let undef = find_undefined_variables(&assigned, &used, &builtins);
        assert_eq!(undef.len(), 1);
        assert_eq!(undef[0].0, "undefined_var");
    }

    #[test]
    fn test_find_undefined_variables_skips_builtins() {
        let builtins = get_builtins();
        let assigned: HashSet<String> = HashSet::new();
        let used = vec![("HOME".to_string(), 1, 1), ("PATH".to_string(), 1, 5)];
        let undef = find_undefined_variables(&assigned, &used, &builtins);
        assert!(undef.is_empty());
    }

    #[test]
    fn test_find_undefined_variables_skips_positional() {
        let builtins = get_builtins();
        let assigned: HashSet<String> = HashSet::new();
        let used = vec![("1".to_string(), 1, 1), ("2".to_string(), 1, 3)];
        let undef = find_undefined_variables(&assigned, &used, &builtins);
        assert!(undef.is_empty());
    }
}
