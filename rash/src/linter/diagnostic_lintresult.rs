
/// Collection of lint diagnostics for a file or project.
///
/// `LintResult` aggregates all linting findings and provides utilities
/// for querying by severity, counting issues, and merging results.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
///
/// let mut result = LintResult::new();
/// let span = Span::new(1, 5, 1, 10);
///
/// result.add(Diagnostic::new("SC2086", Severity::Warning, "Quote variable", span));
/// result.add(Diagnostic::new("SC2046", Severity::Error, "Quote command", span));
///
/// assert_eq!(result.diagnostics.len(), 2);
/// assert!(result.has_errors());
/// assert!(result.has_warnings());
/// ```
///
/// ## Merging results
///
/// ```
/// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
///
/// let mut result1 = LintResult::new();
/// let mut result2 = LintResult::new();
/// let span = Span::new(1, 1, 1, 5);
///
/// result1.add(Diagnostic::new("SC2086", Severity::Warning, "Test 1", span));
/// result2.add(Diagnostic::new("SC2046", Severity::Warning, "Test 2", span));
///
/// result1.merge(result2);
/// assert_eq!(result1.diagnostics.len(), 2);
/// ```
///
/// ## Severity analysis
///
/// ```
/// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
///
/// let mut result = LintResult::new();
/// let span = Span::new(1, 1, 1, 5);
///
/// result.add(Diagnostic::new("W1", Severity::Warning, "Warning", span));
/// result.add(Diagnostic::new("E1", Severity::Error, "Error", span));
///
/// assert_eq!(result.count_by_severity(Severity::Warning), 1);
/// assert_eq!(result.count_by_severity(Severity::Error), 1);
/// assert_eq!(result.max_severity(), Some(Severity::Error));
/// ```
#[derive(Debug, Clone, Default)]

pub struct LintResult {
    /// All diagnostics found during linting.
    pub diagnostics: Vec<Diagnostic>,
}

impl LintResult {
    /// Creates an empty lint result.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::LintResult;
    ///
    /// let result = LintResult::new();
    /// assert_eq!(result.diagnostics.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic to this result.
    ///
    /// # Arguments
    ///
    /// * `diagnostic` - The diagnostic to add
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
    ///
    /// let mut result = LintResult::new();
    /// let span = Span::new(1, 1, 1, 5);
    /// let diag = Diagnostic::new("SC2086", Severity::Warning, "Test", span);
    ///
    /// result.add(diag);
    /// assert_eq!(result.diagnostics.len(), 1);
    /// ```
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Merges another result into this one.
    ///
    /// All diagnostics from `other` are appended to this result.
    ///
    /// # Arguments
    ///
    /// * `other` - The result to merge
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
    ///
    /// let mut result1 = LintResult::new();
    /// let mut result2 = LintResult::new();
    /// let span = Span::new(1, 1, 1, 5);
    ///
    /// result1.add(Diagnostic::new("SC2086", Severity::Warning, "Test 1", span));
    /// result2.add(Diagnostic::new("SC2046", Severity::Warning, "Test 2", span));
    ///
    /// result1.merge(result2);
    /// assert_eq!(result1.diagnostics.len(), 2);
    /// ```
    pub fn merge(&mut self, other: LintResult) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Checks if there are any errors (Severity::Error).
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
    ///
    /// let mut result = LintResult::new();
    /// assert!(!result.has_errors());
    ///
    /// let span = Span::new(1, 1, 1, 5);
    /// result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));
    /// assert!(!result.has_errors());
    ///
    /// result.add(Diagnostic::new("SC2046", Severity::Error, "Test", span));
    /// assert!(result.has_errors());
    /// ```
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Checks if there are any warnings (Severity::Warning).
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
    ///
    /// let mut result = LintResult::new();
    /// assert!(!result.has_warnings());
    ///
    /// let span = Span::new(1, 1, 1, 5);
    /// result.add(Diagnostic::new("SC2086", Severity::Info, "Test", span));
    /// assert!(!result.has_warnings());
    ///
    /// result.add(Diagnostic::new("SC2046", Severity::Warning, "Test", span));
    /// assert!(result.has_warnings());
    /// ```
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning)
    }

    /// Counts diagnostics by severity level.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level to count
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
    ///
    /// let mut result = LintResult::new();
    /// let span = Span::new(1, 1, 1, 5);
    ///
    /// result.add(Diagnostic::new("W1", Severity::Warning, "Test", span));
    /// result.add(Diagnostic::new("W2", Severity::Warning, "Test", span));
    /// result.add(Diagnostic::new("E1", Severity::Error, "Test", span));
    ///
    /// assert_eq!(result.count_by_severity(Severity::Warning), 2);
    /// assert_eq!(result.count_by_severity(Severity::Error), 1);
    /// assert_eq!(result.count_by_severity(Severity::Info), 0);
    /// ```
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .count()
    }

    /// Gets the highest severity level present.
    ///
    /// Returns `None` if there are no diagnostics.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
    ///
    /// let mut result = LintResult::new();
    /// assert_eq!(result.max_severity(), None);
    ///
    /// let span = Span::new(1, 1, 1, 5);
    /// result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));
    /// assert_eq!(result.max_severity(), Some(Severity::Warning));
    ///
    /// result.add(Diagnostic::new("SC2046", Severity::Error, "Test", span));
    /// assert_eq!(result.max_severity(), Some(Severity::Error));
    /// ```
    pub fn max_severity(&self) -> Option<Severity> {
        self.diagnostics.iter().map(|d| d.severity).max()
    }
}

#[cfg(test)]
#[path = "diagnostic_tests_span_creatio.rs"]
mod tests_extracted;
