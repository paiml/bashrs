//! Output formatters for lint results

use crate::linter::{LintResult, Severity};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Output format for lint results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable format (default)
    Human,
    /// JSON format
    Json,
    /// SARIF format (Static Analysis Results Interchange Format)
    Sarif,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "sarif" => Ok(OutputFormat::Sarif),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Write lint results in the specified format
pub fn write_results<W: Write>(
    writer: &mut W,
    result: &LintResult,
    format: OutputFormat,
    file_path: &str,
) -> std::io::Result<()> {
    match format {
        OutputFormat::Human => write_human(writer, result, file_path),
        OutputFormat::Json => write_json(writer, result, file_path),
        OutputFormat::Sarif => write_sarif(writer, result, file_path),
    }
}

/// Human-readable output format with ANSI colors
fn write_human<W: Write>(
    writer: &mut W,
    result: &LintResult,
    file_path: &str,
) -> std::io::Result<()> {
    use crate::cli::color::*;

    if result.diagnostics.is_empty() {
        writeln!(
            writer,
            "{GREEN}✓ No issues found in {CYAN}{file_path}{RESET}"
        )?;
        return Ok(());
    }

    writeln!(writer, "Issues found in {CYAN}{file_path}{RESET}:\n")?;

    for diag in &result.diagnostics {
        let (icon, color) = match diag.severity {
            Severity::Error => ("✗", BRIGHT_RED),
            Severity::Warning => ("⚠", YELLOW),
            Severity::Risk => ("◆", BRIGHT_YELLOW),
            Severity::Perf => ("⚡", CYAN),
            Severity::Info => ("ℹ", DIM),
            Severity::Note => ("→", DIM),
        };

        writeln!(
            writer,
            "{color}{icon}{RESET} {DIM}{}{RESET} {color}{}{RESET}",
            diag.span, diag
        )?;

        if let Some(ref fix) = diag.fix {
            writeln!(writer, "  {GREEN}Fix:{RESET} {}", fix.replacement)?;
        }
        writeln!(writer)?;
    }

    // Summary
    let errors = result.count_by_severity(Severity::Error);
    let warnings = result.count_by_severity(Severity::Warning);
    let infos = result.count_by_severity(Severity::Info);

    let err_color = if errors > 0 { BRIGHT_RED } else { GREEN };
    let warn_color = if warnings > 0 { YELLOW } else { GREEN };

    writeln!(
        writer,
        "Summary: {err_color}{errors} error(s){RESET}, {warn_color}{warnings} warning(s){RESET}, {DIM}{infos} info(s){RESET}",
    )?;

    Ok(())
}

/// JSON output format
#[derive(Serialize, Deserialize)]
struct JsonOutput {
    file: String,
    diagnostics: Vec<JsonDiagnostic>,
    summary: JsonSummary,
}

#[derive(Serialize, Deserialize)]
struct JsonDiagnostic {
    code: String,
    severity: String,
    message: String,
    span: JsonSpan,
    fix: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct JsonSpan {
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
}

#[derive(Serialize, Deserialize)]
struct JsonSummary {
    errors: usize,
    warnings: usize,
    infos: usize,
}

fn write_json<W: Write>(
    writer: &mut W,
    result: &LintResult,
    file_path: &str,
) -> std::io::Result<()> {
    let diagnostics = result
        .diagnostics
        .iter()
        .map(|d| JsonDiagnostic {
            code: d.code.clone(),
            severity: d.severity.to_string(),
            message: d.message.clone(),
            span: JsonSpan {
                start_line: d.span.start_line,
                start_col: d.span.start_col,
                end_line: d.span.end_line,
                end_col: d.span.end_col,
            },
            fix: d.fix.as_ref().map(|f| f.replacement.clone()),
        })
        .collect();

    let output = JsonOutput {
        file: file_path.to_string(),
        diagnostics,
        summary: JsonSummary {
            errors: result.count_by_severity(Severity::Error),
            warnings: result.count_by_severity(Severity::Warning),
            infos: result.count_by_severity(Severity::Info),
        },
    };

    let json = serde_json::to_string_pretty(&output).map_err(std::io::Error::other)?;

    writeln!(writer, "{}", json)?;
    Ok(())
}

/// SARIF output format (simplified version)
#[derive(Serialize, Deserialize)]
struct SarifOutput {
    version: String,
    #[serde(rename = "$schema")]
    schema: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize, Deserialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize, Deserialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize, Deserialize)]
struct SarifDriver {
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize, Deserialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize, Deserialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize, Deserialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize, Deserialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: usize,
    #[serde(rename = "endLine")]
    end_line: usize,
    #[serde(rename = "endColumn")]
    end_column: usize,
}

fn write_sarif<W: Write>(
    writer: &mut W,
    result: &LintResult,
    file_path: &str,
) -> std::io::Result<()> {
    let results = result
        .diagnostics
        .iter()
        .map(|d| {
            let level = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Risk => "warning", // Map Risk to warning in SARIF
                Severity::Perf => "note",    // Map Perf to note in SARIF
                Severity::Info | Severity::Note => "note",
            };

            SarifResult {
                rule_id: d.code.clone(),
                level: level.to_string(),
                message: SarifMessage {
                    text: d.message.clone(),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: file_path.to_string(),
                        },
                        region: SarifRegion {
                            start_line: d.span.start_line,
                            start_column: d.span.start_col,
                            end_line: d.span.end_line,
                            end_column: d.span.end_col,
                        },
                    },
                }],
            }
        })
        .collect();

    let output = SarifOutput {
        version: "2.1.0".to_string(),
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "bashrs".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            },
            results,
        }],
    };

    let json = serde_json::to_string_pretty(&output).map_err(std::io::Error::other)?;

    writeln!(writer, "{}", json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Diagnostic, Span};

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(
            "human".parse::<OutputFormat>().unwrap(),
            OutputFormat::Human
        );
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "sarif".parse::<OutputFormat>().unwrap(),
            OutputFormat::Sarif
        );
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_human_output_no_issues() {
        let result = LintResult::new();
        let mut buffer = Vec::new();

        write_human(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("No issues found"));
    }

    #[test]
    fn test_human_output_with_diagnostics() {
        let mut result = LintResult::new();
        let span = Span::new(1, 5, 1, 10);
        result.add(Diagnostic::new(
            "SC2086",
            Severity::Warning,
            "Test warning",
            span,
        ));

        let mut buffer = Vec::new();
        write_human(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("SC2086"));
        assert!(output.contains("Test warning"));
        assert!(output.contains("1 warning"));
    }

    #[test]
    fn test_json_output() {
        let mut result = LintResult::new();
        let span = Span::new(1, 5, 1, 10);
        result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));

        let mut buffer = Vec::new();
        write_json(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("SC2086"));
        assert!(output.contains("\"file\": \"test.sh\""));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["diagnostics"].is_array());
    }

    #[test]
    fn test_sarif_output() {
        let mut result = LintResult::new();
        let span = Span::new(1, 5, 1, 10);
        result.add(Diagnostic::new("SC2086", Severity::Error, "Test", span));

        let mut buffer = Vec::new();
        write_sarif(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("2.1.0"));
        assert!(output.contains("SC2086"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["runs"].is_array());
    }
}
