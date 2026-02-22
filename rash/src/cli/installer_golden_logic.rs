//! Pure computation logic for installer golden trace, graph, and lockfile commands.
//!
//! This module extracts side-effect-free helpers from `commands.rs` related to:
//! - Simulated trace event building (shared by capture and compare)
//! - Graph JSON structure building
//! - Lockfile summary formatting
//!
//! Functions here do NOT perform file I/O, print to stdout, or call external
//! processes.

#![allow(dead_code)]

// ============================================================================
// Trace Event Building (shared by capture & compare)
// ============================================================================

/// Description of a simulated trace event produced from an installer step.
///
/// This is a lightweight, testable representation of the events that
/// `SimulatedTraceCollector` would record.  The actual collector creates
/// full `TraceEvent` objects; here we capture only the semantic intent so
/// the logic can be tested without the collector infrastructure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimulatedEventDesc {
    /// Event kind: `"process"` or `"file"`
    pub kind: &'static str,
    /// Operation name (e.g. `"exec"`, `"write"`, `"open"`)
    pub operation: String,
    /// Optional command or path associated with the event
    pub target: Option<String>,
    /// Optional flags (e.g. `"O_WRONLY|O_CREAT"`)
    pub flags: Option<String>,
    /// Step ID that triggered this event
    pub step_id: String,
}

/// Input data for a single installer step, decoupled from `spec::Step`.
///
/// Callers extract the relevant fields from their domain-specific step type
/// and pass this lightweight struct instead.
#[derive(Debug, Clone)]
pub(crate) struct StepInput {
    pub id: String,
    pub name: String,
    pub action: String,
    pub path: Option<String>,
    pub script_interpreter: Option<String>,
}

/// Build simulated trace event descriptions from a slice of step inputs.
///
/// This is the pure-computation core shared by both
/// `installer_golden_capture_command` and `installer_golden_compare_command`.
/// Each step produces at least one process/exec event, and depending on
/// the action type, may produce additional file or process events.
pub(crate) fn build_simulated_events(steps: &[StepInput]) -> Vec<SimulatedEventDesc> {
    let mut events = Vec::new();

    for step in steps {
        // Every step gets an initial "exec" process event.
        events.push(SimulatedEventDesc {
            kind: "process",
            operation: "exec".to_string(),
            target: Some(step.name.clone()),
            flags: None,
            step_id: step.id.clone(),
        });

        match step.action.as_str() {
            "file-write" => {
                if let Some(ref path) = step.path {
                    events.push(SimulatedEventDesc {
                        kind: "file",
                        operation: "write".to_string(),
                        target: Some(path.clone()),
                        flags: Some("O_WRONLY|O_CREAT".to_string()),
                        step_id: step.id.clone(),
                    });
                }
            }
            "apt-install" => {
                events.push(SimulatedEventDesc {
                    kind: "file",
                    operation: "open".to_string(),
                    target: Some("/var/lib/apt/lists".to_string()),
                    flags: Some("O_RDONLY".to_string()),
                    step_id: step.id.clone(),
                });
            }
            "script" => {
                if let Some(ref interpreter) = step.script_interpreter {
                    events.push(SimulatedEventDesc {
                        kind: "process",
                        operation: "exec".to_string(),
                        target: Some(interpreter.clone()),
                        flags: None,
                        step_id: step.id.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    events
}

// ============================================================================
// Graph JSON Building
// ============================================================================

/// Lightweight representation of a graph node for JSON serialization.
#[derive(Debug, Clone)]
pub(crate) struct GraphNodeInput {
    pub id: String,
    pub name: String,
    pub estimated_duration_secs: f64,
    pub capabilities: Vec<String>,
    pub exclusive_resource: Option<String>,
}

/// Lightweight representation of an execution wave for JSON serialization.
#[derive(Debug, Clone)]
pub(crate) struct WaveInput {
    pub wave_number: usize,
    pub step_ids: Vec<String>,
    pub is_sequential: bool,
    pub sequential_reason: Option<String>,
    pub estimated_duration_secs: f64,
}

/// Lightweight representation of an execution plan for JSON serialization.
#[derive(Debug, Clone)]
pub(crate) struct PlanInput {
    pub waves: Vec<WaveInput>,
    pub total_duration_parallel_secs: f64,
    pub total_duration_sequential_secs: f64,
    pub speedup_percent: f64,
}

/// Build the JSON value for the graph command output.
///
/// This is the pure computation extracted from the `InstallerGraphFormat::Json`
/// branch of `installer_graph_command`.  It constructs a `serde_json::Value`
/// without any I/O.
pub(crate) fn build_graph_json(
    nodes: &[GraphNodeInput],
    plan: &PlanInput,
) -> serde_json::Value {
    serde_json::json!({
        "nodes": nodes.iter().map(|n| {
            serde_json::json!({
                "id": n.id,
                "name": n.name,
                "estimated_duration_secs": n.estimated_duration_secs,
                "capabilities": n.capabilities,
                "exclusive_resource": n.exclusive_resource,
            })
        }).collect::<Vec<_>>(),
        "execution_plan": {
            "waves": plan.waves.iter().map(|w| {
                serde_json::json!({
                    "wave_number": w.wave_number,
                    "step_ids": w.step_ids,
                    "is_sequential": w.is_sequential,
                    "sequential_reason": w.sequential_reason,
                    "estimated_duration_secs": w.estimated_duration_secs,
                })
            }).collect::<Vec<_>>(),
            "total_duration_parallel_secs": plan.total_duration_parallel_secs,
            "total_duration_sequential_secs": plan.total_duration_sequential_secs,
            "speedup_percent": plan.speedup_percent,
        }
    })
}

/// Serialize a graph JSON value to a pretty-printed string.
///
/// Returns the formatted JSON or an empty string on serialization failure,
/// mirroring the `unwrap_or_default` behavior in the original command.
pub(crate) fn graph_json_to_string(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

// ============================================================================
// Lockfile Summary Building
// ============================================================================

/// Lockfile summary for display after generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LockfileSummary {
    /// Display lines to print.
    pub lines: Vec<String>,
}

/// Build the summary lines for the case where no artifacts need locking.
///
/// Extracted from the `result.artifacts == 0` branch of `installer_lock_generate`.
pub(crate) fn format_empty_lockfile_summary(
    lockfile_path_display: &str,
    source_date_epoch: u64,
) -> LockfileSummary {
    LockfileSummary {
        lines: vec![
            "\u{2713} No external artifacts to lock".to_string(),
            "  Hermetic mode will use empty lockfile".to_string(),
            format!("  Created: {}", lockfile_path_display),
            format!("  SOURCE_DATE_EPOCH: {}", source_date_epoch),
        ],
    }
}

/// Build the summary lines after generating a lockfile with artifacts.
///
/// Extracted from the non-empty artifacts branch of `installer_lock_generate`.
pub(crate) fn format_lockfile_generated_summary(
    lockfile_path_display: &str,
    lockfile_version: &str,
    content_hash: &str,
    artifact_count: usize,
    source_date_epoch: u64,
    installer_path_display: &str,
) -> LockfileSummary {
    LockfileSummary {
        lines: vec![
            format!("\u{2713} Generated lockfile: {}", lockfile_path_display),
            format!("  Version: {}", lockfile_version),
            format!("  Content hash: {}", content_hash),
            format!("  Artifacts locked: {}", artifact_count),
            format!("  SOURCE_DATE_EPOCH: {}", source_date_epoch),
            String::new(),
            "Note: Run with real artifact URLs to generate proper hashes.".to_string(),
            format!(
                "      Use 'bashrs installer run {} --hermetic' to execute.",
                installer_path_display
            ),
        ],
    }
}

/// Build the pre-generation status line for the lockfile command.
///
/// Returns the "Generating lockfile for N artifacts..." header.
pub(crate) fn format_lockfile_generating_header(artifact_count: usize) -> String {
    format!("Generating lockfile for {} artifacts...", artifact_count)
}

/// Generate placeholder artifact names for lockfile generation.
///
/// The original code creates `artifact-1`, `artifact-2`, etc.
pub(crate) fn generate_placeholder_artifact_names(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("artifact-{}", i + 1))
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn si(id: &str, name: &str, action: &str, path: Option<&str>, interp: Option<&str>) -> StepInput {
        StepInput { id: id.into(), name: name.into(), action: action.into(), path: path.map(Into::into), script_interpreter: interp.map(Into::into) }
    }
    fn empty_plan() -> PlanInput {
        PlanInput { waves: vec![], total_duration_parallel_secs: 0.0, total_duration_sequential_secs: 0.0, speedup_percent: 0.0 }
    }

    // build_simulated_events (001-010)
    #[test]
    fn test_GOLDEN_LOGIC_001_004_simulated_events_basic() {
        // 001: empty → no events
        assert!(build_simulated_events(&[]).is_empty());
        // 002: unknown action → 1 process/exec event with correct fields
        let ev = build_simulated_events(&[si("s1", "Check OS", "verify", None, None)]);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].kind, "process"); assert_eq!(ev[0].operation, "exec");
        assert_eq!(ev[0].target.as_deref(), Some("Check OS")); assert_eq!(ev[0].step_id, "s1");
        // 003: file-write with path → 2 events; order: process then file with correct flags
        let ev = build_simulated_events(&[si("fw1", "Write Config", "file-write", Some("/etc/app.conf"), None)]);
        assert_eq!(ev.len(), 2);
        assert_eq!(ev[0].kind, "process"); assert_eq!(ev[1].kind, "file");
        assert_eq!(ev[1].operation, "write"); assert_eq!(ev[1].target.as_deref(), Some("/etc/app.conf"));
        assert_eq!(ev[1].flags.as_deref(), Some("O_WRONLY|O_CREAT"));
        // 004: file-write without path → 1 event only
        assert_eq!(build_simulated_events(&[si("fw2", "Write", "file-write", None, None)]).len(), 1);
    }

    #[test]
    fn test_GOLDEN_LOGIC_005_010_simulated_events_apt_script_multi() {
        // 005: apt-install → process + file/open on /var/lib/apt/lists O_RDONLY
        let ev = build_simulated_events(&[si("apt1", "Install Curl", "apt-install", None, None)]);
        assert_eq!(ev.len(), 2); assert_eq!(ev[1].kind, "file"); assert_eq!(ev[1].operation, "open");
        assert_eq!(ev[1].target.as_deref(), Some("/var/lib/apt/lists")); assert_eq!(ev[1].flags.as_deref(), Some("O_RDONLY"));
        // 006: script with interpreter → 2 process events; targets = name then interpreter
        let ev = build_simulated_events(&[si("sc1", "Run Setup", "script", None, Some("bash"))]);
        assert_eq!(ev.len(), 2); assert_eq!(ev[0].target.as_deref(), Some("Run Setup")); assert_eq!(ev[1].target.as_deref(), Some("bash"));
        // 007: script without interpreter → 1 event
        assert_eq!(build_simulated_events(&[si("sc2", "Script", "script", None, None)]).len(), 1);
        // 008: mixed steps → correct total (verify:1 + apt:2 + file-write:2 = 5)
        let steps = vec![si("a", "Check", "verify", None, None), si("b", "Apt", "apt-install", None, None), si("c", "Write", "file-write", Some("/tmp/out"), None)];
        assert_eq!(build_simulated_events(&steps).len(), 5);
        // 009: step_id preserved in all events
        let ev = build_simulated_events(&[si("unique-id-42", "N", "apt-install", None, None)]);
        assert!(ev.iter().all(|e| e.step_id == "unique-id-42"));
        // 010: exec always precedes secondary event (file kind comes second)
        let ev = build_simulated_events(&[si("s", "Step", "apt-install", None, None)]);
        assert_eq!(ev[0].kind, "process"); assert_eq!(ev[1].kind, "file");
    }

    // build_graph_json (011-016)
    #[test]
    fn test_GOLDEN_LOGIC_011_016_graph_json() {
        // 011: empty nodes and plan
        let json = build_graph_json(&[], &empty_plan());
        assert!(json["nodes"].as_array().unwrap().is_empty());
        assert!(json["execution_plan"]["waves"].as_array().unwrap().is_empty());
        // 012: single node with exclusive resource and sequential wave
        let nodes = vec![GraphNodeInput { id: "step-1".into(), name: "First".into(), estimated_duration_secs: 5.0, capabilities: vec!["apt".into()], exclusive_resource: Some("apt-lock".into()) }];
        let plan = PlanInput { waves: vec![WaveInput { wave_number: 0, step_ids: vec!["step-1".into()], is_sequential: true, sequential_reason: Some("Exclusive resource: apt-lock".into()), estimated_duration_secs: 5.0 }], total_duration_parallel_secs: 5.0, total_duration_sequential_secs: 5.0, speedup_percent: 0.0 };
        let json = build_graph_json(&nodes, &plan);
        let jn = &json["nodes"][0]; let jw = &json["execution_plan"]["waves"][0];
        assert_eq!(jn["id"], "step-1"); assert_eq!(jn["name"], "First"); assert_eq!(jn["estimated_duration_secs"], 5.0);
        assert_eq!(jn["capabilities"][0], "apt"); assert_eq!(jn["exclusive_resource"], "apt-lock");
        assert_eq!(jw["wave_number"], 0); assert_eq!(jw["is_sequential"], true);
        // 013: multiple nodes; duration and speedup fields preserved
        let nodes2 = vec![
            GraphNodeInput { id: "a".into(), name: "A".into(), estimated_duration_secs: 1.0, capabilities: vec![], exclusive_resource: None },
            GraphNodeInput { id: "b".into(), name: "B".into(), estimated_duration_secs: 2.0, capabilities: vec![], exclusive_resource: None },
        ];
        let plan2 = PlanInput { waves: vec![WaveInput { wave_number: 0, step_ids: vec!["a".into(), "b".into()], is_sequential: false, sequential_reason: None, estimated_duration_secs: 2.0 }], total_duration_parallel_secs: 2.0, total_duration_sequential_secs: 3.0, speedup_percent: 33.3 };
        let json2 = build_graph_json(&nodes2, &plan2);
        assert_eq!(json2["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(json2["execution_plan"]["total_duration_sequential_secs"], 3.0);
        assert_eq!(json2["execution_plan"]["total_duration_parallel_secs"], 2.0);
        // 014: null exclusive_resource → JSON null
        let json3 = build_graph_json(&[GraphNodeInput { id: "x".into(), name: "X".into(), estimated_duration_secs: 0.5, capabilities: vec![], exclusive_resource: None }], &empty_plan());
        assert!(json3["nodes"][0]["exclusive_resource"].is_null());
        // 015: null sequential_reason → JSON null
        let plan3 = PlanInput { waves: vec![WaveInput { wave_number: 0, step_ids: vec!["a".into()], is_sequential: false, sequential_reason: None, estimated_duration_secs: 1.0 }], total_duration_parallel_secs: 1.0, total_duration_sequential_secs: 1.0, speedup_percent: 0.0 };
        assert!(build_graph_json(&[], &plan3)["execution_plan"]["waves"][0]["sequential_reason"].is_null());
        // 016: speedup_percent preserved exactly
        let plan4 = PlanInput { waves: vec![], total_duration_parallel_secs: 5.0, total_duration_sequential_secs: 20.0, speedup_percent: 75.0 };
        assert_eq!(build_graph_json(&[], &plan4)["execution_plan"]["speedup_percent"], 75.0);
    }

    // graph_json_to_string (017-018)
    #[test]
    fn test_GOLDEN_LOGIC_017_018_graph_json_to_string() {
        let s = graph_json_to_string(&serde_json::json!({"key": "value"}));
        assert!(s.contains("key") && s.contains("value"));
        assert!(graph_json_to_string(&serde_json::json!({"a": 1, "b": 2})).contains('\n'));
    }

    // format_empty_lockfile_summary (019-023)
    #[test]
    fn test_GOLDEN_LOGIC_019_023_empty_lockfile_summary() {
        let s = format_empty_lockfile_summary("/my/lock.json", 1234567890);
        assert!(s.lines[0].contains('\u{2713}') && s.lines[0].contains("No external artifacts"));
        assert!(s.lines.iter().any(|l| l.contains("/my/lock.json")));
        assert!(s.lines.iter().any(|l| l.contains("1234567890")));
        assert!(s.lines.iter().any(|l| l.contains("Hermetic")));
        assert_eq!(s.lines.len(), 4);
    }

    // format_lockfile_generated_summary (024-031)
    #[test]
    fn test_GOLDEN_LOGIC_024_031_generated_lockfile_summary() {
        let s = format_lockfile_generated_summary("/lock.json", "2.0", "sha256:deadbeef", 7, 9999999999, "/my-installer");
        assert!(s.lines[0].contains('\u{2713}') && s.lines[0].contains("/lock.json"));
        assert!(s.lines.iter().any(|l| l.contains("2.0")));
        assert!(s.lines.iter().any(|l| l.contains("sha256:deadbeef")));
        assert!(s.lines.iter().any(|l| l.contains("7")));
        assert!(s.lines.iter().any(|l| l.contains("9999999999")));
        assert!(s.lines.iter().any(|l| l.contains("--hermetic")));
        assert!(s.lines.iter().any(|l| l.contains("/my-installer")));
        assert!(s.lines.iter().any(|l| l.contains("Note:")));
        assert_eq!(s.lines.len(), 8);
    }

    // format_lockfile_generating_header (032-033)
    #[test]
    fn test_GOLDEN_LOGIC_032_033_generating_header() {
        let h = format_lockfile_generating_header(5);
        assert!(h.contains("5") && h.contains("Generating lockfile"));
        assert!(format_lockfile_generating_header(0).contains("0 artifacts"));
    }

    // generate_placeholder_artifact_names (034-036)
    #[test]
    fn test_GOLDEN_LOGIC_034_036_placeholder_artifact_names() {
        assert!(generate_placeholder_artifact_names(0).is_empty());
        let names = generate_placeholder_artifact_names(3);
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "artifact-1"); assert_eq!(names[1], "artifact-2"); assert_eq!(names[2], "artifact-3");
    }

    // Cross-cutting (037-040)
    #[test]
    fn test_GOLDEN_LOGIC_037_040_cross_cutting() {
        // 037: deterministic output
        let steps = vec![si("s1", "A", "file-write", Some("/tmp/a"), None), si("s2", "B", "apt-install", None, None)];
        assert_eq!(build_simulated_events(&steps), build_simulated_events(&steps));
        // 038: graph JSON round-trip
        let nodes = vec![GraphNodeInput { id: "n1".into(), name: "Node 1".into(), estimated_duration_secs: 3.5, capabilities: vec!["docker".into()], exclusive_resource: None }];
        let plan = PlanInput { waves: vec![WaveInput { wave_number: 0, step_ids: vec!["n1".into()], is_sequential: false, sequential_reason: None, estimated_duration_secs: 3.5 }], total_duration_parallel_secs: 3.5, total_duration_sequential_secs: 3.5, speedup_percent: 0.0 };
        let parsed: serde_json::Value = serde_json::from_str(&graph_json_to_string(&build_graph_json(&nodes, &plan))).unwrap();
        assert_eq!(parsed["nodes"][0]["id"], "n1"); assert_eq!(parsed["execution_plan"]["waves"][0]["wave_number"], 0);
        // 039: empty vs generated summaries differ in line count
        assert_ne!(format_empty_lockfile_summary("/l", 0).lines.len(), format_lockfile_generated_summary("/l", "1", "h", 1, 0, "/i").lines.len());
        // 040: all event kinds are "process" or "file"
        let steps = vec![si("a", "A", "file-write", Some("/x"), None), si("b", "B", "apt-install", None, None), si("c", "C", "script", None, Some("sh")), si("d", "D", "unknown", None, None)];
        for ev in &build_simulated_events(&steps) {
            assert!(ev.kind == "process" || ev.kind == "file", "unexpected kind: {}", ev.kind);
        }
    }
}
