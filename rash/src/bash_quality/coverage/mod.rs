// coverage/mod.rs - Coverage tracking for bash scripts
// Part of Bash Quality Tools (v6.13.0)

use std::collections::{HashMap, HashSet};

/// Coverage report for a bash script
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Total lines in the script (excluding comments and empty lines)
    pub total_lines: usize,

    /// Lines that were executed during tests
    pub covered_lines: HashSet<usize>,

    /// All functions defined in the script
    pub all_functions: Vec<String>,

    /// Functions that were executed during tests
    pub covered_functions: HashSet<String>,

    /// Line-by-line coverage map (line number -> covered)
    pub line_coverage: HashMap<usize, bool>,
}

impl CoverageReport {
    /// Create a new empty coverage report
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            covered_lines: HashSet::new(),
            all_functions: Vec::new(),
            covered_functions: HashSet::new(),
            line_coverage: HashMap::new(),
        }
    }

    /// Calculate line coverage percentage
    pub fn line_coverage_percent(&self) -> f64 {
        if self.total_lines == 0 {
            return 0.0;
        }
        (self.covered_lines.len() as f64 / self.total_lines as f64) * 100.0
    }

    /// Calculate function coverage percentage
    pub fn function_coverage_percent(&self) -> f64 {
        if self.all_functions.is_empty() {
            return 0.0;
        }
        (self.covered_functions.len() as f64 / self.all_functions.len() as f64) * 100.0
    }

    /// Get uncovered line numbers
    pub fn uncovered_lines(&self) -> Vec<usize> {
        let mut uncovered: Vec<usize> = self
            .line_coverage
            .iter()
            .filter(|(_, &covered)| !covered)
            .map(|(line, _)| *line)
            .collect();
        uncovered.sort();
        uncovered
    }

    /// Get uncovered function names
    pub fn uncovered_functions(&self) -> Vec<String> {
        self.all_functions
            .iter()
            .filter(|func| !self.covered_functions.contains(*func))
            .cloned()
            .collect()
    }
}

impl Default for CoverageReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate coverage report by analyzing script and running tests
pub fn generate_coverage(source: &str) -> Result<CoverageReport, String> {
    use crate::bash_quality::testing::{discover_tests, run_tests};

    let mut report = CoverageReport::new();

    // Step 1: Analyze the script to find all executable lines and functions
    analyze_script(source, &mut report);

    // Step 2: Discover and run tests to track coverage
    let tests = discover_tests(source).map_err(|e| format!("Failed to discover tests: {}", e))?;

    if tests.is_empty() {
        // No tests = no coverage
        return Ok(report);
    }

    // Step 3: Mark functions called at top level as covered
    mark_top_level_called_functions(source, &mut report);

    // Step 4: Run tests and track which functions are called
    match run_tests(source, &tests) {
        Ok(_test_report) => {
            // Mark functions as covered if they have tests
            for test in &tests {
                // Extract function name from test name (test_xxx tests xxx)
                let tested_func = test.name.strip_prefix("test_").unwrap_or(&test.name);
                if report.all_functions.iter().any(|f| tested_func.contains(f)) {
                    for func in &report.all_functions {
                        if tested_func.contains(func) {
                            report.covered_functions.insert(func.clone());
                        }
                    }
                }
            }

            // For now, assume all lines in covered functions are covered
            // This is a simplification - real coverage would trace actual execution
            let covered_funcs = report.covered_functions.clone();
            mark_covered_functions_lines(source, &covered_funcs, &mut report);
        }
        Err(_) => {
            // Tests failed to run - return zero coverage
        }
    }

    Ok(report)
}

/// Analyze script to find all executable lines and functions
fn analyze_script(source: &str, report: &mut CoverageReport) {
    let mut line_num = 0;
    let mut in_function = false;
    let mut _current_function: Option<String> = None;

    for line in source.lines() {
        line_num += 1;
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip shebang
        if trimmed.starts_with("#!") {
            continue;
        }

        // Detect function definitions
        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            in_function = true;
            // Extract function name
            let func_name = if let Some(idx) = trimmed.find("() {") {
                trimmed[..idx].trim().to_string()
            } else if trimmed.starts_with("function ") {
                #[allow(clippy::expect_used)] // Safe: checked by starts_with() above
                trimmed
                    .strip_prefix("function ")
                    .expect("checked by starts_with")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string()
            } else {
                "unknown".to_string()
            };

            // Don't track test functions as regular functions
            if !func_name.starts_with("test_") {
                report.all_functions.push(func_name.clone());
                _current_function = Some(func_name);
            }
        }

        // Detect function end
        if in_function && trimmed == "}" {
            in_function = false;
            _current_function = None;
        }

        // Count this as an executable line
        report.total_lines += 1;
        report.line_coverage.insert(line_num, false);
    }
}

/// Mark lines in covered functions as covered
fn mark_covered_functions_lines(
    source: &str,
    covered_functions: &HashSet<String>,
    report: &mut CoverageReport,
) {
    let mut line_num = 0;
    let mut current_function: Option<String> = None;
    let mut in_covered_function = false;

    for line in source.lines() {
        line_num += 1;
        let trimmed = line.trim();

        // Detect function start
        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            let func_name = if let Some(idx) = trimmed.find("() {") {
                trimmed[..idx].trim().to_string()
            } else if trimmed.starts_with("function ") {
                #[allow(clippy::expect_used)] // Safe: checked by starts_with() above
                trimmed
                    .strip_prefix("function ")
                    .expect("checked by starts_with")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string()
            } else {
                "unknown".to_string()
            };

            current_function = Some(func_name.clone());
            in_covered_function = covered_functions.contains(&func_name);
        }

        // Detect function end
        if current_function.is_some() && trimmed == "}" {
            current_function = None;
            in_covered_function = false;
        }

        // Mark line as covered if in a covered function
        if in_covered_function && report.line_coverage.contains_key(&line_num) {
            report.line_coverage.insert(line_num, true);
            report.covered_lines.insert(line_num);
        }

        // Also mark lines outside functions as covered if they're executed in tests
        if current_function.is_none() && !trimmed.is_empty() && !trimmed.starts_with('#') {
            // Assume top-level code is executed
            if let std::collections::hash_map::Entry::Occupied(mut e) =
                report.line_coverage.entry(line_num)
            {
                e.insert(true);
                report.covered_lines.insert(line_num);
            }
        }
    }
}

/// Mark functions that are called at the top level (outside function definitions) as covered
fn mark_top_level_called_functions(source: &str, report: &mut CoverageReport) {
    let mut in_function = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Detect function start
        if trimmed.contains("() {") || trimmed.starts_with("function ") {
            in_function = true;
        }

        // Detect function end
        if in_function && trimmed == "}" {
            in_function = false;
            continue;
        }

        // If we're at top level (not in a function), check for function calls
        if !in_function {
            // Check if any of our functions are called on this line
            for func_name in &report.all_functions {
                // Simple check: if the function name appears as a word on this line
                if trimmed.contains(func_name) {
                    // More precise check: ensure it's not inside a comment or string
                    let words: Vec<&str> = trimmed.split_whitespace().collect();
                    for word in words {
                        if word == func_name || word.starts_with(&format!("{}(", func_name)) {
                            report.covered_functions.insert(func_name.clone());
                            break;
                        }
                    }
                }
            }
        }
    }
}
