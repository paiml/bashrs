//! Feature extraction for ML model.
//!
//! 64 features carefully chosen to capture error patterns:
//! - Numeric features normalized to [0, 1]
//! - Categorical features one-hot encoded
//! - Text features converted to bag-of-words indicators
#![allow(clippy::indexing_slicing)] // Test assertions use direct indexing for clarity

/// Feature vector for ML model (64 features).
#[derive(Debug, Clone)]
pub struct ErrorFeatures {
    /// Feature vector (always 64 elements).
    pub features: Vec<f32>,
}

impl ErrorFeatures {
    /// Feature vector size.
    pub const SIZE: usize = 64;

    /// Extract features from error message and context.
    #[must_use]
    pub fn extract(exit_code: i32, stderr: &str, command: Option<&str>) -> Self {
        let mut features = Vec::with_capacity(Self::SIZE);

        // === Exit code features (0-5) ===
        features.push(exit_code as f32 / 255.0); // Normalized exit code
        features.push(if exit_code == 1 { 1.0 } else { 0.0 }); // General error
        features.push(if exit_code == 2 { 1.0 } else { 0.0 }); // Misuse
        features.push(if exit_code == 126 { 1.0 } else { 0.0 }); // Permission denied
        features.push(if exit_code == 127 { 1.0 } else { 0.0 }); // Command not found
        features.push(if exit_code == 128 { 1.0 } else { 0.0 }); // Signal base

        // === Signal features (6-9) ===
        features.push(if exit_code == 130 { 1.0 } else { 0.0 }); // SIGINT (Ctrl+C)
        features.push(if exit_code == 137 { 1.0 } else { 0.0 }); // SIGKILL
        features.push(if exit_code == 141 { 1.0 } else { 0.0 }); // SIGPIPE
        features.push(if exit_code == 143 { 1.0 } else { 0.0 }); // SIGTERM

        // === Error message length features (10-11) ===
        let stderr_len = stderr.len();
        features.push((stderr_len as f32 / 1000.0).min(1.0)); // Normalized length
        features.push((stderr.lines().count() as f32 / 10.0).min(1.0)); // Line count

        // === Keyword indicators - bag of words (12-31) ===
        let stderr_lower = stderr.to_lowercase();

        // File/path related
        features.push(if stderr_lower.contains("not found") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("no such file") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("permission denied") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("is a directory") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("not a directory") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("too many open") {
            1.0
        } else {
            0.0
        });

        // Syntax related
        features.push(if stderr_lower.contains("syntax error") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("unexpected") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("unmatched") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("unterminated") {
            1.0
        } else {
            0.0
        });

        // Variable related
        features.push(if stderr_lower.contains("unbound variable") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("bad substitution") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("readonly") {
            1.0
        } else {
            0.0
        });

        // Command related
        features.push(if stderr_lower.contains("command not found") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("invalid option") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("missing") {
            1.0
        } else {
            0.0
        });

        // Process/pipe related
        features.push(if stderr_lower.contains("broken pipe") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("killed") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("timeout") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("timed out") {
            1.0
        } else {
            0.0
        });

        // === Quote/bracket analysis (32-37) ===
        let single_quotes = stderr.matches('\'').count();
        let double_quotes = stderr.matches('"').count();
        let parens = stderr.matches('(').count() + stderr.matches(')').count();
        let brackets = stderr.matches('[').count() + stderr.matches(']').count();
        let braces = stderr.matches('{').count() + stderr.matches('}').count();

        features.push((single_quotes as f32 / 10.0).min(1.0));
        features.push((double_quotes as f32 / 10.0).min(1.0));
        features.push(if !single_quotes.is_multiple_of(2) {
            1.0
        } else {
            0.0
        }); // Odd = mismatch
        features.push(if !double_quotes.is_multiple_of(2) {
            1.0
        } else {
            0.0
        }); // Odd = mismatch
        features.push(((parens + brackets + braces) as f32 / 20.0).min(1.0));
        features.push(if !(parens + brackets + braces).is_multiple_of(2) {
            1.0
        } else {
            0.0
        }); // Odd = mismatch

        // === Line position features (38-41) ===
        let has_line_num = stderr_lower.contains("line ");
        let has_column = stderr_lower.contains("column ") || stderr_lower.contains("col ");
        features.push(if has_line_num { 1.0 } else { 0.0 });
        features.push(if has_column { 1.0 } else { 0.0 });
        features.push(if stderr_lower.contains("near") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("expected") {
            1.0
        } else {
            0.0
        });

        // === Command features (42-49) ===
        if let Some(cmd) = command {
            let cmd_len = cmd.len();
            features.push((cmd_len as f32 / 100.0).min(1.0)); // Command length
            features.push(if cmd.contains('|') { 1.0 } else { 0.0 }); // Pipeline
            features.push(if cmd.contains('>') { 1.0 } else { 0.0 }); // Output redirect
            features.push(if cmd.contains('<') { 1.0 } else { 0.0 }); // Input redirect
            features.push(if cmd.contains("2>") { 1.0 } else { 0.0 }); // Stderr redirect
            features.push(if cmd.starts_with("sudo") { 1.0 } else { 0.0 }); // Sudo
            features.push(if cmd.contains("&&") || cmd.contains("||") {
                1.0
            } else {
                0.0
            }); // Compound
            features.push(if cmd.contains('$') { 1.0 } else { 0.0 }); // Variables
        } else {
            features.extend([0.0; 8]);
        }

        // === Shell-specific keywords (50-57) ===
        features.push(if stderr_lower.contains("bash:") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("sh:") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("zsh:") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("dash:") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("ksh:") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("fish:") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("cannot") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("failed") {
            1.0
        } else {
            0.0
        });

        // === Additional error indicators (58-63) ===
        features.push(if stderr_lower.contains("error") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("warning") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("fatal") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("abort") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("segmentation") {
            1.0
        } else {
            0.0
        });
        features.push(if stderr_lower.contains("core dump") {
            1.0
        } else {
            0.0
        });

        // Ensure exactly 64 features
        debug_assert_eq!(features.len(), Self::SIZE, "Feature count mismatch");

        Self { features }
    }

    /// Convert to slice for ML model.
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.features
    }

    /// Get feature by index with name for debugging.
    #[must_use]
    pub fn feature_name(index: usize) -> &'static str {
        match index {
            0 => "exit_code_normalized",
            1 => "exit_code_is_1",
            2 => "exit_code_is_2",
            3 => "exit_code_is_126",
            4 => "exit_code_is_127",
            5 => "exit_code_is_128",
            6 => "signal_sigint",
            7 => "signal_sigkill",
            8 => "signal_sigpipe",
            9 => "signal_sigterm",
            10 => "stderr_length",
            11 => "stderr_line_count",
            12 => "kw_not_found",
            13 => "kw_no_such_file",
            14 => "kw_permission_denied",
            15 => "kw_is_directory",
            16 => "kw_not_directory",
            17 => "kw_too_many_open",
            18 => "kw_syntax_error",
            19 => "kw_unexpected",
            20 => "kw_unmatched",
            21 => "kw_unterminated",
            22 => "kw_unbound_variable",
            23 => "kw_bad_substitution",
            24 => "kw_readonly",
            25 => "kw_command_not_found",
            26 => "kw_invalid_option",
            27 => "kw_missing",
            28 => "kw_broken_pipe",
            29 => "kw_killed",
            30 => "kw_timeout",
            31 => "kw_timed_out",
            32 => "single_quote_count",
            33 => "double_quote_count",
            34 => "single_quote_mismatch",
            35 => "double_quote_mismatch",
            36 => "bracket_count",
            37 => "bracket_mismatch",
            38 => "has_line_number",
            39 => "has_column",
            40 => "has_near",
            41 => "has_expected",
            42 => "cmd_length",
            43 => "cmd_has_pipe",
            44 => "cmd_has_output_redirect",
            45 => "cmd_has_input_redirect",
            46 => "cmd_has_stderr_redirect",
            47 => "cmd_has_sudo",
            48 => "cmd_is_compound",
            49 => "cmd_has_variables",
            50 => "shell_bash",
            51 => "shell_sh",
            52 => "shell_zsh",
            53 => "shell_dash",
            54 => "shell_ksh",
            55 => "shell_fish",
            56 => "kw_cannot",
            57 => "kw_failed",
            58 => "kw_error",
            59 => "kw_warning",
            60 => "kw_fatal",
            61 => "kw_abort",
            62 => "kw_segmentation",
            63 => "kw_core_dump",
            _ => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_size() {
        let features = ErrorFeatures::extract(1, "test error", None);
        assert_eq!(features.features.len(), ErrorFeatures::SIZE);
    }

    #[test]
    fn test_exit_code_127_features() {
        let features = ErrorFeatures::extract(127, "bash: foobar: command not found", None);
        assert!((features.features[4] - 1.0).abs() < f32::EPSILON); // exit_code_is_127
        assert!((features.features[25] - 1.0).abs() < f32::EPSILON); // kw_command_not_found
    }

    #[test]
    fn test_exit_code_126_features() {
        let features = ErrorFeatures::extract(126, "bash: ./script.sh: Permission denied", None);
        assert!((features.features[3] - 1.0).abs() < f32::EPSILON); // exit_code_is_126
        assert!((features.features[14] - 1.0).abs() < f32::EPSILON); // kw_permission_denied
    }

    #[test]
    fn test_syntax_error_features() {
        let features =
            ErrorFeatures::extract(1, "bash: syntax error near unexpected token 'done'", None);
        assert!((features.features[18] - 1.0).abs() < f32::EPSILON); // kw_syntax_error
        assert!((features.features[19] - 1.0).abs() < f32::EPSILON); // kw_unexpected
        assert!((features.features[40] - 1.0).abs() < f32::EPSILON); // has_near
    }

    #[test]
    fn test_quote_mismatch_detection() {
        let features = ErrorFeatures::extract(1, "unexpected EOF looking for matching '\"'", None);
        assert!((features.features[35] - 1.0).abs() < f32::EPSILON); // double_quote_mismatch (odd count)
    }

    #[test]
    fn test_command_features() {
        let features =
            ErrorFeatures::extract(1, "error", Some("cat file.txt | grep 'test' > output.txt"));
        assert!(features.features[43] > 0.0); // cmd_has_pipe
        assert!(features.features[44] > 0.0); // cmd_has_output_redirect
    }

    #[test]
    fn test_signal_features() {
        let features = ErrorFeatures::extract(141, "", None); // SIGPIPE
        assert!((features.features[8] - 1.0).abs() < f32::EPSILON); // signal_sigpipe
    }

    #[test]
    fn test_shell_detection() {
        let features_bash = ErrorFeatures::extract(1, "bash: error", None);
        assert!((features_bash.features[50] - 1.0).abs() < f32::EPSILON); // shell_bash

        let features_zsh = ErrorFeatures::extract(1, "zsh: error", None);
        assert!((features_zsh.features[52] - 1.0).abs() < f32::EPSILON); // shell_zsh
    }

    #[test]
    fn test_file_not_found_features() {
        let features =
            ErrorFeatures::extract(1, "cat: /nonexistent: No such file or directory", None);
        assert!((features.features[13] - 1.0).abs() < f32::EPSILON); // kw_no_such_file
    }

    #[test]
    fn test_unbound_variable_features() {
        let features = ErrorFeatures::extract(1, "bash: VAR: unbound variable", None);
        assert!((features.features[22] - 1.0).abs() < f32::EPSILON); // kw_unbound_variable
    }

    #[test]
    fn test_feature_names_coverage() {
        for i in 0..ErrorFeatures::SIZE {
            let name = ErrorFeatures::feature_name(i);
            assert_ne!(name, "unknown", "Feature {i} has no name");
        }
    }

    #[test]
    fn test_normalization_bounds() {
        // Very long error message
        let long_stderr = "x".repeat(10000);
        let features = ErrorFeatures::extract(255, &long_stderr, Some(&"x".repeat(1000)));

        for (i, &val) in features.features.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&val),
                "Feature {i} ({}) out of bounds: {val}",
                ErrorFeatures::feature_name(i)
            );
        }
    }
}
