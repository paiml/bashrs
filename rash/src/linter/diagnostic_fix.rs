
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

    /// Creates an UNSAFE fix that provides suggestions but no automatic replacement.
    ///
    /// These fixes require human judgment because they change control/data flow
    /// or require understanding developer intent. They are NEVER auto-applied.
    ///
    /// # Arguments
    ///
    /// * `suggested_alternatives` - List of suggested fixes for humans to choose from
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Fix;
    ///
    /// let fix = Fix::new_unsafe(vec![
    ///     "Option 1: Add error handling with || true".to_string(),
    ///     "Option 2: Use set -e for fail-fast".to_string(),
    /// ]);
    /// assert!(fix.is_unsafe());
    /// assert!(fix.replacement.is_empty());
    /// ```
    pub fn new_unsafe(suggested_alternatives: Vec<String>) -> Self {
        Self {
            replacement: String::new(), // No automatic replacement
            safety_level: FixSafetyLevel::Unsafe,
            assumptions: Vec::new(),
            suggested_alternatives,
        }
    }

    /// Checks if this fix can be applied with `--fix` (Safe level only).
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Fix;
    ///
    /// let safe_fix = Fix::new("\"$var\"");
    /// assert!(safe_fix.is_safe());
    ///
    /// let unsafe_fix = Fix::new_unsafe(vec!["Option 1".to_string()]);
    /// assert!(!unsafe_fix.is_safe());
    /// ```
    pub fn is_safe(&self) -> bool {
        self.safety_level == FixSafetyLevel::Safe
    }

    /// Checks if this fix can be applied with `--fix --fix-assumptions`.
    ///
    /// Returns `true` for both Safe and SafeWithAssumptions levels.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Fix;
    ///
    /// let safe = Fix::new("\"$var\"");
    /// assert!(safe.is_safe_with_assumptions());
    ///
    /// let with_assumptions = Fix::new_with_assumptions(
    ///     "mkdir -p /tmp/dir",
    ///     vec!["No special permissions".to_string()]
    /// );
    /// assert!(with_assumptions.is_safe_with_assumptions());
    ///
    /// let unsafe_fix = Fix::new_unsafe(vec!["Option".to_string()]);
    /// assert!(!unsafe_fix.is_safe_with_assumptions());
    /// ```
    pub fn is_safe_with_assumptions(&self) -> bool {
        matches!(
            self.safety_level,
            FixSafetyLevel::Safe | FixSafetyLevel::SafeWithAssumptions
        )
    }

    /// Checks if this fix should never be auto-applied (Unsafe level).
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::Fix;
    ///
    /// let unsafe_fix = Fix::new_unsafe(vec!["Option 1".to_string()]);
    /// assert!(unsafe_fix.is_unsafe());
    ///
    /// let safe_fix = Fix::new("\"$var\"");
    /// assert!(!safe_fix.is_unsafe());
    /// ```
    pub fn is_unsafe(&self) -> bool {
        self.safety_level == FixSafetyLevel::Unsafe
    }
}

/// A lint diagnostic with code, message, location, and optional fix.
///
/// Diagnostics represent linting findings (errors, warnings, suggestions) with
/// all the context needed for actionable feedback:
/// - **Code**: ShellCheck-compatible identifier (e.g., "SC2086", "DET001")
/// - **Severity**: Level of concern (Info, Warning, Error, etc.)
/// - **Message**: Human-readable explanation
/// - **Span**: Exact source location
/// - **Fix**: Optional automated repair (with safety guarantees)
///
/// # Examples
///
/// ## Creating a diagnostic without a fix
///
/// ```
/// use bashrs::linter::{Diagnostic, Severity, Span};
///
/// let span = Span::new(1, 5, 1, 10);
/// let diag = Diagnostic::new(
///     "SC2086",
///     Severity::Warning,
///     "Double quote to prevent globbing and word splitting",
///     span,
/// );
/// assert_eq!(diag.code, "SC2086");
/// assert!(diag.fix.is_none());
/// ```
///
/// ## Creating a diagnostic with a fix
///
/// ```
/// use bashrs::linter::{Diagnostic, Fix, Severity, Span};
///
/// let span = Span::new(1, 5, 1, 10);
/// let fix = Fix::new("\"$var\"");
/// let diag = Diagnostic::new("SC2086", Severity::Warning, "Quote variable", span)
///     .with_fix(fix);
/// assert!(diag.fix.is_some());
/// ```
///
/// ## Displaying a diagnostic
///
/// ```
/// use bashrs::linter::{Diagnostic, Severity, Span};
///
/// let span = Span::new(1, 5, 1, 10);
/// let diag = Diagnostic::new("SC2086", Severity::Warning, "Quote variable", span);
/// let display = format!("{}", diag);
/// assert!(display.contains("1:5-10"));
/// assert!(display.contains("warning"));
/// assert!(display.contains("SC2086"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Diagnostic code (e.g., "SC2086", "DET001", "IDEM002").
    pub code: String,

    /// Severity level.
    pub severity: Severity,

    /// Human-readable message explaining the issue.
    pub message: String,

    /// Source location (line and column span).
    pub span: Span,

    /// Optional suggested fix with safety guarantees.
    pub fix: Option<Fix>,
}

impl Diagnostic {
    /// Creates a new diagnostic without a fix.
    ///
    /// # Arguments
    ///
    /// * `code` - Diagnostic code (e.g., "SC2086", "DET001")
    /// * `severity` - Severity level
    /// * `message` - Human-readable explanation
    /// * `span` - Source location
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, Severity, Span};
    ///
    /// let span = Span::new(1, 5, 1, 10);
    /// let diag = Diagnostic::new(
    ///     "SC2086",
    ///     Severity::Warning,
    ///     "Double quote to prevent globbing",
    ///     span,
    /// );
    /// assert_eq!(diag.code, "SC2086");
    /// ```
    pub fn new(
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            span,
            fix: None,
        }
    }

    /// Adds a suggested fix to this diagnostic (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `fix` - The suggested fix
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::{Diagnostic, Fix, Severity, Span};
    ///
    /// let span = Span::new(1, 5, 1, 10);
    /// let fix = Fix::new("\"$var\"");
    /// let diag = Diagnostic::new("SC2086", Severity::Warning, "Quote", span)
    ///     .with_fix(fix);
    /// assert!(diag.fix.is_some());
    /// ```
    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] {}: {}",
            self.span, self.severity, self.code, self.message
        )
    }
}
include!("diagnostic_lintresult.rs");
