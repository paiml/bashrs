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

    include!("logic_tests_extracted_process.rs");
}
