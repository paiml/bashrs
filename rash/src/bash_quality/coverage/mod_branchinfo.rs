// coverage/mod.rs - Coverage tracking for bash scripts
// Part of Bash Quality Tools (v6.13.0)

use std::collections::{HashMap, HashSet};

/// A branch point detected in a bash script.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Line number where the branch starts
    pub line: usize,
    /// Type of branch construct
    pub kind: BranchKind,
    /// Whether this branch arm was taken during test execution
    pub taken: bool,
}

/// Types of branch constructs in bash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKind {
    /// `if` condition (then branch)
    IfThen,
    /// `elif` condition
    Elif,
    /// `else` branch
    Else,
    /// `case` pattern match
    CasePattern,
    /// `while` loop body
    While,
    /// `for` loop body
    For,
}

impl std::fmt::Display for BranchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchKind::IfThen => write!(f, "if/then"),
            BranchKind::Elif => write!(f, "elif"),
            BranchKind::Else => write!(f, "else"),
            BranchKind::CasePattern => write!(f, "case"),
            BranchKind::While => write!(f, "while"),
            BranchKind::For => write!(f, "for"),
        }
    }
}

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

    /// All branch points detected in the script
    pub branches: Vec<BranchInfo>,
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
            branches: Vec::new(),
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

    /// Calculate branch coverage percentage
    pub fn branch_coverage_percent(&self) -> f64 {
        if self.branches.is_empty() {
            return 0.0;
        }
        let taken = self.branches.iter().filter(|b| b.taken).count();
        (taken as f64 / self.branches.len() as f64) * 100.0
    }

    /// Total number of branches detected
    pub fn total_branches(&self) -> usize {
        self.branches.len()
    }

    /// Number of branches taken during execution
    pub fn covered_branches(&self) -> usize {
        self.branches.iter().filter(|b| b.taken).count()
    }

    /// Get untaken branches
    pub fn untaken_branches(&self) -> Vec<&BranchInfo> {
        self.branches.iter().filter(|b| !b.taken).collect()
    }

    /// Get uncovered line numbers
    pub fn uncovered_lines(&self) -> Vec<usize> {
        let mut uncovered: Vec<usize> = self
            .line_coverage
            .iter()
            .filter(|(_, &covered)| !covered)
            .map(|(line, _)| *line)
            .collect();
        uncovered.sort_unstable();
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

/// Analyze script to find all executable lines, functions, and branches
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

        // Detect branch points
        detect_branches(trimmed, line_num, report);

        // Count this as an executable line
        report.total_lines += 1;
        report.line_coverage.insert(line_num, false);
    }
}

/// Detect branch points (if/elif/else/case/while/for) on a line
fn detect_branches(trimmed: &str, line_num: usize, report: &mut CoverageReport) {
    // if ... then (or `if ...;then` or `if [`)
    if trimmed.starts_with("if ")
        || trimmed.starts_with("if [")
        || trimmed == "if"
    {
        report.branches.push(BranchInfo {
            line: line_num,
            kind: BranchKind::IfThen,
            taken: false,
        });
    }

    // elif
    if trimmed.starts_with("elif ") || trimmed.starts_with("elif [") {
        report.branches.push(BranchInfo {
            line: line_num,
            kind: BranchKind::Elif,
            taken: false,
        });
    }

    // else
    if trimmed == "else" || trimmed == "else;" {
        report.branches.push(BranchInfo {
            line: line_num,
            kind: BranchKind::Else,
            taken: false,
        });
    }

    // case pattern: ends with ) but not ;;) or esac
    if trimmed.ends_with(')')
        && !trimmed.starts_with("esac")
        && !trimmed.starts_with(";;")
        && !trimmed.starts_with("case ")
    {
        report.branches.push(BranchInfo {
            line: line_num,
            kind: BranchKind::CasePattern,
            taken: false,
        });
    }

    // while loop
    if trimmed.starts_with("while ") || trimmed == "while" {
        report.branches.push(BranchInfo {
            line: line_num,
            kind: BranchKind::While,
            taken: false,
        });
    }

    // for loop
    if trimmed.starts_with("for ") {
        report.branches.push(BranchInfo {
            line: line_num,
            kind: BranchKind::For,
            taken: false,
        });
    }
}

/// Check if line starts a function definition
fn is_function_start(trimmed: &str) -> bool {
    trimmed.contains("() {") || trimmed.starts_with("function ")
}

/// Extract function name from function definition line
fn extract_function_name(trimmed: &str) -> String {
    if let Some(idx) = trimmed.find("() {") {
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
    }
}

/// Check if line is a function end
fn is_function_end(trimmed: &str) -> bool {
    trimmed == "}"
}

/// Check if line is top-level executable code
fn is_top_level_code(trimmed: &str) -> bool {
    !trimmed.is_empty() && !trimmed.starts_with('#')
}

/// Mark line as covered in report
fn mark_line_covered(line_num: usize, report: &mut CoverageReport) {
    report.line_coverage.insert(line_num, true);
    report.covered_lines.insert(line_num);
}

/// Mark lines in covered functions as covered, and mark branches as taken
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
        if is_function_start(trimmed) {
            let func_name = extract_function_name(trimmed);
            current_function = Some(func_name.clone());
            in_covered_function = covered_functions.contains(&func_name);
        }

        // Detect function end
        if current_function.is_some() && is_function_end(trimmed) {
            current_function = None;
            in_covered_function = false;
        }

        // Mark line as covered if in a covered function
        if in_covered_function && report.line_coverage.contains_key(&line_num) {
            mark_line_covered(line_num, report);
            // Also mark any branch on this line as taken
            for branch in &mut report.branches {
                if branch.line == line_num {
                    branch.taken = true;
                }
            }
        }

        // Also mark lines outside functions as covered if they're executed in tests
        if current_function.is_none() && is_top_level_code(trimmed) {
            if let std::collections::hash_map::Entry::Occupied(mut e) =
                report.line_coverage.entry(line_num)
            {
                e.insert(true);
                report.covered_lines.insert(line_num);
            }
            // Mark top-level branches as taken
            for branch in &mut report.branches {
                if branch.line == line_num {
                    branch.taken = true;
                }
            }
        }
    }
}

/// Check if line should be skipped (empty or comment)
fn should_skip_line(trimmed: &str) -> bool {
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Check if line indicates function start
fn is_function_start_line(trimmed: &str) -> bool {
    trimmed.contains("() {") || trimmed.starts_with("function ")
}

/// Check if we should exit function scope
fn should_exit_function(trimmed: &str, in_function: bool) -> bool {
    in_function && trimmed == "}"
}

/// Check if a word is a function call (exact match or with parentheses)
fn is_function_call(word: &str, func_name: &str) -> bool {
    word == func_name || word.starts_with(&format!("{}(", func_name))
}

/// Mark function calls found on this line as covered
fn mark_function_calls_on_line(trimmed: &str, report: &mut CoverageReport) {
    // Check if any of our functions are called on this line
    for func_name in &report.all_functions {
        // Simple check: if the function name appears as a word on this line
        if trimmed.contains(func_name) {
            // More precise check: ensure it's not inside a comment or string
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            for word in words {
                if is_function_call(word, func_name) {
                    report.covered_functions.insert(func_name.clone());
                    break;
                }
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
        if should_skip_line(trimmed) {
            continue;
        }

        // Detect function start
        if is_function_start_line(trimmed) {
            in_function = true;
        }

        // Detect function end
        if should_exit_function(trimmed, in_function) {
            in_function = false;
            continue;
        }


}
}

                include!("mod_part2_incl2.rs");
