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
