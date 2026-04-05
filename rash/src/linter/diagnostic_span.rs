//! Diagnostic types for linting
//!
//! Core types for representing lint violations, warnings, and suggested fixes.
//!
//! # Examples
//!
//! ## Creating a diagnostic
//!
//! ```
//! use bashrs::linter::{Diagnostic, Severity, Span};
//!
//! let span = Span::new(1, 5, 1, 10);
//! let diag = Diagnostic::new(
//!     "SC2086",
//!     Severity::Warning,
//!     "Double quote to prevent globbing",
//!     span,
//! );
//! println!("{}", diag); // "1:5-10 warning [SC2086] Double quote to prevent globbing"
//! ```
//!
//! ## Creating a diagnostic with a fix
//!
//! ```
//! use bashrs::linter::{Diagnostic, Fix, Severity, Span};
//!
//! let span = Span::new(1, 5, 1, 10);
//! let fix = Fix::new("\"$var\"");
//! let diag = Diagnostic::new("SC2086", Severity::Warning, "Double quote", span)
//!     .with_fix(fix);
//! assert!(diag.fix.is_some());
//! ```

use std::fmt;

/// A source code location span (1-indexed line and column numbers).
///
/// Represents a contiguous region in source code, from start position to end position.
/// All line and column numbers are 1-indexed to match standard editor conventions.
///
/// # Examples
///
/// ## Creating a span for a range
///
/// ```
/// use bashrs::linter::Span;
///
/// // Span from line 1, column 5 to line 1, column 10
/// let span = Span::new(1, 5, 1, 10);
/// assert_eq!(span.to_string(), "1:5-10");
/// ```
///
/// ## Creating a span for a single point
///
/// ```
/// use bashrs::linter::Span;
///
/// // Span at line 5, column 10 (zero-width)
/// let span = Span::point(5, 10);
/// assert_eq!(span.to_string(), "5:10-10");
/// ```
///
/// ## Multi-line spans
///
/// ```
/// use bashrs::linter::Span;
///
/// // Span from line 1, column 5 to line 3, column 10
/// let span = Span::new(1, 5, 3, 10);
/// assert_eq!(span.to_string(), "1:5-3:10");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start line (1-indexed)
    pub start_line: usize,
    /// Start column (1-indexed)
    pub start_col: usize,
    /// End line (1-indexed)
    pub end_line: usize,
    /// End column (1-indexed)
    pub end_col: usize,
}

impl Span {
    /// Creates a new span from start to end positions.
    ///
    /// # Arguments
    ///
    /// * `start_line` - Starting line number (1-indexed)
    /// * `start_col` - Starting column number (1-indexed)
    /// * `end_line` - Ending line number (1-indexed)
    /// * `end_col` - Ending column number (1-indexed)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Span;
    ///
    /// let span = Span::new(1, 5, 1, 10);
    /// assert_eq!(span.start_line, 1);
    /// assert_eq!(span.start_col, 5);
    /// assert_eq!(span.end_line, 1);
    /// assert_eq!(span.end_col, 10);
    /// ```
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Creates a zero-width span at a single point.
    ///
    /// Useful for diagnostics that refer to a specific location without a range.
    ///
    /// # Arguments
    ///
    /// * `line` - Line number (1-indexed)
    /// * `col` - Column number (1-indexed)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Span;
    ///
    /// let span = Span::point(5, 10);
    /// assert_eq!(span.start_line, 5);
    /// assert_eq!(span.start_col, 10);
    /// assert_eq!(span.end_line, 5);
    /// assert_eq!(span.end_col, 10);
    /// ```
    pub fn point(line: usize, col: usize) -> Self {
        Self::new(line, col, line, col)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_line == self.end_line {
            write!(f, "{}:{}-{}", self.start_line, self.start_col, self.end_col)
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.start_line, self.start_col, self.end_line, self.end_col
            )
        }
    }
}

/// Severity level of a diagnostic.
///
/// Severity levels are ordered from least to most severe:
/// `Info < Note < Perf < Risk < Warning < Error`
///
/// This ordering allows filtering and prioritization of diagnostics.
///
/// # Examples
///
/// ## Comparing severities
///
/// ```
/// use bashrs::linter::Severity;
///
/// assert!(Severity::Info < Severity::Warning);
/// assert!(Severity::Warning < Severity::Error);
/// assert_eq!(Severity::Error.to_string(), "error");
/// ```
///
/// ## Filtering diagnostics by severity
///
/// ```
/// use bashrs::linter::{Diagnostic, LintResult, Severity, Span};
///
/// let mut result = LintResult::new();
/// let span = Span::new(1, 1, 1, 5);
///
/// result.add(Diagnostic::new("INFO001", Severity::Info, "Style", span));
/// result.add(Diagnostic::new("WARN001", Severity::Warning, "Issue", span));
///
/// // Count only warnings and above
/// let serious = result.diagnostics.iter()
///     .filter(|d| d.severity >= Severity::Warning)
///     .count();
/// assert_eq!(serious, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational message (style suggestions, best practices).
    ///
    /// These are optional improvements that don't affect correctness.
    /// Example: "Consider using double brackets for better readability"
    Info,

    /// Suggestion or note (helpful context or alternative approaches).
    ///
    /// These provide additional context or suggest alternatives.
    /// Example: "Note: This could be simplified using parameter expansion"
    Note,

    /// Performance anti-pattern (not critical but affects efficiency).
    ///
    /// These indicate inefficient code that works correctly but could be optimized.
    /// Example: "Using external command in loop (performance impact)"
    Perf,

    /// Risk of potential runtime failure (context-dependent).
    ///
    /// These indicate patterns that might fail in some contexts.
    /// Example: "Variable may be unset in some environments"
    Risk,

    /// Warning (likely bug that should be fixed).
    ///
    /// These indicate probable bugs that should be addressed.
    /// Example: "Unquoted variable expansion may cause word splitting"
    Warning,

    /// Error (definite syntax or semantic error that must be fixed).
    ///
    /// These indicate code that will definitely fail or cause problems.
    /// Example: "Syntax error: unexpected token 'then'"
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Note => write!(f, "note"),
            Severity::Perf => write!(f, "perf"),
            Severity::Risk => write!(f, "risk"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Fix safety level for automatic code repairs (following APR research best practices).
///
/// Based on peer-reviewed research in Automated Program Repair:
/// - Le et al. (2017): S3: Syntax- and Semantic-Guided Repair Synthesis
/// - Monperrus (2018): Automatic Software Repair: A Bibliography
///
/// Key insight: **Plausible patches ≠ Correct patches**
///
/// bashrs uses a conservative, research-backed approach to automatic fixes:
/// - **Safe**: Guaranteed semantic preservation (applied with `--fix`)
/// - **SafeWithAssumptions**: Preserved under documented assumptions (requires `--fix --fix-assumptions`)
/// - **Unsafe**: Human decision required (suggestions only, never auto-applied)
///
/// # Examples
///
/// ## Safe fix (automatic with `--fix`)
///
/// ```
/// use bashrs::linter::{Fix, FixSafetyLevel};
///
/// // Quoting a variable is always safe
/// let fix = Fix::new("\"$var\"");
/// assert!(fix.is_safe());
/// assert_eq!(fix.safety_level, FixSafetyLevel::Safe);
/// ```
///
/// ## Safe-with-assumptions fix (requires explicit opt-in)
///
/// ```
/// use bashrs::linter::{Fix, FixSafetyLevel};
///
/// // mkdir -p is safe assuming the directory doesn't need special permissions
/// let fix = Fix::new_with_assumptions(
///     "mkdir -p /tmp/mydir",
///     vec!["Assumes no special permissions needed".to_string()]
/// );
/// assert!(fix.is_safe_with_assumptions());
/// assert_eq!(fix.safety_level, FixSafetyLevel::SafeWithAssumptions);
/// ```
///
/// ## Unsafe fix (suggestions only)
///
/// ```
/// use bashrs::linter::{Fix, FixSafetyLevel};
///
/// // Requires understanding developer intent
/// let fix = Fix::new_unsafe(vec![
///     "Option 1: Use rm -f for idempotency".to_string(),
///     "Option 2: Add error handling".to_string(),
/// ]);
/// assert!(fix.is_unsafe());
/// assert_eq!(fix.safety_level, FixSafetyLevel::Unsafe);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixSafetyLevel {
    /// SAFE: Semantic preservation guaranteed.
    ///
    /// Criteria:
    /// - No change to control flow
    /// - No change to data flow
    /// - No change to observable side effects
    /// - Equivalent AST modulo formatting/style
    ///
    /// Examples: Quoting variables (SC2086), formatting whitespace
    ///
    /// Applied by: `--fix` (default)
    Safe,

    /// SAFE-WITH-ASSUMPTIONS: Semantic preservation under documented assumptions.
    ///
    /// Criteria:
    /// - Semantics preserved for 95%+ of real-world usage
    /// - Edge cases are well-documented
    /// - Failure mode is fail-safe (errors become explicit, not silent)
    ///
    /// Examples: `rm -f` (IDEM002), `mkdir -p` (IDEM001)
    ///
    /// Applied by: `--fix --fix-assumptions` (explicit opt-in)
    SafeWithAssumptions,

    /// UNSAFE: Semantic transformation required.
    ///
    /// Criteria:
    /// - Changes control flow or data flow
    /// - Adds or removes operations
    /// - Requires understanding of developer intent
    ///
    /// Examples: IDEM003 (adds `rm -f`), DET001 (needs intent)
    ///
    /// Applied by: NEVER (human-only, provides suggestions)
    Unsafe,
}

impl fmt::Display for FixSafetyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixSafetyLevel::Safe => write!(f, "safe"),
            FixSafetyLevel::SafeWithAssumptions => write!(f, "safe-with-assumptions"),
            FixSafetyLevel::Unsafe => write!(f, "unsafe"),
        }
    }
}

/// A suggested fix for a diagnostic with safety guarantees.
///
/// `Fix` represents a proposed code change with explicit safety levels:
/// - **Safe**: Automatic application preserves semantics
/// - **SafeWithAssumptions**: Automatic application requires documented assumptions
/// - **Unsafe**: Human review required (suggestions only)
///
/// # Examples
///
/// ## Creating a safe fix
///
/// ```
/// use bashrs::linter::Fix;
///
/// let fix = Fix::new("\"$var\"");
/// assert!(fix.is_safe());
/// assert_eq!(fix.replacement, "\"$var\"");
/// ```
///
/// ## Creating a fix with assumptions
///
/// ```
/// use bashrs::linter::Fix;
///
/// let fix = Fix::new_with_assumptions(
///     "mkdir -p /tmp/dir",
///     vec!["Directory does not require special permissions".to_string()]
/// );
/// assert!(fix.is_safe_with_assumptions());
/// assert_eq!(fix.assumptions.len(), 1);
/// ```
///
/// ## Creating an unsafe fix (suggestions only)
///
/// ```
/// use bashrs::linter::Fix;
///
/// let fix = Fix::new_unsafe(vec![
///     "Option 1: Add error handling".to_string(),
///     "Option 2: Use set -e".to_string(),
/// ]);
/// assert!(fix.is_unsafe());
/// assert_eq!(fix.suggested_alternatives.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    /// The replacement text (empty for unsafe fixes).
    pub replacement: String,

    /// Safety level of this fix.
    pub safety_level: FixSafetyLevel,

    /// Assumptions required for this fix (for SafeWithAssumptions).
    pub assumptions: Vec<String>,

    /// Alternative suggested fixes (for Unsafe - human must choose).
    pub suggested_alternatives: Vec<String>,
}

impl Fix {
    /// Creates a new SAFE fix with guaranteed semantic preservation.
    ///
    /// Safe fixes can be applied automatically with `--fix` because they
    /// preserve program semantics (equivalent AST modulo formatting).
    ///
    /// # Arguments
    ///
    /// * `replacement` - The replacement text
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Fix;
    ///
    /// let fix = Fix::new("\"$var\"");
    /// assert!(fix.is_safe());
    /// ```
    pub fn new(replacement: impl Into<String>) -> Self {
        Self {
            replacement: replacement.into(),
            safety_level: FixSafetyLevel::Safe,
            assumptions: Vec::new(),
            suggested_alternatives: Vec::new(),
        }
    }

    /// Creates a SAFE-WITH-ASSUMPTIONS fix that requires documented assumptions.
    ///
    /// These fixes preserve semantics for 95%+ of cases but require explicit
    /// opt-in via `--fix --fix-assumptions`.
    ///
    /// # Arguments
    ///
    /// * `replacement` - The replacement text
    /// * `assumptions` - List of assumptions that must hold for correctness
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Fix;
    ///
    /// let fix = Fix::new_with_assumptions(
    ///     "mkdir -p /tmp/dir",
    ///     vec!["Directory does not require special permissions".to_string()]
    /// );
    /// assert!(fix.is_safe_with_assumptions());
    /// ```
    pub fn new_with_assumptions(replacement: impl Into<String>, assumptions: Vec<String>) -> Self {
        Self {
            replacement: replacement.into(),
            safety_level: FixSafetyLevel::SafeWithAssumptions,
            assumptions,
            suggested_alternatives: Vec::new(),
        }
    }


    include!("diagnostic_part2_incl2.rs");
