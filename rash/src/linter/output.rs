//! Output formatters for lint results

use crate::linter::{LintResult, Severity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// SARIF output format
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rules: Vec<SarifRuleDescriptor>,
}

#[derive(Serialize, Deserialize)]
struct SarifRuleDescriptor {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "helpUri")]
    help_uri: String,
}

#[derive(Serialize, Deserialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fixes: Vec<SarifFix>,
    #[serde(
        rename = "partialFingerprints",
        skip_serializing_if = "HashMap::is_empty"
    )]
    partial_fingerprints: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct SarifFix {
    description: SarifMessage,
    #[serde(rename = "artifactChanges")]
    artifact_changes: Vec<SarifArtifactChange>,
}

#[derive(Serialize, Deserialize)]
struct SarifArtifactChange {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    replacements: Vec<SarifReplacement>,
}

#[derive(Serialize, Deserialize)]
struct SarifReplacement {
    #[serde(rename = "deletedRegion")]
    deleted_region: SarifRegion,
    #[serde(rename = "insertedContent")]
    inserted_content: SarifMessage,
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
    use crate::linter::rule_registry;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Collect unique rule IDs referenced in results
    let mut seen_rules = std::collections::HashSet::new();
    for d in &result.diagnostics {
        seen_rules.insert(d.code.as_str());
    }

    // Build driver.rules[] from registry metadata (only rules referenced in results)
    let rules: Vec<SarifRuleDescriptor> = rule_registry::all_rules()
        .into_iter()
        .filter(|r| seen_rules.contains(r.id))
        .map(|r| {
            let category = r.id.trim_end_matches(char::is_numeric);
            SarifRuleDescriptor {
                id: r.id.to_string(),
                name: r.name.to_string(),
                short_description: SarifMessage {
                    text: r.name.to_string(),
                },
                help_uri: format!(
                    "https://github.com/paiml/bashrs/blob/main/docs/rules/{}.md",
                    category.to_lowercase()
                ),
            }
        })
        .collect();

    // Build results with fixes and fingerprints
    let results = result
        .diagnostics
        .iter()
        .map(|d| {
            let level = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Risk => "warning",
                Severity::Perf => "note",
                Severity::Info | Severity::Note => "note",
            };

            // Build fixes from diagnostic autofix data
            let fixes = if let Some(ref fix) = d.fix {
                vec![SarifFix {
                    description: SarifMessage {
                        text: format!("Replace with: {}", fix.replacement),
                    },
                    artifact_changes: vec![SarifArtifactChange {
                        artifact_location: SarifArtifactLocation {
                            uri: file_path.to_string(),
                        },
                        replacements: vec![SarifReplacement {
                            deleted_region: SarifRegion {
                                start_line: d.span.start_line,
                                start_column: d.span.start_col,
                                end_line: d.span.end_line,
                                end_column: d.span.end_col,
                            },
                            inserted_content: SarifMessage {
                                text: fix.replacement.clone(),
                            },
                        }],
                    }],
                }]
            } else {
                Vec::new()
            };

            // Compute partial fingerprint from rule_id + location + message
            let mut fingerprints = HashMap::new();
            let mut hasher = DefaultHasher::new();
            d.code.hash(&mut hasher);
            d.span.start_line.hash(&mut hasher);
            d.message.hash(&mut hasher);
            fingerprints.insert(
                "primaryLocationLineHash".to_string(),
                format!("{:x}", hasher.finish()),
            );

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
                fixes,
                partial_fingerprints: fingerprints,
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
                    rules,
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

    #[test]
    fn test_sarif_output_has_partial_fingerprints() {
        let mut result = LintResult::new();
        let span = Span::new(1, 5, 1, 10);
        result.add(Diagnostic::new(
            "SEC001",
            Severity::Error,
            "Injection risk",
            span,
        ));

        let mut buffer = Vec::new();
        write_sarif(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let first_result = &parsed["runs"][0]["results"][0];
        assert!(
            first_result["partialFingerprints"]["primaryLocationLineHash"].is_string(),
            "Should have partialFingerprints: {output}"
        );
    }

    #[test]
    fn test_sarif_output_has_rule_descriptors() {
        let mut result = LintResult::new();
        let span = Span::new(1, 5, 1, 10);
        result.add(Diagnostic::new(
            "SEC001",
            Severity::Error,
            "Injection risk",
            span,
        ));

        let mut buffer = Vec::new();
        write_sarif(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let rules = &parsed["runs"][0]["tool"]["driver"]["rules"];
        assert!(rules.is_array(), "Should have rules array: {output}");
        let rules_arr = rules.as_array().expect("rules is array");
        assert!(!rules_arr.is_empty(), "Rules should not be empty");
        let first_rule = &rules_arr[0];
        assert_eq!(first_rule["id"], "SEC001");
        assert!(first_rule["shortDescription"]["text"].is_string());
        assert!(first_rule["helpUri"]
            .as_str()
            .expect("helpUri")
            .contains("docs/rules"));
    }

    #[test]
    fn test_sarif_output_has_fixes() {
        let mut result = LintResult::new();
        let span = Span::new(1, 5, 1, 10);
        let mut diag = Diagnostic::new("IDEM001", Severity::Warning, "mkdir without -p", span);
        diag.fix = Some(crate::linter::Fix::new("mkdir -p /tmp/foo"));
        result.add(diag);

        let mut buffer = Vec::new();
        write_sarif(&mut buffer, &result, "test.sh").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let first_result = &parsed["runs"][0]["results"][0];
        let fixes = first_result["fixes"].as_array().expect("fixes is array");
        assert_eq!(fixes.len(), 1);
        assert!(fixes[0]["description"]["text"]
            .as_str()
            .expect("desc")
            .contains("mkdir -p"));
    }
}
