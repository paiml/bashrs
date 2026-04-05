//! Progress visualization for installers (#107)
//!
//! Provides visual progress bars and status tracking for installer execution.
//! Inspired by trueno-viz, but implemented standalone for minimal dependencies.
//!
//! # Example
//!
//! ```bash
//! bashrs installer run ./my-installer --progress
//! ```

use std::collections::HashMap;
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Step execution state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepState {
    /// Step is waiting to execute
    Pending,

    /// Step is currently executing
    Running {
        /// Progress percentage (0-100)
        progress: u8,
        /// Current operation message
        message: String,
        /// When step started
        started_at: Instant,
    },

    /// Step completed successfully
    Completed {
        /// Time taken to complete
        duration: Duration,
    },

    /// Step failed with error
    Failed {
        /// Error message
        error: String,
        /// Time spent before failure
        duration: Duration,
    },

    /// Step was skipped
    Skipped {
        /// Reason for skipping
        reason: String,
    },
}

impl StepState {
    /// Get status symbol for display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Pending => "⏳",
            Self::Running { .. } => "▶",
            Self::Completed { .. } => "✓",
            Self::Failed { .. } => "✗",
            Self::Skipped { .. } => "⊘",
        }
    }

    /// Get status text
    pub fn text(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Running { .. } => "RUNNING",
            Self::Completed { .. } => "COMPLETE",
            Self::Failed { .. } => "FAILED",
            Self::Skipped { .. } => "SKIPPED",
        }
    }

    /// Check if step is complete (success or failure)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Failed { .. } | Self::Skipped { .. }
        )
    }

    /// Get progress percentage
    pub fn progress(&self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Running { progress, .. } => *progress,
            Self::Completed { .. } => 100,
            Self::Failed { .. } => 0,
            Self::Skipped { .. } => 0,
        }
    }
}

/// Information about a step being tracked
#[derive(Debug, Clone)]
pub struct StepInfo {
    /// Step ID
    pub id: String,
    /// Step name for display
    pub name: String,
    /// Current state
    pub state: StepState,
    /// Step index (1-based)
    pub index: usize,
}

impl StepInfo {
    /// Create new pending step
    pub fn new(id: &str, name: &str, index: usize) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            state: StepState::Pending,
            index,
        }
    }
}

/// Progress display style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressStyle {
    /// Minimal output (single line updates)
    Minimal,
    /// Standard output (one line per step)
    #[default]
    Standard,
    /// Verbose output (full progress bars)
    Verbose,
    /// Quiet (no output)
    Quiet,
}

/// Execution mode indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionMode {
    /// Normal execution
    #[default]
    Normal,
    /// Dry-run (simulation)
    DryRun,
    /// Hermetic (reproducible)
    Hermetic,
    /// Test mode
    Test,
}

impl ExecutionMode {
    /// Get mode label for display
    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::DryRun => "DRY-RUN",
            Self::Hermetic => "HERMETIC",
            Self::Test => "TEST",
        }
    }
}

/// Progress tracker for installer execution
#[derive(Debug)]
pub struct InstallerProgress {
    /// Installer name
    name: String,
    /// Installer version
    version: String,
    /// Steps being tracked
    steps: Vec<StepInfo>,
    /// Step lookup by ID
    step_index: HashMap<String, usize>,
    /// When execution started
    started_at: Instant,
    /// Current checkpoint ID
    checkpoint: Option<String>,
    /// Execution mode
    mode: ExecutionMode,
    /// Artifacts verified count
    artifacts_verified: usize,
    /// Total artifacts
    artifacts_total: usize,
    /// Whether signatures are verified
    signatures_verified: bool,
    /// Whether trace is being recorded
    trace_recording: bool,
    /// Display style
    style: ProgressStyle,
}

impl InstallerProgress {
    /// Create new progress tracker
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            steps: Vec::new(),
            step_index: HashMap::new(),
            started_at: Instant::now(),
            checkpoint: None,
            mode: ExecutionMode::default(),
            artifacts_verified: 0,
            artifacts_total: 0,
            signatures_verified: false,
            trace_recording: false,
            style: ProgressStyle::default(),
        }
    }

    /// Set display style
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Set execution mode
    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set artifact counts
    pub fn with_artifacts(mut self, verified: usize, total: usize) -> Self {
        self.artifacts_verified = verified;
        self.artifacts_total = total;
        self
    }

    /// Set signature verification status
    pub fn with_signatures(mut self, verified: bool) -> Self {
        self.signatures_verified = verified;
        self
    }

    /// Set trace recording status
    pub fn with_trace(mut self, recording: bool) -> Self {
        self.trace_recording = recording;
        self
    }

    /// Add a step to track
    pub fn add_step(&mut self, id: &str, name: &str) {
        let index = self.steps.len() + 1;
        self.step_index.insert(id.to_string(), self.steps.len());
        self.steps.push(StepInfo::new(id, name, index));
    }

    /// Get step by ID
    pub fn get_step(&self, id: &str) -> Option<&StepInfo> {
        self.step_index.get(id).and_then(|&i| self.steps.get(i))
    }

    /// Get mutable step by ID
    fn get_step_mut(&mut self, id: &str) -> Option<&mut StepInfo> {
        if let Some(&i) = self.step_index.get(id) {
            self.steps.get_mut(i)
        } else {
            None
        }
    }

    /// Start a step
    pub fn start_step(&mut self, id: &str, message: &str) {
        if let Some(step) = self.get_step_mut(id) {
            step.state = StepState::Running {
                progress: 0,
                message: message.to_string(),
                started_at: Instant::now(),
            };
        }
    }

    /// Update step progress
    pub fn update_step(&mut self, id: &str, progress: u8, message: &str) {
        if let Some(step) = self.get_step_mut(id) {
            if let StepState::Running { started_at, .. } = &step.state {
                step.state = StepState::Running {
                    progress: progress.min(100),
                    message: message.to_string(),
                    started_at: *started_at,
                };
            }
        }
    }

    /// Complete a step successfully
    pub fn complete_step(&mut self, id: &str) {
        if let Some(step) = self.get_step_mut(id) {
            let duration = if let StepState::Running { started_at, .. } = &step.state {
                started_at.elapsed()
            } else {
                Duration::ZERO
            };
            step.state = StepState::Completed { duration };
            self.checkpoint = Some(id.to_string());
        }
    }

    /// Fail a step
    pub fn fail_step(&mut self, id: &str, error: &str) {
        if let Some(step) = self.get_step_mut(id) {
            let duration = if let StepState::Running { started_at, .. } = &step.state {
                started_at.elapsed()
            } else {
                Duration::ZERO
            };
            step.state = StepState::Failed {
                error: error.to_string(),
                duration,
            };
        }
    }

    /// Skip a step
    pub fn skip_step(&mut self, id: &str, reason: &str) {
        if let Some(step) = self.get_step_mut(id) {
            step.state = StepState::Skipped {
                reason: reason.to_string(),
            };
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get completed step count
    pub fn completed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s.state, StepState::Completed { .. }))
            .count()
    }

    /// Get failed step count
    pub fn failed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s.state, StepState::Failed { .. }))
            .count()
    }

    /// Get skipped step count
    pub fn skipped_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s.state, StepState::Skipped { .. }))
            .count()
    }

    /// Estimate remaining time
    pub fn estimated_remaining(&self) -> Option<Duration> {
        let completed = self.completed_count();
        if completed == 0 {
            return None;
        }

        let remaining = self
            .steps
            .len()
            .saturating_sub(completed + self.skipped_count());
        if remaining == 0 {
            return Some(Duration::ZERO);
        }

        let elapsed = self.elapsed();
        let avg_per_step = elapsed / completed as u32;
        Some(avg_per_step * remaining as u32)
    }

    /// Check if all steps are complete
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| s.state.is_terminal())
    }

    /// Check if any steps failed
    pub fn has_failures(&self) -> bool {
        self.steps
            .iter()
            .any(|s| matches!(s.state, StepState::Failed { .. }))
    }

    /// Get steps iterator
    pub fn steps(&self) -> &[StepInfo] {
        &self.steps
    }

    /// Get total step count
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }
}

/// Progress renderer trait
pub trait ProgressRenderer {
    /// Render header
    fn render_header(&self, progress: &InstallerProgress) -> String;

    /// Render a single step
    fn render_step(&self, step: &StepInfo, total: usize) -> String;

    /// Render footer with summary
    fn render_footer(&self, progress: &InstallerProgress) -> String;

    /// Render full progress display
    fn render(&self, progress: &InstallerProgress) -> String {
        let mut output = self.render_header(progress);
        for step in progress.steps() {
            output.push_str(&self.render_step(step, progress.total_steps()));
        }
        output.push_str(&self.render_footer(progress));
        output
    }
}

/// Terminal text renderer
#[derive(Debug, Default)]
pub struct TerminalRenderer {
    /// Terminal width (0 for auto)
    width: usize,
}

impl TerminalRenderer {
    /// Create with default width
    pub fn new() -> Self {
        Self { width: 80 }
    }

    /// Create with specific width
    pub fn with_width(width: usize) -> Self {
        Self { width }
    }

    /// Format duration for display
    fn format_duration(d: Duration) -> String {
        let secs = d.as_secs();
        if secs >= 60 {
            format!("{}m {:02}s", secs / 60, secs % 60)
        } else if secs > 0 {
            format!("{}.{:02}s", secs, d.subsec_millis() / 10)
        } else {
            format!("{}ms", d.as_millis())
        }
    }

    /// Create progress bar string
    fn progress_bar(&self, progress: u8, width: usize) -> String {
        let filled = (progress as usize * width) / 100;
        let empty = width.saturating_sub(filled);

        let filled_char = '━';
        let partial_char = if progress < 100 && filled < width {
            '╸'
        } else {
            filled_char
        };
        let empty_char = '━';

        if progress >= 100 {
            filled_char.to_string().repeat(width)
        } else if filled > 0 {
            format!(
                "{}{}{}",
                filled_char.to_string().repeat(filled.saturating_sub(1)),
                partial_char,
                empty_char.to_string().repeat(empty)
            )
        } else {
            empty_char.to_string().repeat(width)
        }
    }
}

impl ProgressRenderer for TerminalRenderer {
    fn render_header(&self, progress: &InstallerProgress) -> String {
        let line = "═".repeat(self.width);
        format!("{} v{}\n{}\n\n", progress.name, progress.version, line)
    }

    fn render_step(&self, step: &StepInfo, total: usize) -> String {
        let bar_width = 40;
        let bar = self.progress_bar(step.state.progress(), bar_width);

        let (status_style, detail) = match &step.state {
            StepState::Pending => ("", "Pending".to_string()),
            StepState::Running { message, .. } => ("", message.clone()),
            StepState::Completed { duration } => {
                ("", format!("({})", Self::format_duration(*duration)))
            }
            StepState::Failed { error, .. } => ("", format!("← {}", truncate(error, 30))),
            StepState::Skipped { reason } => ("", format!("({})", truncate(reason, 30))),
        };

        let progress_pct = format!("{:>3}%", step.state.progress());

        format!(
            "  Step {}/{}: {}\n  {} {} {} {} {}{}\n\n",
            step.index,
            total,
            step.name,
            step.state.symbol(),
            truncate(&step.id, 20),
            bar,
            progress_pct,
            status_style,
            detail
        )
    }

    fn render_footer(&self, progress: &InstallerProgress) -> String {
        let line = "─".repeat(self.width);
        let elapsed = Self::format_duration(progress.elapsed());
        let remaining = progress
            .estimated_remaining()
            .map_or_else(|| "calculating...".to_string(), Self::format_duration);

        let checkpoint = progress.checkpoint.as_deref().unwrap_or("none");

        let mut footer = format!("{}\n", line);
        footer.push_str(&format!(
            "  Elapsed: {} │ Remaining: ~{} │ Checkpoint: {} │ Mode: {}\n",
            elapsed,
            remaining,
            truncate(checkpoint, 15),
            progress.mode.label()
        ));

        if progress.artifacts_total > 0 || progress.signatures_verified || progress.trace_recording
        {
            footer.push_str(&format!(
                "  Artifacts: {}/{} verified │ Signatures: {} │ Trace: {}\n",
                progress.artifacts_verified,
                progress.artifacts_total,
                if progress.signatures_verified {
                    "✓"
                } else {
                    "○"
                },
                if progress.trace_recording {
                    "recording"
                } else {
                    "off"
                }
            ));
        }

        let end_line = "═".repeat(self.width);
        footer.push_str(&format!("{}\n", end_line));

        footer
    }
}

/// JSON renderer for programmatic output
#[derive(Debug, Default)]
pub struct JsonRenderer;

impl JsonRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressRenderer for JsonRenderer {
    fn render_header(&self, _progress: &InstallerProgress) -> String {
        String::from("{\n  \"steps\": [\n")
    }

    fn render_step(&self, step: &StepInfo, total: usize) -> String {
        let (status, message) = match &step.state {
            StepState::Pending => ("pending", None),
            StepState::Running { message, .. } => ("running", Some(message.clone())),
            StepState::Completed { .. } => ("completed", None),
            StepState::Failed { error, .. } => ("failed", Some(error.clone())),
            StepState::Skipped { reason } => ("skipped", Some(reason.clone())),
        };

        let duration_ms = match &step.state {
            StepState::Completed { duration } | StepState::Failed { duration, .. } => {
                Some(duration.as_millis())
            }
            StepState::Running { started_at, .. } => Some(started_at.elapsed().as_millis()),
            _ => None,
        };

        let comma = if step.index < total { "," } else { "" };

        let mut json = format!(
            "    {{\n      \"id\": \"{}\",\n      \"name\": \"{}\",\n      \"status\": \"{}\",\n      \"progress\": {}",
            escape_json(&step.id),
            escape_json(&step.name),
            status,
            step.state.progress()
        );

        if let Some(d) = duration_ms {
            json.push_str(&format!(",\n      \"duration_ms\": {}", d));
        }

        if let Some(msg) = message {
            json.push_str(&format!(",\n      \"message\": \"{}\"", escape_json(&msg)));
        }

        json.push_str(&format!("\n    }}{}\n", comma));
        json
    }

    fn render_footer(&self, progress: &InstallerProgress) -> String {
        format!(
            "  ],\n  \"summary\": {{\n    \"total\": {},\n    \"completed\": {},\n    \"failed\": {},\n    \"skipped\": {},\n    \"elapsed_ms\": {}\n  }}\n}}\n",
            progress.total_steps(),
            progress.completed_count(),
            progress.failed_count(),
            progress.skipped_count(),
            progress.elapsed().as_millis()
        )
    }
}

/// Live progress display (for terminal)
pub struct LiveProgress {
    progress: InstallerProgress,
    renderer: TerminalRenderer,
    last_render: Instant,
    min_refresh: Duration,
}

impl LiveProgress {
    /// Create new live progress display
    pub fn new(progress: InstallerProgress) -> Self {
        Self {
            progress,
            renderer: TerminalRenderer::new(),
            last_render: Instant::now(),
            min_refresh: Duration::from_millis(100),
        }
    }

    /// Get mutable reference to progress
    pub fn progress_mut(&mut self) -> &mut InstallerProgress {
        &mut self.progress
    }

    /// Refresh display if enough time has passed
    pub fn refresh(&mut self) {
        if self.last_render.elapsed() >= self.min_refresh {
            self.force_refresh();
        }
    }

    /// Force refresh display
    pub fn force_refresh(&mut self) {
        let output = self.renderer.render(&self.progress);
        // Clear and redraw
        print!("\x1B[2J\x1B[H"); // Clear screen, move to top
        print!("{}", output);
        let _ = io::stdout().flush();
        self.last_render = Instant::now();
    }

    /// Finish display (don't clear)
    pub fn finish(self) {
        let output = self.renderer.render(&self.progress);
        println!("{}", output);
    }
}

/// Generate summary of completed installation
pub fn generate_summary(progress: &InstallerProgress) -> InstallationSummary {
    let mut step_results = Vec::new();

    for step in progress.steps() {
        let (status, duration, message) = match &step.state {
            StepState::Completed { duration } => ("passed", Some(*duration), None),
            StepState::Failed { error, duration } => {
                ("failed", Some(*duration), Some(error.clone()))
            }
            StepState::Skipped { reason } => ("skipped", None, Some(reason.clone())),
            _ => ("pending", None, None),
        };

        step_results.push(StepResult {
            id: step.id.clone(),
            name: step.name.clone(),
            status: status.to_string(),
            duration,
            message,
        });
    }

    InstallationSummary {
        name: progress.name.clone(),
        version: progress.version.clone(),
        total_steps: progress.total_steps(),
        completed: progress.completed_count(),
        failed: progress.failed_count(),
        skipped: progress.skipped_count(),
        total_duration: progress.elapsed(),
        success: !progress.has_failures(),
        step_results,
    }
}

/// Summary of installation run
#[derive(Debug, Clone)]
pub struct InstallationSummary {
    /// Installer name
    pub name: String,
    /// Installer version
    pub version: String,
    /// Total steps
    pub total_steps: usize,
    /// Completed steps
    pub completed: usize,
    /// Failed steps
    pub failed: usize,
    /// Skipped steps
    pub skipped: usize,
    /// Total duration
    pub total_duration: Duration,
    /// Overall success
    pub success: bool,
    /// Individual step results
    pub step_results: Vec<StepResult>,
}

impl InstallationSummary {
    /// Format as text
    pub fn format(&self) -> String {
        let status = if self.success {
            "✓ SUCCESS"
        } else {
            "✗ FAILED"
        };
        let duration = if self.total_duration.as_secs() >= 60 {
            format!(
                "{}m {:02}s",
                self.total_duration.as_secs() / 60,
                self.total_duration.as_secs() % 60
            )
        } else {
            format!("{:.2}s", self.total_duration.as_secs_f64())
        };

        let mut output = format!("{} v{} - {}\n", self.name, self.version, status);
        output.push_str(&format!(
            "Steps: {}/{} completed, {} failed, {} skipped\n",
            self.completed, self.total_steps, self.failed, self.skipped
        ));
        output.push_str(&format!("Duration: {}\n", duration));

        if self.failed > 0 {
            output.push_str("\nFailed steps:\n");
            for step in &self.step_results {
                if step.status == "failed" {
                    let msg = step.message.as_deref().unwrap_or("unknown error");
                    output.push_str(&format!("  ✗ {}: {}\n", step.id, msg));
                }
            }
        }

        output
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"name\": \"{}\",\n", escape_json(&self.name)));
        json.push_str(&format!(
            "  \"version\": \"{}\",\n",
            escape_json(&self.version)
        ));
        json.push_str(&format!("  \"success\": {},\n", self.success));
        json.push_str(&format!("  \"total_steps\": {},\n", self.total_steps));
        json.push_str(&format!("  \"completed\": {},\n", self.completed));
        json.push_str(&format!("  \"failed\": {},\n", self.failed));
        json.push_str(&format!("  \"skipped\": {},\n", self.skipped));
        json.push_str(&format!(
            "  \"duration_ms\": {},\n",
            self.total_duration.as_millis()
        ));
        json.push_str("  \"steps\": [\n");

        for (i, step) in self.step_results.iter().enumerate() {
            let comma = if i < self.step_results.len() - 1 {
                ","
            } else {
                ""
            };
            json.push_str(&format!(
                "    {{\"id\": \"{}\", \"status\": \"{}\"}}{}",
                escape_json(&step.id),
                step.status,
                comma
            ));
            json.push('\n');
        }

        json.push_str("  ]\n");
        json.push_str("}\n");
        json
    }
}

/// Result of a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Status (passed, failed, skipped)
    pub status: String,
    /// Duration if completed
    pub duration: Option<Duration>,
    /// Error or skip message
    pub message: Option<String>,
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Escape string for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
#[path = "progress_tests_extracted.rs"]
mod tests_extracted;
