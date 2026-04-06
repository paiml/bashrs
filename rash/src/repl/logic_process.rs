/// Process purify command and return result
pub fn process_purify_command(line: &str) -> PurifyCommandResult {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return PurifyCommandResult::MissingInput;
    }

    let bash_code = parts.get(1).unwrap_or(&"");

    match purify_bash(bash_code) {
        Ok(result) => PurifyCommandResult::Success(result),
        Err(e) => PurifyCommandResult::Error(e.to_string()),
    }
}

/// Process lint command and return result
pub fn process_lint_command(line: &str) -> LintCommandResult {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return LintCommandResult::MissingInput;
    }

    let bash_code = parts.get(1).unwrap_or(&"");

    match lint_bash(bash_code) {
        Ok(result) => LintCommandResult::Success(format_lint_results(&result)),
        Err(e) => LintCommandResult::Error(e.to_string()),
    }
}

/// Process load command and return result with functions to add
pub fn process_load_command(line: &str) -> (LoadCommandResult, Option<LoadResult>) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return (LoadCommandResult::MissingInput, None);
    }

    let file_path = parts.get(1).unwrap_or(&"");
    let result = load_script(file_path);

    match &result {
        LoadResult::Success(script) => (
            LoadCommandResult::Success {
                path: script.path.clone(),
                function_count: script.functions.len(),
                formatted: result.format(),
            },
            Some(result),
        ),
        LoadResult::FileError(_) | LoadResult::ParseError(_) => {
            (LoadCommandResult::Error(result.format()), None)
        }
    }
}

/// Process source command and return result
pub fn process_source_command(line: &str) -> (LoadCommandResult, Option<LoadResult>) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return (
            LoadCommandResult::Error(
                "Usage: :source <file>\nExample: :source examples/functions.sh".to_string(),
            ),
            None,
        );
    }

    let file_path = parts.get(1).unwrap_or(&"");
    let result = load_script(file_path);

    match &result {
        LoadResult::Success(script) => (
            LoadCommandResult::Success {
                path: script.path.clone(),
                function_count: script.functions.len(),
                formatted: format!(
                    "✓ Sourced: {} ({} functions)",
                    script.path.display(),
                    script.functions.len()
                ),
            },
            Some(result),
        ),
        LoadResult::FileError(_) | LoadResult::ParseError(_) => {
            (LoadCommandResult::Error(result.format()), None)
        }
    }
}

/// Process command based on current mode
pub fn process_command_by_mode(line: &str, state: &ReplState) -> ModeProcessResult {
    use crate::repl::executor::execute_command;

    let expanded_line = expand_variables(line, state.variables());

    match state.mode() {
        ReplMode::Normal => {
            let result = execute_command(&expanded_line);
            ModeProcessResult::Executed(result.format())
        }
        ReplMode::Purify => match purify_bash(&expanded_line) {
            Ok(result) => ModeProcessResult::Purified(result),
            Err(e) => ModeProcessResult::Error(format!("✗ Purification error: {}", e)),
        },
        ReplMode::Lint => match lint_bash(&expanded_line) {
            Ok(result) => ModeProcessResult::Linted(format_lint_results(&result)),
            Err(e) => ModeProcessResult::Error(format!("✗ Lint error: {}", e)),
        },
        ReplMode::Debug => ModeProcessResult::Debug(expanded_line),
        ReplMode::Explain => match explain_bash(line) {
            Some(explanation) => ModeProcessResult::Explained(explanation.format()),
            None => ModeProcessResult::NoExplanation(line.to_string()),
        },
    }
}

/// Process history command
pub fn process_history_command(state: &ReplState) -> HistoryResult {
    let history = state.history();

    if history.is_empty() {
        HistoryResult::Empty
    } else {
        HistoryResult::Entries(history.to_vec())
    }
}

/// Process vars command
pub fn process_vars_command(state: &ReplState) -> VarsResult {
    let variables = state.variables();

    if variables.is_empty() {
        VarsResult::Empty
    } else {
        let mut vars: Vec<_> = variables
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        vars.sort_by(|(a, _), (b, _)| a.cmp(b));
        VarsResult::Variables(vars)
    }
}

/// Process functions command
pub fn process_functions_command(state: &ReplState) -> FunctionsResult {
    let functions = state.loaded_functions();
    FunctionsResult(format_functions(functions))
}

/// Process reload command
pub fn process_reload_command(state: &ReplState) -> (ReloadResult, Option<LoadResult>) {
    if let Some(last_script) = state.last_loaded_script() {
        let path = last_script.clone();
        let result = load_script(&path);

        match &result {
            LoadResult::Success(script) => (
                ReloadResult::Success {
                    path: script.path.clone(),
                    function_count: script.functions.len(),
                },
                Some(result),
            ),
            LoadResult::FileError(_) | LoadResult::ParseError(_) => {
                (ReloadResult::Error(result.format()), None)
            }
        }
    } else {
        (ReloadResult::NoScript, None)
    }
}

/// Get history file path
pub fn get_history_path() -> anyhow::Result<PathBuf> {
    if let Ok(custom_path) = std::env::var("BASHRS_HISTORY_PATH") {
        return Ok(PathBuf::from(custom_path));
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let history_path = PathBuf::from(home).join(".bashrs_history");
    Ok(history_path)
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "logic_tests_process_mode.rs"]
// FIXME(PMAT-238): mod tests_extracted;
