#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::history::MemHistory;

    // ===== RED PHASE: Unit Tests (These should FAIL initially) =====

    #[test]
    fn test_REPL_006_001_completer_new() {
        let completer = ReplCompleter::new();

        assert_eq!(completer.commands.len(), 11);
        assert_eq!(completer.modes.len(), 5);
        assert!(completer.commands.contains(&"mode".to_string()));
        assert!(completer.commands.contains(&"load".to_string()));
        assert!(completer.commands.contains(&"source".to_string()));
        assert!(completer.modes.contains(&"normal".to_string()));
    }

    #[test]
    fn test_REPL_006_001_complete_command_prefix() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("mo");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, ":mode");
    }

    #[test]
    fn test_REPL_006_001_complete_command_multiple_matches() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("p");

        assert_eq!(completions.len(), 2); // "parse" and "purify"
        let replacements: Vec<_> = completions.iter().map(|p| p.replacement.as_str()).collect();
        assert!(replacements.contains(&":parse"));
        assert!(replacements.contains(&":purify"));
    }

    #[test]
    fn test_REPL_006_001_complete_command_no_match() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("xyz");

        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_REPL_006_001_complete_mode_prefix() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("pur");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "purify");
    }

    #[test]
    fn test_REPL_006_001_complete_mode_multiple_matches() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("l");

        assert_eq!(completions.len(), 1); // Only "lint"
        assert_eq!(completions[0].replacement, "lint");
    }

    #[test]
    fn test_REPL_006_001_complete_mode_empty_shows_all() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("");

        assert_eq!(completions.len(), 5); // All modes
    }

    #[test]
    fn test_REPL_006_001_complete_bash_construct_parameter_expansion() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_bash_construct("${var:");

        assert!(completions.len() >= 4); // At least :-,  :=, :?, :+
    }

    #[test]
    fn test_REPL_006_001_complete_bash_construct_for_loop() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_bash_construct("for");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "for i in");
    }

    #[test]
    fn test_REPL_006_001_complete_full_line_command() {
        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let (start, completions) = completer.complete(":mo", 3, &ctx).unwrap();

        assert_eq!(start, 0);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, ":mode");
    }

    #[test]
    fn test_REPL_006_001_complete_full_line_mode_name() {
        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let (start, completions) = completer.complete(":mode pur", 9, &ctx).unwrap();

        assert_eq!(start, 6); // Start after ":mode "
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "purify");
    }

    #[test]
    fn test_REPL_006_001_complete_bash_in_normal_line() {
        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let (start, completions) = completer.complete("for", 3, &ctx).unwrap();

        assert_eq!(start, 0);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "for i in");
    }

    #[test]
    fn test_REPL_006_001_complete_case_insensitive_command() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("MO");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, ":mode");
    }

    #[test]
    fn test_REPL_006_001_complete_case_insensitive_mode() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("PUR");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "purify");
    }

    #[test]
    fn test_REPL_006_001_default_trait() {
        let completer = ReplCompleter::default();

        assert_eq!(completer.commands.len(), 11);
        assert_eq!(completer.modes.len(), 5);
    }

    // ===== REPL-009-002: File Path Completion Tests =====

    #[test]
    fn test_REPL_009_002_complete_file_path_current_dir() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let completer = ReplCompleter::new();
        let completions = completer.complete_file_path("te");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|p| p.replacement == "test.sh"));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_with_directory() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("examples");
        std::fs::create_dir(&sub_dir).unwrap();
        let test_file = sub_dir.join("script.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        let completer = ReplCompleter::new();
        let path_str = format!("{}/scr", sub_dir.display());
        let completions = completer.complete_file_path(&path_str);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|p| p.display.contains("script.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_directories_first() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir_a");
        let dir2 = temp_dir.path().join("dir_b");
        let file1 = temp_dir.path().join("file_a.sh");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();
        std::fs::File::create(&file1).unwrap();

        let completer = ReplCompleter::new();
        let path_str = format!("{}/", temp_dir.path().display());
        let completions = completer.complete_file_path(&path_str);

        // Directories should come before files
        let dir_positions: Vec<usize> = completions
            .iter()
            .enumerate()
            .filter(|(_, p)| p.display.ends_with('/'))
            .map(|(i, _)| i)
            .collect();

        let file_positions: Vec<usize> = completions
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.display.ends_with('/'))
            .map(|(i, _)| i)
            .collect();

        if !dir_positions.is_empty() && !file_positions.is_empty() {
            assert!(dir_positions.iter().max().unwrap() < file_positions.iter().min().unwrap());
        }
    }

    #[test]
    fn test_REPL_009_002_complete_full_line_load_command() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("example.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let path_str = format!("{}/ex", temp_dir.path().display());
        let line = format!(":load {}", path_str);
        let (start, completions) = completer.complete(&line, line.len(), &ctx).unwrap();

        assert_eq!(start, 6); // Position after ":load "
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|p| p.display.contains("example.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_full_line_source_command() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("script.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let path_str = format!("{}/scr", temp_dir.path().display());
        let line = format!(":source {}", path_str);
        let (start, completions) = completer.complete(&line, line.len(), &ctx).unwrap();

        assert_eq!(start, 8); // Position after ":source "
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|p| p.display.contains("script.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_no_hidden_files() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let hidden_file = temp_dir.path().join(".hidden.sh");
        let visible_file = temp_dir.path().join("visible.sh");
        std::fs::File::create(&hidden_file).unwrap();
        std::fs::File::create(&visible_file).unwrap();

        let completer = ReplCompleter::new();
        let path_str = format!("{}/", temp_dir.path().display());
        let completions = completer.complete_file_path(&path_str);

        // Should not include hidden files unless user explicitly types "."
        assert!(completions.iter().any(|p| p.display.contains("visible.sh")));
        assert!(!completions.iter().any(|p| p.display.contains(".hidden.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_nonexistent_dir() {
        let completer = ReplCompleter::new();
        let completions = completer.complete_file_path("/nonexistent/path/file");

        // Should return empty vector for nonexistent directories
        assert_eq!(completions.len(), 0);
    }

    // ===== REPL-015-002-INT: Syntax Highlighting Integration Tests =====

    /// Test: REPL-015-002-INT-001 - Highlighter integration basic
    #[test]
    fn test_REPL_015_002_INT_001_highlighter_basic() {
        use crate::repl::highlighting::strip_ansi_codes;

        let completer = ReplCompleter::new();

        let input = "echo hello";
        let highlighted = completer.highlight(input, 0);

        // Should contain ANSI codes
        assert!(highlighted.contains("\x1b["));

        // Should preserve original text when stripped
        let stripped = strip_ansi_codes(&highlighted);
        assert_eq!(stripped, input);
    }

    /// Test: REPL-015-002-INT-002 - Highlight with variables
    #[test]
    fn test_REPL_015_002_INT_002_highlight_variables() {
        let completer = ReplCompleter::new();

        let input = "echo $HOME";
        let highlighted = completer.highlight(input, 0);

        // Should highlight 'echo' as command (cyan)
        assert!(highlighted.contains("\x1b[36mecho\x1b[0m"));

        // Should highlight '$HOME' as variable (yellow)
        assert!(highlighted.contains("\x1b[33m$HOME\x1b[0m"));
    }

    /// Test: REPL-015-002-INT-003 - Highlight with keywords
    #[test]
    fn test_REPL_015_002_INT_003_highlight_keywords() {
        let completer = ReplCompleter::new();

        let input = "if [ -f test ]; then echo found; fi";
        let highlighted = completer.highlight(input, 0);

        // Should highlight keywords (blue)
        assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));
    }

    /// Test: REPL-015-002-INT-004 - Highlight multiline input
    #[test]
    fn test_REPL_015_002_INT_004_highlight_multiline() {
        let completer = ReplCompleter::new();

        let input = "for i in 1 2 3\ndo echo $i\ndone";
        let highlighted = completer.highlight(input, 0);

        // Should highlight keywords across lines
        assert!(highlighted.contains("\x1b[1;34mfor\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mdo\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mdone\x1b[0m"));

        // Should highlight variable
        assert!(highlighted.contains("\x1b[33m$i\x1b[0m"));
    }

    /// Test: REPL-015-002-INT-005 - Empty input
    #[test]
    fn test_REPL_015_002_INT_005_empty_input() {
        let completer = ReplCompleter::new();

        let highlighted = completer.highlight("", 0);

        // Should handle empty input gracefully
        assert_eq!(highlighted.as_ref(), "");
    }

    /// Test: REPL-015-002-INT-006 - Special characters
    #[test]
    fn test_REPL_015_002_INT_006_special_characters() {
        let completer = ReplCompleter::new();

        let input = "echo \"test\" | grep 'pattern' && exit 0";
        let highlighted = completer.highlight(input, 0);

        // Should highlight strings (green)
        assert!(highlighted.contains("\x1b[32m\"test\"\x1b[0m"));
        assert!(highlighted.contains("\x1b[32m'pattern'\x1b[0m"));

        // Should highlight operators (magenta)
        assert!(highlighted.contains("\x1b[35m|\x1b[0m"));
        assert!(highlighted.contains("\x1b[35m&&\x1b[0m"));
    }
}
