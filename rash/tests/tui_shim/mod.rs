//! Minimal in-repo replacement for the `jugar-probar` TUI/GUI coverage helpers.
//!
//! The tests used jugar-probar only for coverage counters, frame text
//! assertions and snapshot hashes. jugar-probar pulled `chrono` into
//! Cargo.lock (RUSTSEC-2026-0543), so the used subset lives here, on std only.
#![allow(dead_code, unreachable_pub)]

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

/// Declared-vs-visited coverage of clickable elements and screens.
pub struct GuiCoverage {
    buttons: BTreeSet<String>,
    screens: BTreeSet<String>,
    clicked: BTreeSet<String>,
    visited: BTreeSet<String>,
}

pub struct CoverageReport {
    pub total_elements: usize,
    pub covered_elements: usize,
    pub total_states: usize,
    pub covered_states: usize,
    pub element_coverage: f64,
    pub state_coverage: f64,
    pub overall_coverage: f64,
    pub is_complete: bool,
}

fn count(hit: &BTreeSet<String>, declared: &BTreeSet<String>) -> usize {
    declared.iter().filter(|d| hit.contains(*d)).count()
}

fn ratio(hit: &BTreeSet<String>, declared: &BTreeSet<String>) -> f64 {
    if declared.is_empty() {
        return 1.0;
    }
    declared.iter().filter(|d| hit.contains(*d)).count() as f64 / declared.len() as f64
}

impl GuiCoverage {
    pub fn new(buttons: &[&str], screens: &[&str]) -> Self {
        let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
        Self {
            buttons: set(buttons),
            screens: set(screens),
            clicked: BTreeSet::new(),
            visited: BTreeSet::new(),
        }
    }
    pub fn click(&mut self, id: &str) {
        self.clicked.insert(id.to_string());
    }
    pub fn visit(&mut self, screen: &str) {
        self.visited.insert(screen.to_string());
    }
    pub fn generate_report(&self) -> CoverageReport {
        let element_coverage = ratio(&self.clicked, &self.buttons);
        let state_coverage = ratio(&self.visited, &self.screens);
        let overall_coverage = (element_coverage + state_coverage) / 2.0;
        CoverageReport {
            total_elements: self.buttons.len(),
            covered_elements: count(&self.clicked, &self.buttons),
            total_states: self.screens.len(),
            covered_states: count(&self.visited, &self.screens),
            element_coverage,
            state_coverage,
            overall_coverage,
            is_complete: element_coverage >= 1.0 && state_coverage >= 1.0,
        }
    }
    /// Overall coverage in percent.
    pub fn percent(&self) -> f64 {
        self.generate_report().overall_coverage * 100.0
    }
    pub fn meets(&self, threshold: f64) -> bool {
        self.percent() >= threshold
    }
}

#[macro_export]
macro_rules! gui_coverage {
    (buttons: [$($b:expr),* $(,)?], screens: [$($s:expr),* $(,)?]) => {
        $crate::tui_shim::GuiCoverage::new(&[$($b),*], &[$($s),*])
    };
}

pub struct TuiFrame {
    lines: Vec<String>,
}

impl TuiFrame {
    pub fn from_lines<S: AsRef<str>>(lines: &[S]) -> Self {
        Self {
            lines: lines.iter().map(|l| l.as_ref().to_string()).collect(),
        }
    }
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
    pub fn as_text(&self) -> String {
        self.lines.join("\n")
    }
    pub fn contains(&self, needle: &str) -> bool {
        self.as_text().contains(needle)
    }
}

pub struct FrameExpect<'a>(&'a TuiFrame);

pub fn expect_frame(frame: &TuiFrame) -> FrameExpect<'_> {
    FrameExpect(frame)
}

impl FrameExpect<'_> {
    pub fn expect_contain_text(&self, text: &str) -> Result<(), String> {
        if self.0.contains(text) {
            Ok(())
        } else {
            Err(format!("frame does not contain {text:?}"))
        }
    }
}

pub struct TuiSnapshot {
    pub name: String,
    pub hash: String,
}

impl TuiSnapshot {
    pub fn from_frame(name: &str, frame: &TuiFrame) -> Self {
        let mut h = DefaultHasher::new();
        frame.as_text().hash(&mut h);
        Self {
            name: name.to_string(),
            hash: format!("{:016x}", h.finish()),
        }
    }
}

/// Clickable-element / screen coverage tracker (jugar-probar `ux_coverage` subset).
#[derive(Default)]
pub struct UxCoverageBuilder {
    clickable: BTreeSet<String>,
    screens: BTreeSet<String>,
}

pub struct UxCoverageTracker {
    inner: GuiCoverage,
}

impl UxCoverageBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clickable(mut self, group: &str, id: &str) -> Self {
        self.clickable.insert(format!("{group}:{id}"));
        self
    }
    pub fn screen(mut self, name: &str) -> Self {
        self.screens.insert(name.to_string());
        self
    }
    pub fn build(self) -> UxCoverageTracker {
        UxCoverageTracker {
            inner: GuiCoverage {
                buttons: self.clickable,
                screens: self.screens,
                clicked: BTreeSet::new(),
                visited: BTreeSet::new(),
            },
        }
    }
}

impl UxCoverageTracker {
    pub fn record_click(&mut self, group: &str, id: &str) {
        self.inner.click(&format!("{group}:{id}"));
    }
    pub fn record_state(&mut self, screen: &str) {
        self.inner.visit(screen);
    }
    pub fn generate_report(&self) -> CoverageReport {
        self.inner.generate_report()
    }
}

impl TuiFrame {
    pub fn matches(&self, other: &TuiFrame) -> bool {
        self.lines == other.lines
    }
}

/// Ordered list of captured frames.
pub struct FrameSequence {
    pub name: String,
    frames: Vec<TuiFrame>,
}

impl FrameSequence {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            frames: Vec::new(),
        }
    }
    pub fn add_frame(&mut self, frame: &TuiFrame) {
        self.frames.push(TuiFrame {
            lines: frame.lines.clone(),
        });
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    pub fn first(&self) -> Option<&TuiFrame> {
        self.frames.first()
    }
    pub fn last(&self) -> Option<&TuiFrame> {
        self.frames.last()
    }
}

/// Golden-file snapshots: first call records, later calls must match.
pub struct SnapshotManager {
    dir: std::path::PathBuf,
}

impl SnapshotManager {
    pub fn new(dir: &std::path::Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }
    fn path(&self, name: &str) -> std::path::PathBuf {
        self.dir.join(format!("{name}.snap"))
    }
    pub fn exists(&self, name: &str) -> bool {
        self.path(name).exists()
    }
    pub fn assert_snapshot(&self, name: &str, frame: &TuiFrame) -> Result<(), String> {
        let path = self.path(name);
        if path.exists() {
            let stored = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if stored == frame.as_text() {
                Ok(())
            } else {
                Err(format!("snapshot {name} differs from golden file"))
            }
        } else {
            std::fs::write(&path, frame.as_text()).map_err(|e| e.to_string())
        }
    }
}

/// Soft assertions: collect failures, report them together.
pub struct SoftExpect<'a> {
    frame: &'a TuiFrame,
    errors: Vec<String>,
}

impl<'a> FrameExpect<'a> {
    pub fn soft(self) -> SoftExpect<'a> {
        SoftExpect {
            frame: self.0,
            errors: Vec::new(),
        }
    }
}

impl SoftExpect<'_> {
    pub fn expect_contain_text(&mut self, text: &str) -> Result<(), String> {
        if self.frame.contains(text) {
            Ok(())
        } else {
            let e = format!("frame does not contain {text:?}");
            self.errors.push(e.clone());
            Err(e)
        }
    }
    pub fn not_to_contain_text(&mut self, text: &str) -> Result<(), String> {
        if self.frame.contains(text) {
            let e = format!("frame unexpectedly contains {text:?}");
            self.errors.push(e.clone());
            Err(e)
        } else {
            Ok(())
        }
    }
    pub fn errors(&self) -> &[String] {
        &self.errors
    }
    pub fn finalize(self) -> Result<(), String> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("; "))
        }
    }
}

impl TuiSnapshot {
    pub fn matches(&self, other: &TuiSnapshot) -> bool {
        self.hash == other.hash
    }
}
