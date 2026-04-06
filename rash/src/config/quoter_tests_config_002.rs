use super::*;

#[test]
fn test_config_002_detect_simple_unquoted_var() {
    // ARRANGE
    let line = "export PROJECT_DIR=$HOME/my projects";

    // ACT
    let variables = analyze_unquoted_variables(line);

    // ASSERT
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0].variable, "$HOME");
    assert_eq!(variables[0].line, 1);
}

#[test]
fn test_config_002_detect_unquoted_in_cd() {
    // ARRANGE
    let source = "cd $PROJECT_DIR";

    // ACT
    let variables = analyze_unquoted_variables(source);

    // ASSERT
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0].variable, "$PROJECT_DIR");
}

#[test]
fn test_config_002_ignore_already_quoted() {
    // ARRANGE
    let source = r#"export DIR="${HOME}/projects""#;

    // ACT
    let variables = analyze_unquoted_variables(source);

    // ASSERT
    assert_eq!(
        variables.len(),
        0,
        "Should not flag already quoted variables"
    );
}

#[test]
fn test_config_002_ignore_comments() {
    // ARRANGE
    let source = "# export DIR=$HOME/projects";

    // ACT
    let variables = analyze_unquoted_variables(source);

    // ASSERT
    assert_eq!(variables.len(), 0, "Should ignore variables in comments");
}

#[test]
fn test_config_002_detect_multiple_on_same_line() {
    // ARRANGE
    let source = "cp $SOURCE $DEST";

    // ACT
    let variables = analyze_unquoted_variables(source);

    // ASSERT
    assert_eq!(variables.len(), 2);
    assert_eq!(variables[0].variable, "$SOURCE");
    assert_eq!(variables[1].variable, "$DEST");
}

#[test]
fn test_config_002_detect_command_substitution() {
    // ARRANGE
    let source = "FILES=$(ls *.txt)";

    // ACT
    let variables = analyze_unquoted_variables(source);

    // ASSERT
    // Note: This test is about the variable assignment, not the command substitution
    // The value side should be quoted: FILES="$(ls *.txt)"
    assert_eq!(
        variables.len(),
        0,
        "Command substitution on RHS is OK in assignment"
    );
}

#[test]
fn test_config_002_generate_issues() {
    // ARRANGE
    let source = r#"export PROJECT_DIR=$HOME/my projects
cd $PROJECT_DIR"#;

    let variables = analyze_unquoted_variables(source);

    // ACT
    let issues = detect_unquoted_variables(&variables);

    // ASSERT
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].rule_id, "CONFIG-002");
    assert_eq!(issues[0].severity, Severity::Warning);
    assert!(issues[0].message.contains("word splitting"));
}

#[test]
fn test_config_002_quote_simple_variable() {
    // ARRANGE
    let source = "export DIR=$HOME/projects";

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, r#"export DIR="${HOME}/projects""#);
}

#[test]
fn test_config_002_quote_multiple_variables() {
    // ARRANGE
    let source = "cp $SOURCE $DEST";

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, r#"cp "${SOURCE}" "${DEST}""#);
}

#[test]
fn test_config_002_preserve_already_quoted() {
    // ARRANGE
    let source = r#"export DIR="${HOME}/projects"
echo "Hello $USER""#;

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, source, "Should not change already quoted variables");
}

#[test]
fn test_config_002_preserve_comments() {
    // ARRANGE
    let source = r#"# My config
export DIR=$HOME/projects
# End"#;

    let expected = r#"# My config
export DIR="${HOME}/projects"
# End"#;

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, expected);
}

#[test]
fn test_config_002_handle_braced_variables() {
    // ARRANGE
    let source = "export DIR=${HOME}/projects";

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, r#"export DIR="${HOME}/projects""#);
}

#[test]
fn test_config_002_real_world_example() {
    // ARRANGE
    let source = r#"export PROJECT_DIR=$HOME/my projects
export BACKUP_DIR=$HOME/backups
cd $PROJECT_DIR
cp $PROJECT_DIR/file.txt $BACKUP_DIR/"#;

    // Note: Currently quotes each variable individually (safe but verbose)
    // TODO v7.1: Optimize to quote entire arguments
    let expected = r#"export PROJECT_DIR="${HOME}/my projects"
export BACKUP_DIR="${HOME}/backups"
cd "${PROJECT_DIR}"
cp "${PROJECT_DIR}"/file.txt "${BACKUP_DIR}"/"#;

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, expected);
}

#[test]
fn test_config_002_idempotent() {
    // ARRANGE
    let source = "export DIR=$HOME/projects";

    // ACT
    let quoted_once = quote_variables(source);
    let quoted_twice = quote_variables(&quoted_once);

    // ASSERT
    assert_eq!(quoted_once, quoted_twice, "Quoting should be idempotent");
}

#[test]
fn test_config_002_debug_add_braces() {
    // Test the add_braces_to_variables function directly
    let input = "$HOME/projects";
    let result = add_braces_to_variables(input);
    assert_eq!(
        result, "${HOME}/projects",
        "add_braces_to_variables should convert $HOME to ${{HOME}}"
    );
}

#[test]
fn test_config_002_debug_quote_assignment() {
    // Test quote_assignment_line directly
    let input = "export DIR=$HOME/projects";
    let result = quote_assignment_line(input);
    println!("Input: {}", input);
    println!("Result: {}", result);
    assert_eq!(result, r#"export DIR="${HOME}/projects""#);
}

#[test]
fn test_config_002_empty_input() {
    // ARRANGE
    let source = "";

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, "");
}

#[test]
fn test_config_002_no_variables() {
    // ARRANGE
    let source = r#"export EDITOR="vim"
alias ll='ls -la'"#;

    // ACT
    let result = quote_variables(source);

    // ASSERT
    assert_eq!(result, source, "Should not change lines without variables");
}
