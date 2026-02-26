use crate::cli::args::AuditOutputFormat;
use crate::models::{Error, Result};
use std::path::Path;

// ============================================================================
// INSTALLER GOLDEN TRACE AND AUDIT COMMANDS
// Extracted from installer_commands.rs for module size compliance
// ============================================================================

pub(crate) fn installer_golden_capture_command(path: &Path, trace_name: &str) -> Result<()> {
    use crate::installer::{
        GoldenTrace, GoldenTraceManager, InstallerSpec, SimulatedTraceCollector, TraceResult,
    };

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    // Parse spec
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {}", e),
        ))
    })?;
    let spec = InstallerSpec::parse(&content)?;

    // Create trace manager
    let trace_dir = path.parent().unwrap_or(path).join(".golden-traces");
    let manager = GoldenTraceManager::new(&trace_dir);

    // Create simulated trace collector
    // In production, this would integrate with renacer for real syscall tracing
    let mut collector = SimulatedTraceCollector::new();

    // Record simulated events for each step
    for step in &spec.step {
        collector.record_process_event(
            "exec",
            Some(&step.name),
            None,
            Some(&step.id),
            TraceResult::Success,
        );

        // Add file events based on step action
        match step.action.as_str() {
            "file-write" => {
                if let Some(ref path) = step.path {
                    collector.record_file_event(
                        "write",
                        path,
                        Some("O_WRONLY|O_CREAT"),
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            "apt-install" => {
                collector.record_file_event(
                    "open",
                    "/var/lib/apt/lists",
                    Some("O_RDONLY"),
                    Some(&step.id),
                    TraceResult::Success,
                );
            }
            "script" => {
                if let Some(ref script) = step.script {
                    collector.record_process_event(
                        "exec",
                        Some(&script.interpreter),
                        None,
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            _ => {}
        }
    }

    // Create golden trace
    let events = collector
        .into_trace(trace_name, &spec.installer.version)
        .events;
    let trace = GoldenTrace {
        name: trace_name.to_string(),
        captured_at: chrono::Utc::now().to_rfc3339(),
        installer_version: spec.installer.version.clone(),
        result_hash: format!("{:016x}", {
            // Simple hash of events for reproducibility check
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            events.len().hash(&mut hasher);
            trace_name.hash(&mut hasher);
            hasher.finish()
        }),
        events,
        steps_executed: spec.step.len(),
        duration_ms: 0,
    };

    // Save trace
    let trace_path = manager.save_trace(&trace)?;

    println!("Golden trace captured successfully:");
    println!("  Name: {}", trace_name);
    println!("  Path: {}", trace_path.display());
    println!("  Events: {}", trace.events.len());
    println!("  Steps: {}", trace.steps_executed);
    println!();
    println!("To compare against this trace later:");
    println!(
        "  bashrs installer golden-compare {} --trace {}",
        path.display(),
        trace_name
    );

    Ok(())
}

pub(crate) fn installer_golden_compare_command(path: &Path, trace_name: &str) -> Result<()> {
    use crate::installer::{
        GoldenTrace, GoldenTraceManager, InstallerSpec, SimulatedTraceCollector, TraceComparison,
        TraceResult,
    };

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    // Parse spec
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {}", e),
        ))
    })?;
    let spec = InstallerSpec::parse(&content)?;

    // Create trace manager
    let trace_dir = path.parent().unwrap_or(path).join(".golden-traces");
    let manager = GoldenTraceManager::new(&trace_dir);

    // Load golden trace
    let golden = manager.load_trace(trace_name)?;

    // Capture current trace (simulated)
    let mut collector = SimulatedTraceCollector::new();
    for step in &spec.step {
        collector.record_process_event(
            "exec",
            Some(&step.name),
            None,
            Some(&step.id),
            TraceResult::Success,
        );

        match step.action.as_str() {
            "file-write" => {
                if let Some(ref path) = step.path {
                    collector.record_file_event(
                        "write",
                        path,
                        Some("O_WRONLY|O_CREAT"),
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            "apt-install" => {
                collector.record_file_event(
                    "open",
                    "/var/lib/apt/lists",
                    Some("O_RDONLY"),
                    Some(&step.id),
                    TraceResult::Success,
                );
            }
            "script" => {
                if let Some(ref script) = step.script {
                    collector.record_process_event(
                        "exec",
                        Some(&script.interpreter),
                        None,
                        Some(&step.id),
                        TraceResult::Success,
                    );
                }
            }
            _ => {}
        }
    }

    let current = GoldenTrace {
        name: format!("{}-current", trace_name),
        captured_at: chrono::Utc::now().to_rfc3339(),
        installer_version: spec.installer.version.clone(),
        events: collector
            .into_trace(trace_name, &spec.installer.version)
            .events,
        result_hash: String::new(),
        steps_executed: spec.step.len(),
        duration_ms: 0,
    };

    // Compare traces
    let comparison = TraceComparison::compare(&golden, &current);

    // Print report
    println!("{}", comparison.to_report());

    if comparison.is_equivalent() {
        println!("Result: PASS - No regression detected");
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "Trace regression detected: {} added, {} removed events",
            comparison.added.len(),
            comparison.removed.len()
        )))
    }
}

pub(crate) fn installer_audit_command(
    path: &Path,
    format: AuditOutputFormat,
    security_only: bool,
    min_severity: Option<&str>,
    ignore: &[String],
) -> Result<()> {
    use crate::installer::{AuditContext, AuditSeverity, InstallerSpec};

    // Find installer.toml
    let installer_toml = if path.is_dir() {
        path.join("installer.toml")
    } else {
        path.to_path_buf()
    };

    if !installer_toml.exists() {
        return Err(Error::Validation(format!(
            "installer.toml not found at {}",
            installer_toml.display()
        )));
    }

    // Parse the TOML
    let content = std::fs::read_to_string(&installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read installer.toml: {e}"),
        ))
    })?;

    let spec = InstallerSpec::parse(&content)?;

    // Set up audit context
    let mut ctx = if security_only {
        AuditContext::security_only()
    } else {
        AuditContext::new()
    };

    // Set minimum severity if specified
    if let Some(sev) = min_severity {
        let severity = match sev.to_lowercase().as_str() {
            "info" => AuditSeverity::Info,
            "suggestion" => AuditSeverity::Suggestion,
            "warning" => AuditSeverity::Warning,
            "error" => AuditSeverity::Error,
            "critical" => AuditSeverity::Critical,
            _ => {
                return Err(Error::Validation(format!(
                    "Invalid severity '{}'. Valid values: info, suggestion, warning, error, critical",
                    sev
                )));
            }
        };
        ctx = ctx.with_min_severity(severity);
    }

    // Issue #110: Add ignored rules
    for rule in ignore {
        ctx = ctx.with_ignored_rule(rule);
    }

    // Run audit
    let report = ctx.audit_parsed_spec(&spec, &installer_toml);

    // Output report
    match format {
        AuditOutputFormat::Human => {
            println!("{}", report.format());
        }
        AuditOutputFormat::Json => {
            println!("{}", report.to_json());
        }
        AuditOutputFormat::Sarif => {
            // SARIF format not yet implemented for installer audit
            println!("{}", report.to_json());
        }
    }

    // Return error if there are errors or critical issues
    if report.has_errors() {
        Err(Error::Validation(format!(
            "Audit found {} error(s). Score: {}/100 (Grade: {})",
            report.findings_by_severity(AuditSeverity::Error).len()
                + report.findings_by_severity(AuditSeverity::Critical).len(),
            report.score(),
            report.grade()
        )))
    } else {
        Ok(())
    }
}
