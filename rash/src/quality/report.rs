//! Rich ASCII Reporting Module (ML-011, ML-012)
//!
//! Provides visual feedback using ASCII box drawing and sparklines
//! following Toyota Way principles of visual management (Mieruka).
//!
//! # Toyota Way Principles
//!
//! - **Mieruka** (Visual Management): Make problems visible immediately
//! - **Andon**: Signal when quality issues are detected
//! - **Genchi Genbutsu**: Go and see the actual data
//!
//! # References
//!
//! - BASHRS-SPEC-ML-011: ASCII Box Drawing
//! - BASHRS-SPEC-ML-012: Sparkline Generation
//! - Tufte, E. (2001). "The Visual Display of Quantitative Information"

use std::fmt::Write;
use std::time::Duration;

/// Characters for box drawing (Unicode box-drawing characters)
pub mod chars {
    pub const TOP_LEFT: char = '┌';
    pub const TOP_RIGHT: char = '┐';
    pub const BOTTOM_LEFT: char = '└';
    pub const BOTTOM_RIGHT: char = '┘';
    pub const HORIZONTAL: char = '─';
    pub const VERTICAL: char = '│';
    pub const T_DOWN: char = '┬';
    pub const T_UP: char = '┴';
    pub const T_RIGHT: char = '├';
    pub const T_LEFT: char = '┤';
    pub const CROSS: char = '┼';

    // Sparkline characters (Unicode block elements)
    pub const SPARK_EMPTY: char = ' ';
    pub const SPARK_1_8: char = '▁';
    pub const SPARK_2_8: char = '▂';
    pub const SPARK_3_8: char = '▃';
    pub const SPARK_4_8: char = '▄';
    pub const SPARK_5_8: char = '▅';
    pub const SPARK_6_8: char = '▆';
    pub const SPARK_7_8: char = '▇';
    pub const SPARK_FULL: char = '█';

    // Status indicators
    pub const CHECK: &str = "✓";
    pub const CROSS_MARK: &str = "✗";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const BULLET: &str = "•";
}

/// Quality grade based on score
/// Ordered from worst (F) to best (APlus) for comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    /// F (< 50%)
    F,
    /// D (50-54%)
    D,
    /// C- (55-59%)
    CMinus,
    /// C (60-64%)
    C,
    /// C+ (65-69%)
    CPlus,
    /// B- (70-74%)
    BMinus,
    /// B (75-79%)
    B,
    /// B+ (80-84%)
    BPlus,
    /// A- (85-89%)
    AMinus,
    /// A (90-94%)
    A,
    /// A+ (95-100%)
    APlus,
}

impl Grade {
    /// Calculate grade from percentage score
    pub fn from_percentage(pct: f64) -> Self {
        match pct as u32 {
            95..=100 => Grade::APlus,
            90..=94 => Grade::A,
            85..=89 => Grade::AMinus,
            80..=84 => Grade::BPlus,
            75..=79 => Grade::B,
            70..=74 => Grade::BMinus,
            65..=69 => Grade::CPlus,
            60..=64 => Grade::C,
            55..=59 => Grade::CMinus,
            50..=54 => Grade::D,
            _ => Grade::F,
        }
    }

    /// Get letter representation
    pub fn as_letter(&self) -> &'static str {
        match self {
            Grade::APlus => "A+",
            Grade::A => "A",
            Grade::AMinus => "A-",
            Grade::BPlus => "B+",
            Grade::B => "B",
            Grade::BMinus => "B-",
            Grade::CPlus => "C+",
            Grade::C => "C",
            Grade::CMinus => "C-",
            Grade::D => "D",
            Grade::F => "F",
        }
    }

    /// Check if grade is passing (C- or above)
    pub fn is_passing(&self) -> bool {
        *self >= Grade::CMinus
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_letter())
    }
}

/// Generate a sparkline from a slice of values
pub fn sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    let sparks = [
        chars::SPARK_1_8,
        chars::SPARK_2_8,
        chars::SPARK_3_8,
        chars::SPARK_4_8,
        chars::SPARK_5_8,
        chars::SPARK_6_8,
        chars::SPARK_7_8,
        chars::SPARK_FULL,
    ];

    values
        .iter()
        .map(|&v| {
            if range == 0.0 {
                sparks.get(3).copied().unwrap_or(chars::SPARK_4_8) // Middle value when all same
            } else {
                let normalized = (v - min) / range;
                let index = ((normalized * 7.0).round() as usize).min(7);
                sparks.get(index).copied().unwrap_or(chars::SPARK_FULL)
            }
        })
        .collect()
}

/// Generate a progress bar
pub fn progress_bar(current: f64, total: f64, width: usize) -> String {
    let pct = if total > 0.0 { current / total } else { 0.0 };
    let filled = ((pct * width as f64).round() as usize).min(width);
    let empty = width.saturating_sub(filled);

    format!(
        "[{}{}] {:.1}%",
        chars::SPARK_FULL.to_string().repeat(filled),
        chars::SPARK_EMPTY.to_string().repeat(empty),
        pct * 100.0
    )
}

/// Rich report builder following Toyota Way visual management
pub struct ReportBuilder {
    sections: Vec<ReportSection>,
    title: String,
    width: usize,
}

/// A section in the report
#[derive(Debug, Clone)]
pub struct ReportSection {
    pub title: String,
    pub items: Vec<ReportItem>,
}

/// An item in a report section
#[derive(Debug, Clone)]
pub enum ReportItem {
    /// Key-value pair
    KeyValue { key: String, value: String },
    /// Status indicator with message
    Status { passed: bool, message: String },
    /// Progress bar
    Progress {
        label: String,
        current: f64,
        total: f64,
    },
    /// Sparkline with label
    Sparkline { label: String, values: Vec<f64> },
    /// Table row
    TableRow { cells: Vec<String> },
    /// Plain text
    Text(String),
    /// Metric with grade
    Metric {
        name: String,
        value: f64,
        unit: String,
        grade: Grade,
    },
}

impl ReportBuilder {
    /// Create a new report builder
    pub fn new(title: &str) -> Self {
        Self {
            sections: Vec::new(),
            title: title.to_string(),
            width: 60,
        }
    }

    /// Set report width
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Add a section
    pub fn section(mut self, title: &str) -> Self {
        self.sections.push(ReportSection {
            title: title.to_string(),
            items: Vec::new(),
        });
        self
    }

    /// Add an item to the current section
    pub fn item(mut self, item: ReportItem) -> Self {
        if let Some(section) = self.sections.last_mut() {
            section.items.push(item);
        }
        self
    }

    /// Add a key-value pair
    pub fn kv(self, key: &str, value: &str) -> Self {
        self.item(ReportItem::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        })
    }

    /// Add a status item
    pub fn status(self, passed: bool, message: &str) -> Self {
        self.item(ReportItem::Status {
            passed,
            message: message.to_string(),
        })
    }

    /// Add a progress bar
    pub fn progress(self, label: &str, current: f64, total: f64) -> Self {
        self.item(ReportItem::Progress {
            label: label.to_string(),
            current,
            total,
        })
    }

    /// Add a sparkline
    pub fn sparkline(self, label: &str, values: Vec<f64>) -> Self {
        self.item(ReportItem::Sparkline {
            label: label.to_string(),
            values,
        })
    }

    /// Add a metric with grade
    pub fn metric(self, name: &str, value: f64, unit: &str) -> Self {
        let grade = Grade::from_percentage(value);
        self.item(ReportItem::Metric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            grade,
        })
    }

    /// Build the final report
    pub fn build(&self) -> RichReport {
        RichReport {
            title: self.title.clone(),
            sections: self.sections.clone(),
            width: self.width,
        }
    }
}

/// A rich visual report
#[derive(Debug, Clone)]
pub struct RichReport {
    pub title: String,
    pub sections: Vec<ReportSection>,
    pub width: usize,
}

impl RichReport {
    /// Render report to string
    pub fn render(&self) -> String {
        let mut out = String::new();

        // Title box
        self.draw_title_box(&mut out);

        // Sections
        for section in &self.sections {
            self.draw_section(&mut out, section);
        }

        // Footer
        self.draw_footer(&mut out);

        out
    }

    fn draw_title_box(&self, out: &mut String) {
        let inner_width = self.width - 2;
        let title_padding = (inner_width.saturating_sub(self.title.len())) / 2;

        // Top border
        out.push(chars::TOP_LEFT);
        for _ in 0..inner_width {
            out.push(chars::HORIZONTAL);
        }
        out.push(chars::TOP_RIGHT);
        out.push('\n');

        // Title line
        out.push(chars::VERTICAL);
        for _ in 0..title_padding {
            out.push(' ');
        }
        out.push_str(&self.title);
        for _ in 0..(inner_width - title_padding - self.title.len()) {
            out.push(' ');
        }
        out.push(chars::VERTICAL);
        out.push('\n');

        // Bottom border
        out.push(chars::BOTTOM_LEFT);
        for _ in 0..inner_width {
            out.push(chars::HORIZONTAL);
        }
        out.push(chars::BOTTOM_RIGHT);
        out.push('\n');
    }

    fn draw_section(&self, out: &mut String, section: &ReportSection) {
        // Section header
        out.push('\n');
        let _ = writeln!(
            out,
            "{} {} {}",
            chars::T_RIGHT,
            section.title,
            chars::HORIZONTAL
                .to_string()
                .repeat(self.width.saturating_sub(section.title.len() + 4))
        );

        // Items
        for item in &section.items {
            self.draw_item(out, item);
        }
    }

    fn draw_item(&self, out: &mut String, item: &ReportItem) {
        match item {
            ReportItem::KeyValue { key, value } => {
                let _ = writeln!(out, "  {}: {}", key, value);
            }
            ReportItem::Status { passed, message } => {
                let icon = if *passed {
                    chars::CHECK
                } else {
                    chars::CROSS_MARK
                };
                let _ = writeln!(out, "  {} {}", icon, message);
            }
            ReportItem::Progress {
                label,
                current,
                total,
            } => {
                let bar = progress_bar(*current, *total, 20);
                let _ = writeln!(out, "  {}: {}", label, bar);
            }
            ReportItem::Sparkline { label, values } => {
                let spark = sparkline(values);
                let _ = writeln!(out, "  {}: {}", label, spark);
            }
            ReportItem::TableRow { cells } => {
                let _ = writeln!(out, "  {}", cells.join(" │ "));
            }
            ReportItem::Text(text) => {
                let _ = writeln!(out, "  {}", text);
            }
            ReportItem::Metric {
                name,
                value,
                unit,
                grade,
            } => {
                let icon = if grade.is_passing() {
                    chars::CHECK
                } else {
                    chars::CROSS_MARK
                };
                let _ = writeln!(out, "  {} {} {:.1}{} ({})", icon, name, value, unit, grade);
            }
        }
    }

    fn draw_footer(&self, out: &mut String) {
        out.push('\n');
        for _ in 0..self.width {
            out.push(chars::HORIZONTAL);
        }
        out.push('\n');
        let _ = writeln!(out, "Generated by bashrs v{}", env!("CARGO_PKG_VERSION"));
    }
}

/// Format duration in human-readable form
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let millis = d.subsec_millis();

    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs > 0 {
        format!("{}.{}s", secs, millis / 100)
    } else {
        format!("{}ms", millis)
    }
}

/// Create a quality gate report
pub fn gate_report(
    tier: u8,
    gates_passed: usize,
    gates_total: usize,
    duration: Duration,
    details: &[(String, bool, Duration)],
) -> String {
    let mut builder = ReportBuilder::new(&format!("Tier {} Quality Gate Report", tier))
        .width(60)
        .section("Summary");

    let pct = if gates_total > 0 {
        (gates_passed as f64 / gates_total as f64) * 100.0
    } else {
        100.0
    };

    builder = builder
        .metric("Pass Rate", pct, "%")
        .kv("Gates Passed", &format!("{}/{}", gates_passed, gates_total))
        .kv("Total Duration", &format_duration(duration));

    builder = builder.section("Gate Results");

    for (name, passed, dur) in details {
        builder = builder.status(*passed, &format!("{} ({})", name, format_duration(*dur)));
    }

    builder.build().render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_011_sparkline_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let spark = sparkline(&values);

        assert_eq!(spark.chars().count(), 8);
        assert!(spark.contains(chars::SPARK_1_8));
        assert!(spark.contains(chars::SPARK_FULL));
    }

    #[test]
    fn test_ml_011_sparkline_empty() {
        let values: Vec<f64> = vec![];
        let spark = sparkline(&values);
        assert!(spark.is_empty());
    }

    #[test]
    fn test_ml_011_sparkline_constant() {
        let values = vec![5.0, 5.0, 5.0, 5.0];
        let spark = sparkline(&values);

        // All same value should produce middle-height bars
        assert_eq!(spark.chars().count(), 4);
    }

    #[test]
    fn test_ml_011_progress_bar() {
        let bar = progress_bar(50.0, 100.0, 10);
        assert!(bar.contains("50.0%"));
        assert!(bar.starts_with('['));
        assert!(bar.contains(']'));
    }

    #[test]
    fn test_ml_012_grade_from_percentage() {
        assert_eq!(Grade::from_percentage(95.0), Grade::APlus);
        assert_eq!(Grade::from_percentage(90.0), Grade::A);
        assert_eq!(Grade::from_percentage(85.0), Grade::AMinus);
        assert_eq!(Grade::from_percentage(80.0), Grade::BPlus);
        assert_eq!(Grade::from_percentage(70.0), Grade::BMinus);
        assert_eq!(Grade::from_percentage(60.0), Grade::C);
        assert_eq!(Grade::from_percentage(40.0), Grade::F);
    }

    #[test]
    fn test_ml_012_grade_passing() {
        assert!(Grade::APlus.is_passing());
        assert!(Grade::C.is_passing());
        assert!(Grade::CMinus.is_passing());
        assert!(!Grade::D.is_passing());
        assert!(!Grade::F.is_passing());
    }

    #[test]
    fn test_ml_011_report_builder() {
        let report = ReportBuilder::new("Test Report")
            .width(50)
            .section("Results")
            .status(true, "All tests passed")
            .kv("Coverage", "91.2%")
            .progress("Completion", 45.0, 50.0)
            .build();

        let output = report.render();

        assert!(output.contains("Test Report"));
        assert!(output.contains("Results"));
        assert!(output.contains("All tests passed"));
        assert!(output.contains("Coverage"));
        assert!(output.contains("91.2%"));
    }

    #[test]
    fn test_ml_011_gate_report() {
        let details = vec![
            ("clippy".to_string(), true, Duration::from_millis(500)),
            ("tests".to_string(), true, Duration::from_secs(30)),
            ("coverage".to_string(), false, Duration::from_secs(45)),
        ];

        let report = gate_report(2, 2, 3, Duration::from_secs(76), &details);

        assert!(report.contains("Tier 2"));
        assert!(report.contains("Quality Gate"));
        assert!(report.contains("2/3"));
        assert!(report.contains("clippy"));
        assert!(report.contains("coverage"));
    }

    #[test]
    fn test_ml_011_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(5)), "5.0s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3700)), "1h 1m");
    }
}
