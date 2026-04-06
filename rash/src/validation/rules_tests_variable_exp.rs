use super::*;

// ===== VariableExpansion tests =====

#[test]
fn test_variable_expansion_quoted() {
    let exp = VariableExpansion::Quoted("var".to_string());
    assert!(exp.validate().is_ok());
}

#[test]
fn test_variable_expansion_unquoted() {
    let exp = VariableExpansion::Unquoted("var".to_string());
    let result = exp.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2086");
}

#[test]
fn test_variable_expansion_word_split() {
    let exp = VariableExpansion::WordSplit("var".to_string());
    assert!(exp.validate().is_ok());
}

#[test]
fn test_variable_expansion_array() {
    let exp = VariableExpansion::ArrayExpansion("arr".to_string());
    assert!(exp.validate().is_ok());
}

// ===== CommandSubstitution tests =====

#[test]
fn test_command_substitution_assignment() {
    let cmd = CommandSubstitution {
        command: "ls".to_string(),
        context: SubstitutionContext::Assignment,
    };
    assert!(cmd.validate().is_ok());
}

#[test]
fn test_command_substitution_array_init() {
    let cmd = CommandSubstitution {
        command: "find .".to_string(),
        context: SubstitutionContext::ArrayInit,
    };
    assert!(cmd.validate().is_ok());
}

#[test]
fn test_command_substitution_quoted() {
    let cmd = CommandSubstitution {
        command: "pwd".to_string(),
        context: SubstitutionContext::Quoted,
    };
    assert!(cmd.validate().is_ok());
}

#[test]
fn test_command_substitution_unquoted() {
    let cmd = CommandSubstitution {
        command: "date".to_string(),
        context: SubstitutionContext::Unquoted,
    };
    let result = cmd.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2046");
}

// ===== validate_glob_pattern tests =====

#[test]
fn test_glob_pattern_normal() {
    let result = validate_glob_pattern("*.txt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "*.txt");
}

#[test]
fn test_glob_pattern_with_question() {
    let result = validate_glob_pattern("file?.txt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "file?.txt");
}

#[test]
fn test_glob_pattern_with_brackets() {
    let result = validate_glob_pattern("[abc].txt");
    assert!(result.is_ok());
}

#[test]
fn test_glob_pattern_starts_with_dash() {
    let result = validate_glob_pattern("-rf");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2035");
}

#[test]
fn test_glob_pattern_no_glob_chars() {
    let result = validate_glob_pattern("file.txt");
    assert!(result.is_ok());
}

// ===== CommandSequence tests =====

#[test]
fn test_command_sequence_valid() {
    let seq = CommandSequence {
        commands: vec!["cmd1".to_string(), "cmd2".to_string()],
        exit_code_checks: vec![
            ExitCodeCheck { command_index: 0 },
            ExitCodeCheck { command_index: 1 },
        ],
    };
    assert!(seq.validate().is_ok());
}

#[test]
fn test_command_sequence_invalid() {
    let seq = CommandSequence {
        commands: vec!["cmd1".to_string(), "cmd2".to_string()],
        exit_code_checks: vec![
            ExitCodeCheck { command_index: 0 },
            ExitCodeCheck { command_index: 0 }, // Wrong index
        ],
    };
    let result = seq.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2181");
}

#[test]
fn test_command_sequence_empty() {
    let seq = CommandSequence {
        commands: vec![],
        exit_code_checks: vec![],
    };
    assert!(seq.validate().is_ok());
}

// ===== validate_backticks tests =====

#[test]
fn test_backticks_present() {
    let result = validate_backticks("echo `date`");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2006");
}

#[test]
fn test_backticks_absent() {
    let result = validate_backticks("echo $(date)");
    assert!(result.is_ok());
}

// ===== validate_cd_usage tests =====

#[test]
fn test_cd_without_error_handling() {
    let result = validate_cd_usage("cd /tmp");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2164");
}

#[test]
fn test_cd_with_error_handling() {
    let result = validate_cd_usage("cd /tmp || exit 1");
    assert!(result.is_ok());
}

#[test]
fn test_not_cd_command() {
    let result = validate_cd_usage("ls /tmp");
    assert!(result.is_ok());
}

// ===== validate_read_command tests =====

#[test]
fn test_read_without_r() {
    let result = validate_read_command("read var");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2162");
}

#[test]
fn test_read_with_r() {
    let result = validate_read_command("read -r var");
    assert!(result.is_ok());
}

#[test]
fn test_not_read_command() {
    let result = validate_read_command("echo hello");
    assert!(result.is_ok());
}

// ===== validate_unicode_quotes tests =====

#[test]
fn test_unicode_left_double_quote() {
    let result = validate_unicode_quotes("echo \u{201c}hello\u{201d}");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2220");
}

#[test]
fn test_unicode_single_quote() {
    let result = validate_unicode_quotes("echo \u{2018}hello\u{2019}");
    assert!(result.is_err());
}

#[test]
fn test_ascii_quotes() {
    let result = validate_unicode_quotes("echo \"hello\"");
    assert!(result.is_ok());
}

// ===== validate_all tests =====

#[test]
fn test_validate_all_valid() {
    let result = validate_all("echo $(date)");
    assert!(result.is_ok());
}

#[test]
fn test_validate_all_backticks() {
    let result = validate_all("echo `date`");
    assert!(result.is_err());
}

#[test]
fn test_validate_all_cd() {
    let result = validate_all("cd /tmp");
    assert!(result.is_err());
}

#[test]
fn test_validate_all_read() {
    let result = validate_all("read var");
    assert!(result.is_err());
}

#[test]
fn test_validate_all_unicode() {
    let result = validate_all("echo \u{201c}hi\u{201d}");
    assert!(result.is_err());
}

// ===== ConditionalExpression tests =====

#[test]
fn test_conditional_string_comparison_quoted() {
    let expr = ConditionalExpression::StringComparison {
        left: Box::new(ShellExpression::Variable("x".to_string(), true)),
        op: ComparisonOp::Eq,
        right: Box::new(ShellExpression::Variable("y".to_string(), true)),
    };
    assert!(expr.validate().is_ok());
}

#[test]
fn test_conditional_string_comparison_unquoted() {
    let expr = ConditionalExpression::StringComparison {
        left: Box::new(ShellExpression::Variable("x".to_string(), false)),
        op: ComparisonOp::Eq,
        right: Box::new(ShellExpression::Variable("y".to_string(), true)),
    };
    let result = expr.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2086");
}

#[test]
fn test_conditional_file_test_quoted() {
    let expr = ConditionalExpression::FileTest {
        op: FileTestOp::Exists,
        path: Box::new(ShellExpression::Variable("path".to_string(), true)),
    };
    assert!(expr.validate().is_ok());
}

#[test]
fn test_conditional_file_test_unquoted() {
    let expr = ConditionalExpression::FileTest {
        op: FileTestOp::IsFile,
        path: Box::new(ShellExpression::Variable("path".to_string(), false)),
    };
    let result = expr.validate();
    assert!(result.is_err());
}

// ===== Clone/Debug trait tests =====

#[test]
fn test_variable_expansion_clone() {
    let exp = VariableExpansion::Quoted("var".to_string());
    let cloned = exp.clone();
    matches!(cloned, VariableExpansion::Quoted(_));
}

#[test]
fn test_command_substitution_clone() {
    let cmd = CommandSubstitution {
        command: "ls".to_string(),
        context: SubstitutionContext::Assignment,
    };
    let cloned = cmd.clone();
    assert_eq!(cloned.command, "ls");
}

#[test]
fn test_comparison_op_clone() {
    let ops = [
        ComparisonOp::Eq,
        ComparisonOp::Ne,
        ComparisonOp::Lt,
        ComparisonOp::Gt,
        ComparisonOp::Le,
        ComparisonOp::Ge,
    ];
    for op in ops {
        let _ = op.clone();
    }
}

#[test]
fn test_file_test_op_clone() {
    let ops = [
        FileTestOp::Exists,
        FileTestOp::IsFile,
        FileTestOp::IsDir,
        FileTestOp::IsReadable,
        FileTestOp::IsWritable,
        FileTestOp::IsExecutable,
    ];
    for op in ops {
        let _ = op.clone();
    }
}

#[test]
fn test_substitution_context_clone() {
    let contexts = [
        SubstitutionContext::Assignment,
        SubstitutionContext::ArrayInit,
        SubstitutionContext::Quoted,
        SubstitutionContext::Unquoted,
    ];
    for ctx in contexts {
        let _ = ctx.clone();
    }
}

#[test]
fn test_variable_expansion_debug() {
    let exp = VariableExpansion::Quoted("var".to_string());
    let debug = format!("{:?}", exp);
    assert!(debug.contains("Quoted"));
}
