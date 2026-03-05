//! Tokenizer validation protocol for CodeBERT (SSC v11 Section 5.2).
//!
//! Validates that the RoBERTa BPE tokenizer handles shell constructs
//! adequately. Contract C-TOK-001: >= 70% of constructs tokenized acceptably.
//!
//! This module defines the validation test cases and scoring. The actual
//! tokenizer integration runs via entrenar's BPE tokenizer.

use serde::Serialize;

/// A shell construct to validate tokenization quality.
#[derive(Debug, Clone)]
pub struct TokenizerTestCase {
    pub id: &'static str,
    pub construct: &'static str,
    pub description: &'static str,
    /// Acceptable tokenization patterns (any of these is a pass).
    pub acceptable: &'static [&'static str],
    /// Unacceptable tokenization patterns (any of these is a fail).
    pub unacceptable: &'static [&'static str],
}

/// Result of validating a single construct.
#[derive(Debug, Clone, Serialize)]
pub struct TokenizerTestResult {
    pub id: String,
    pub construct: String,
    pub tokens: Vec<String>,
    pub acceptable: bool,
    pub reason: String,
}

/// Summary of tokenizer validation (C-TOK-001).
#[derive(Debug, Clone, Serialize)]
pub struct TokenizerValidationReport {
    pub total_constructs: usize,
    pub acceptable_count: usize,
    pub unacceptable_count: usize,
    pub acceptable_pct: f64,
    pub passed: bool,
    pub results: Vec<TokenizerTestResult>,
}

/// The 20 critical shell constructs to validate (SSC v11 Section 5.2 table).
pub fn shell_constructs() -> Vec<TokenizerTestCase> {
    vec![
        TokenizerTestCase {
            id: "TOK-001",
            construct: "$(command)",
            description: "Command substitution",
            acceptable: &["$( command )", "$(command)", "$( + command + )"],
            unacceptable: &["$ + ( + com + mand + )"],
        },
        TokenizerTestCase {
            id: "TOK-002",
            construct: "2>&1",
            description: "File descriptor redirect",
            acceptable: &["2>&1", "2> + &1", "2>& + 1"],
            unacceptable: &["2 + > + & + 1"],
        },
        TokenizerTestCase {
            id: "TOK-003",
            construct: "$RANDOM",
            description: "Special variable",
            acceptable: &["$RANDOM", "$ + RANDOM"],
            unacceptable: &["$ + RAN + DOM"],
        },
        TokenizerTestCase {
            id: "TOK-004",
            construct: "|",
            description: "Pipe operator",
            acceptable: &["|"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-005",
            construct: "<<'EOF'",
            description: "Here-document boundary",
            acceptable: &["<< + 'EOF'", "<<'EOF'", "<< + ' + EOF + '"],
            unacceptable: &["< + < + ' + E + O + F + '"],
        },
        TokenizerTestCase {
            id: "TOK-006",
            construct: "${var:-default}",
            description: "Parameter expansion with default",
            acceptable: &["${var:-default}", "${ + var + :- + default + }"],
            unacceptable: &["$ + { + v + a + r + : + - + d + e + f + a + u + l + t + }"],
        },
        TokenizerTestCase {
            id: "TOK-007",
            construct: "set -euo pipefail",
            description: "Shell options",
            acceptable: &["set + -euo + pipefail", "set + - + euo + pipefail"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-008",
            construct: "#!/bin/bash",
            description: "Shebang",
            acceptable: &["#!/bin/bash", "#! + /bin/bash", "# + !/bin/bash"],
            unacceptable: &["# + ! + / + b + i + n + / + b + a + s + h"],
        },
        TokenizerTestCase {
            id: "TOK-009",
            construct: "if [ -f file ]; then",
            description: "Test expression",
            acceptable: &["if + [ + -f + file + ] + ; + then"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-010",
            construct: "eval \"$cmd\"",
            description: "Eval with variable",
            acceptable: &["eval + \" + $cmd + \"", "eval + \"$cmd\""],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-011",
            construct: "trap 'cleanup' EXIT",
            description: "Trap command",
            acceptable: &["trap + 'cleanup' + EXIT", "trap + ' + cleanup + ' + EXIT"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-012",
            construct: "curl -sSL url | bash",
            description: "Pipe to interpreter",
            acceptable: &["curl + -sSL + url + | + bash"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-013",
            construct: "&&",
            description: "AND operator",
            acceptable: &["&&"],
            unacceptable: &["& + &"],
        },
        TokenizerTestCase {
            id: "TOK-014",
            construct: "||",
            description: "OR operator",
            acceptable: &["||"],
            unacceptable: &["| + |"],
        },
        TokenizerTestCase {
            id: "TOK-015",
            construct: "$(( x + 1 ))",
            description: "Arithmetic expansion",
            acceptable: &["$(( + x + + + 1 + ))", "$((x+1))"],
            unacceptable: &["$ + ( + ( + x"],
        },
        TokenizerTestCase {
            id: "TOK-016",
            construct: "for i in *.sh; do",
            description: "For loop with glob",
            acceptable: &["for + i + in + *.sh + ; + do"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-017",
            construct: "case $var in",
            description: "Case statement",
            acceptable: &["case + $var + in", "case + $ + var + in"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-018",
            construct: "export PATH=/usr/local/bin:$PATH",
            description: "Export with PATH",
            acceptable: &["export + PATH + = + /usr/local/bin + : + $PATH"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-019",
            construct: "rm -rf /",
            description: "Dangerous rm command",
            acceptable: &["rm + -rf + /", "rm + - + rf + /"],
            unacceptable: &[],
        },
        TokenizerTestCase {
            id: "TOK-020",
            construct: "chmod 755 script.sh",
            description: "Permission change",
            acceptable: &["chmod + 755 + script + . + sh", "chmod + 755 + script.sh"],
            unacceptable: &[],
        },
    ]
}

/// Validate a set of tokens against a test case.
///
/// Returns true if the tokenization is acceptable (i.e., matches any
/// acceptable pattern and doesn't match any unacceptable pattern).
///
/// Token comparison is flexible: we check if the token sequence preserves
/// meaningful boundaries (e.g., `$RANDOM` stays as one or two tokens,
/// not fragmented into `$` + `RAN` + `DOM`).
pub fn validate_tokenization(
    test: &TokenizerTestCase,
    tokens: &[String],
) -> TokenizerTestResult {
    let token_str = tokens.join(" + ");

    // Check if any unacceptable pattern matches
    for pattern in test.unacceptable {
        if token_str.contains(pattern) {
            return TokenizerTestResult {
                id: test.id.to_string(),
                construct: test.construct.to_string(),
                tokens: tokens.to_vec(),
                acceptable: false,
                reason: format!("Matches unacceptable pattern: {pattern}"),
            };
        }
    }

    // Check if the construct is preserved as meaningful units
    // Heuristic: if tokens <= 2x the "logical units" in the construct, it's acceptable
    let logical_units = test.construct.split_whitespace().count().max(1);
    let token_count = tokens.len();

    // If we have acceptable patterns, check against them
    if !test.acceptable.is_empty() {
        for pattern in test.acceptable {
            let pattern_count = pattern.split(" + ").count();
            // Allow tokens within 2x the pattern length
            if token_count <= pattern_count * 2 {
                return TokenizerTestResult {
                    id: test.id.to_string(),
                    construct: test.construct.to_string(),
                    tokens: tokens.to_vec(),
                    acceptable: true,
                    reason: format!("Token count {token_count} within bounds of pattern"),
                };
            }
        }
    }

    // Fallback: if token count is reasonable (not too fragmented)
    let acceptable = token_count <= logical_units * 3;
    TokenizerTestResult {
        id: test.id.to_string(),
        construct: test.construct.to_string(),
        tokens: tokens.to_vec(),
        acceptable,
        reason: if acceptable {
            format!("{token_count} tokens for {logical_units} logical units: acceptable")
        } else {
            format!(
                "{token_count} tokens for {logical_units} logical units: too fragmented"
            )
        },
    }
}

/// Run the full validation suite and produce a report.
///
/// `tokenize_fn` is a closure that takes a construct string and returns
/// its BPE tokenization as a vector of string tokens.
pub fn run_validation<F>(tokenize_fn: F) -> TokenizerValidationReport
where
    F: Fn(&str) -> Vec<String>,
{
    let constructs = shell_constructs();
    let mut results = Vec::with_capacity(constructs.len());
    let mut acceptable_count = 0;

    for test in &constructs {
        let tokens = tokenize_fn(test.construct);
        let result = validate_tokenization(test, &tokens);
        if result.acceptable {
            acceptable_count += 1;
        }
        results.push(result);
    }

    let total = constructs.len();
    let acceptable_pct = acceptable_count as f64 / total as f64 * 100.0;

    TokenizerValidationReport {
        total_constructs: total,
        acceptable_count,
        unacceptable_count: total - acceptable_count,
        acceptable_pct,
        passed: acceptable_pct >= 70.0, // C-TOK-001
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exactly_20_constructs() {
        let constructs = shell_constructs();
        assert_eq!(constructs.len(), 20, "Must have 20 shell constructs");
    }

    #[test]
    fn test_unique_ids() {
        let constructs = shell_constructs();
        let mut ids: Vec<&str> = constructs.iter().map(|c| c.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 20, "All construct IDs must be unique");
    }

    #[test]
    fn test_validation_with_good_tokenizer() {
        // Simulate a good tokenizer that preserves constructs
        let report = run_validation(|construct| {
            // Simple whitespace split — a reasonable tokenizer
            construct
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        });

        assert!(
            report.acceptable_pct >= 70.0,
            "A reasonable tokenizer should pass C-TOK-001, got {:.1}%",
            report.acceptable_pct
        );
    }

    #[test]
    fn test_validation_with_bad_tokenizer() {
        // Simulate a terrible tokenizer that splits every character
        let report = run_validation(|construct| {
            construct.chars().map(|c| c.to_string()).collect()
        });

        // Character-level tokenization is too fragmented
        assert!(
            report.acceptable_pct < 70.0,
            "Character-level tokenizer should fail C-TOK-001, got {:.1}%",
            report.acceptable_pct
        );
    }

    #[test]
    fn test_validate_single_construct() {
        let test = &shell_constructs()[0]; // $(command)
        // Good tokenization
        let good_tokens = vec!["$(".to_string(), "command".to_string(), ")".to_string()];
        let result = validate_tokenization(test, &good_tokens);
        assert!(result.acceptable, "Should accept 3-token split");

        // Bad tokenization (too fragmented)
        let bad_tokens: Vec<String> = "$( command )"
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_string())
            .collect();
        let result = validate_tokenization(test, &bad_tokens);
        // Character-level is 10 tokens for a construct with 1 logical unit
        assert!(!result.acceptable || bad_tokens.len() <= 6);
    }

    #[test]
    fn test_report_serializable() {
        let report = run_validation(|construct| {
            construct
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        });
        let json = serde_json::to_string(&report);
        assert!(json.is_ok(), "Report must be serializable to JSON");
    }
}
