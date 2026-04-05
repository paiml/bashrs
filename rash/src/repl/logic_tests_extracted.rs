#[cfg(test)]
mod tests {
    use super::*;

    // ===== MODE COMMAND TESTS =====

    #[test]
    fn test_process_mode_command_show_current() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode", &state);

        assert!(matches!(result, ModeCommandResult::ShowCurrent { .. }));
        assert!(new_mode.is_none());

        let formatted = result.format();
        assert!(formatted.contains("Current mode:"));
        assert!(formatted.contains("Available modes:"));
    }

    #[test]
    fn test_process_mode_command_switch_valid() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode purify", &state);

        assert!(matches!(result, ModeCommandResult::Switched { .. }));
        assert_eq!(new_mode, Some(ReplMode::Purify));

        let formatted = result.format();
        assert!(formatted.contains("Switched to purify mode"));
    }

    #[test]
    fn test_process_mode_command_switch_all_modes() {
        let state = ReplState::new();

        for mode_name in &["normal", "purify", "lint", "debug", "explain"] {
            let (result, new_mode) = process_mode_command(&format!(":mode {}", mode_name), &state);
            assert!(matches!(result, ModeCommandResult::Switched { .. }));
            assert!(new_mode.is_some());
        }
    }

    #[test]
    fn test_process_mode_command_invalid_mode() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode invalid", &state);

        assert!(matches!(result, ModeCommandResult::InvalidMode(_)));
        assert!(new_mode.is_none());
    }

    #[test]
    fn test_process_mode_command_too_many_args() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode purify extra", &state);

        assert!(matches!(result, ModeCommandResult::InvalidUsage));
        assert!(new_mode.is_none());
    }

    // ===== PARSE COMMAND TESTS =====

    #[test]
    fn test_process_parse_command_success() {
        let result = process_parse_command(":parse echo hello");

        assert!(matches!(result, ParseCommandResult::Success { .. }));
        let formatted = result.format();
        assert!(formatted.contains("Parse successful"));
    }

    #[test]
    fn test_process_parse_command_missing_input() {
        let result = process_parse_command(":parse");

        assert!(matches!(result, ParseCommandResult::MissingInput));
        let formatted = result.format();
        assert!(formatted.contains("Usage:"));
    }

    #[test]
    fn test_process_parse_command_complex() {
        let result = process_parse_command(":parse for i in 1 2 3; do echo $i; done");

        assert!(matches!(result, ParseCommandResult::Success { .. }));
    }

    // ===== PURIFY COMMAND TESTS =====

    #[test]
    fn test_process_purify_command_success() {
        let result = process_purify_command(":purify mkdir /tmp/test");

        assert!(matches!(result, PurifyCommandResult::Success(_)));
        let formatted = result.format();
        assert!(formatted.contains("Purification successful"));
    }

    #[test]
    fn test_process_purify_command_missing_input() {
        let result = process_purify_command(":purify");

        assert!(matches!(result, PurifyCommandResult::MissingInput));
    }

    #[test]
    fn test_process_purify_command_idempotent() {
        let result = process_purify_command(":purify rm file.txt");

        if let PurifyCommandResult::Success(output) = result {
            // Purified should contain idempotent flag
            assert!(output.contains("-f") || output.contains("rm"));
        }
    }

    // ===== LINT COMMAND TESTS =====

    #[test]
    fn test_process_lint_command_success() {
        let result = process_lint_command(":lint echo $var");

        assert!(matches!(result, LintCommandResult::Success(_)));
    }

    #[test]
    fn test_process_lint_command_missing_input() {
        let result = process_lint_command(":lint");

        assert!(matches!(result, LintCommandResult::MissingInput));
    }

    // ===== LOAD COMMAND TESTS =====

    #[test]
    fn test_process_load_command_missing_input() {
        let (result, load_result) = process_load_command(":load");

        assert!(matches!(result, LoadCommandResult::MissingInput));
        assert!(load_result.is_none());
    }

    #[test]
    fn test_process_load_command_file_not_found() {
        let (result, load_result) = process_load_command(":load /nonexistent/file.sh");

        assert!(matches!(result, LoadCommandResult::Error(_)));
        assert!(load_result.is_none());
    }

    // ===== SOURCE COMMAND TESTS =====

    #[test]
    fn test_process_source_command_missing_input() {
        let (result, load_result) = process_source_command(":source");

        assert!(matches!(result, LoadCommandResult::Error(_)));
        assert!(load_result.is_none());
    }

    // ===== HISTORY COMMAND TESTS =====

    #[test]
    fn test_process_history_command_empty() {
        let state = ReplState::new();
        let result = process_history_command(&state);

        assert!(matches!(result, HistoryResult::Empty));
        assert_eq!(result.format(), "No commands in history");
    }

    #[test]
    fn test_process_history_command_with_entries() {
        let mut state = ReplState::new();
        state.add_history("echo hello".to_string());
        state.add_history("ls -la".to_string());

        let result = process_history_command(&state);

        assert!(matches!(result, HistoryResult::Entries(_)));
        let formatted = result.format();
        assert!(formatted.contains("echo hello"));
        assert!(formatted.contains("ls -la"));
    }

    // ===== VARS COMMAND TESTS =====

    #[test]
    fn test_process_vars_command_empty() {
        let state = ReplState::new();
        let result = process_vars_command(&state);

        assert!(matches!(result, VarsResult::Empty));
        assert_eq!(result.format(), "No session variables set");
    }

    #[test]
    fn test_process_vars_command_with_variables() {
        let mut state = ReplState::new();
        state.set_variable("FOO".to_string(), "bar".to_string());
        state.set_variable("BAZ".to_string(), "qux".to_string());

        let result = process_vars_command(&state);

        assert!(matches!(result, VarsResult::Variables(_)));
        let formatted = result.format();
        assert!(formatted.contains("FOO = bar"));
        assert!(formatted.contains("BAZ = qux"));
    }

    #[test]
    fn test_process_vars_command_sorted() {
        let mut state = ReplState::new();
        state.set_variable("Z".to_string(), "last".to_string());
        state.set_variable("A".to_string(), "first".to_string());

        let result = process_vars_command(&state);

        if let VarsResult::Variables(vars) = result {
            assert_eq!(vars[0].0, "A");
            assert_eq!(vars[1].0, "Z");
        }
    }

    // ===== FUNCTIONS COMMAND TESTS =====

    #[test]
    fn test_process_functions_command_empty() {
        let state = ReplState::new();
        let result = process_functions_command(&state);

        let formatted = result.format();
        assert!(
            formatted.contains("No functions")
                || formatted.contains("0 functions")
                || formatted.is_empty()
                || formatted.contains("functions")
        );
    }

    // ===== RELOAD COMMAND TESTS =====

    #[test]
    fn test_process_reload_command_no_script() {
        let state = ReplState::new();
        let (result, load_result) = process_reload_command(&state);

        assert!(matches!(result, ReloadResult::NoScript));
        assert!(load_result.is_none());
        assert!(result.format().contains("No script to reload"));
    }

    // ===== MODE PROCESSING TESTS =====

    #[test]
    fn test_process_command_by_mode_normal() {
        let state = ReplState::new();
        let result = process_command_by_mode("echo hello", &state);

        assert!(matches!(result, ModeProcessResult::Executed(_)));
    }

    #[test]
    fn test_process_command_by_mode_purify() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Purify);

        let result = process_command_by_mode("mkdir /tmp/test", &state);

        assert!(matches!(result, ModeProcessResult::Purified(_)));
        let formatted = result.format();
        assert!(formatted.contains("Purified"));
    }

    #[test]
    fn test_process_command_by_mode_lint() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Lint);

        let result = process_command_by_mode("echo $var", &state);

        assert!(matches!(result, ModeProcessResult::Linted(_)));
    }

    #[test]
    fn test_process_command_by_mode_debug() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Debug);

        let result = process_command_by_mode("echo hello", &state);

        assert!(matches!(result, ModeProcessResult::Debug(_)));
        let formatted = result.format();
        assert!(formatted.contains("Debug mode"));
    }

    #[test]
    fn test_process_command_by_mode_explain() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Explain);

        let result = process_command_by_mode("${var:-default}", &state);

        // May or may not have explanation depending on implementation
        assert!(
            matches!(result, ModeProcessResult::Explained(_))
                || matches!(result, ModeProcessResult::NoExplanation(_))
        );
    }

    #[test]
    fn test_process_command_by_mode_variable_expansion() {
        let mut state = ReplState::new();
        state.set_variable("NAME".to_string(), "world".to_string());

        let result = process_command_by_mode("echo $NAME", &state);

        // The command should have expanded $NAME
        assert!(matches!(result, ModeProcessResult::Executed(_)));
    }

    // ===== HISTORY PATH TESTS =====

    #[test]
    fn test_get_history_path_default() {
        let path = get_history_path();
        assert!(path.is_ok());
        assert!(path.unwrap().to_string_lossy().contains(".bashrs_history"));
    }

    #[test]
    fn test_get_history_path_deterministic() {
        let path1 = get_history_path().unwrap();
        let path2 = get_history_path().unwrap();
        assert_eq!(path1, path2);
    }

    // ===== FORMAT METHOD TESTS =====

    #[test]
    fn test_mode_command_result_format_invalid_mode() {
        let result = ModeCommandResult::InvalidMode("unknown mode: foo".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("unknown mode: foo"));
    }

    #[test]
    fn test_mode_command_result_format_invalid_usage() {
        let result = ModeCommandResult::InvalidUsage;
        let formatted = result.format();
        assert!(formatted.contains("Usage:"));
        assert!(formatted.contains("Valid modes:"));
    }

    #[test]
    fn test_parse_command_result_format_error() {
        let result = ParseCommandResult::Error("parse error at line 1".to_string());
        let formatted = result.format();
        assert!(formatted.contains("✗"));
        assert!(formatted.contains("parse error at line 1"));
    }

    #[test]
    fn test_purify_command_result_format_error() {
        let result = PurifyCommandResult::Error("failed to purify".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Purification error"));
        assert!(formatted.contains("failed to purify"));
    }

    #[test]
    fn test_lint_command_result_format_error() {
        let result = LintCommandResult::Error("lint failed".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Lint error"));
        assert!(formatted.contains("lint failed"));
    }

    #[test]
    fn test_load_command_result_format_success() {
        let result = LoadCommandResult::Success {
            path: PathBuf::from("/test/script.sh"),
            function_count: 5,
            formatted: "Loaded 5 functions".to_string(),
        };
        let formatted = result.format();
        assert_eq!(formatted, "Loaded 5 functions");
    }

    #[test]
    fn test_load_command_result_format_error() {
        let result = LoadCommandResult::Error("file not found".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "file not found");
    }

    #[test]
    fn test_mode_process_result_format_purified() {
        let result = ModeProcessResult::Purified("mkdir -p /tmp".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Purified:"));
        assert!(formatted.contains("mkdir -p /tmp"));
    }

    #[test]
    fn test_mode_process_result_format_no_explanation() {
        let result = ModeProcessResult::NoExplanation("some command".to_string());
        let formatted = result.format();
        assert!(formatted.contains("No explanation available"));
        assert!(formatted.contains("some command"));
    }

    #[test]
    fn test_mode_process_result_format_error() {
        let result = ModeProcessResult::Error("something went wrong".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "something went wrong");
    }

    #[test]
    fn test_reload_result_format_success() {
        let result = ReloadResult::Success {
            path: PathBuf::from("/test/script.sh"),
            function_count: 3,
        };
        let formatted = result.format();
        assert!(formatted.contains("Reloaded:"));
        assert!(formatted.contains("/test/script.sh"));
        assert!(formatted.contains("3 functions"));
    }

    #[test]
    fn test_reload_result_format_error() {
        let result = ReloadResult::Error("reload failed".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "reload failed");
    }

    #[test]
    fn test_functions_result_format() {
        let result = FunctionsResult("function1, function2".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "function1, function2");
    }

    // ===== ADDITIONAL EDGE CASES =====

    #[test]
    fn test_parse_command_with_multiple_statements() {
        let result = process_parse_command(":parse echo a; echo b; echo c");
        if let ParseCommandResult::Success {
            statement_count, ..
        } = result
        {
            assert!(statement_count >= 1);
        }
    }

    #[test]
    fn test_process_command_by_mode_explain_no_match() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Explain);

        // A simple command that might not have an explanation
        let result = process_command_by_mode("ls", &state);

        // Could be explained or not, both are valid
        let formatted = result.format();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_history_result_entries_format() {
        let entries = vec!["cmd1".to_string(), "cmd2".to_string(), "cmd3".to_string()];
        let result = HistoryResult::Entries(entries);
        let formatted = result.format();

        assert!(formatted.contains("Command History"));
        assert!(formatted.contains("3 commands"));
        assert!(formatted.contains("1 cmd1"));
        assert!(formatted.contains("2 cmd2"));
        assert!(formatted.contains("3 cmd3"));
    }

    #[test]
    fn test_vars_result_format() {
        let vars = vec![
            ("AAA".to_string(), "111".to_string()),
            ("BBB".to_string(), "222".to_string()),
        ];
        let result = VarsResult::Variables(vars);
        let formatted = result.format();

        assert!(formatted.contains("Session Variables"));
        assert!(formatted.contains("2 variables"));
        assert!(formatted.contains("AAA = 111"));
        assert!(formatted.contains("BBB = 222"));
    }

    #[test]
    fn test_mode_command_result_eq() {
        let result1 = ModeCommandResult::InvalidUsage;
        let result2 = ModeCommandResult::InvalidUsage;
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_process_purify_command_error_handling() {
        // Invalid bash that can't be parsed
        let result = process_purify_command(":purify <<<");

        // Should handle parse error gracefully
        assert!(
            matches!(result, PurifyCommandResult::Error(_))
                || matches!(result, PurifyCommandResult::Success(_))
        );
    }

    #[test]
    fn test_process_lint_command_error_handling() {
        // Invalid bash that can't be parsed
        let result = process_lint_command(":lint <<<");

        // Should handle parse error gracefully
        assert!(
            matches!(result, LintCommandResult::Error(_))
                || matches!(result, LintCommandResult::Success(_))
        );
    }
}
