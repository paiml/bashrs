// REPL Explain Mode Module
//
// Task: REPL-005-002 - Explain Mode (Explain bash constructs interactively)
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 15+ scenarios
// - Integration tests: CLI explain mode with assert_cmd
// - Mutation score: ≥90%
// - Complexity: <10 per function

/// Explanation for a bash construct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    /// Title of the construct
    pub title: String,
    /// Brief description
    pub description: String,
    /// Detailed explanation
    pub details: String,
    /// Example usage
    pub example: Option<String>,
}

impl Explanation {
    /// Create a new explanation
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            details: details.into(),
            example: None,
        }
    }

    /// Add an example to the explanation
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }

    /// Format explanation for display
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("📖 {}\n", self.title));
        output.push_str(&format!("   {}\n\n", self.description));
        output.push_str(&format!("{}\n", self.details));

        if let Some(ref example) = self.example {
            output.push_str("\n");
            output.push_str(&format!("Example:\n{}\n", example));
        }

        output
    }
}

/// Explain a bash construct
///
/// This function analyzes bash code and returns an explanation
/// of the construct or syntax used.
///
/// # Examples
///
/// ```
/// use bashrs::repl::explain::explain_bash;
///
/// let explanation = explain_bash("${var:-default}");
/// assert!(explanation.is_some());
/// ```
pub fn explain_bash(input: &str) -> Option<Explanation> {
    let trimmed = input.trim();

    // Parameter expansion patterns
    if let Some(exp) = explain_parameter_expansion(trimmed) {
        return Some(exp);
    }

    // Control flow patterns
    if let Some(exp) = explain_control_flow(trimmed) {
        return Some(exp);
    }

    // Redirection patterns
    if let Some(exp) = explain_redirection(trimmed) {
        return Some(exp);
    }

    None
}

/// Explain parameter expansion constructs
fn explain_parameter_expansion(input: &str) -> Option<Explanation> {
    // ${var:-default} - Use default value
    if input.contains(":-") && input.starts_with("${") && input.ends_with("}") {
        return Some(
            Explanation::new(
                "Parameter Expansion: ${parameter:-word}",
                "Use Default Value",
                "If parameter is unset or null, expand to 'word'.\nThe original parameter remains unchanged."
            )
            .with_example("  $ var=\"\"\n  $ echo \"${var:-fallback}\"  # Outputs: fallback\n  $ echo \"$var\"               # Still empty")
        );
    }

    // ${var:=default} - Assign default value
    if input.contains(":=") && input.starts_with("${") && input.ends_with("}") {
        return Some(
            Explanation::new(
                "Parameter Expansion: ${parameter:=word}",
                "Assign Default Value",
                "If parameter is unset or null, assign 'word' to it.\nThen expand to the new value."
            )
            .with_example("  $ unset var\n  $ echo \"${var:=fallback}\"  # Outputs: fallback\n  $ echo \"$var\"               # Now set to fallback")
        );
    }

    // ${var:?error} - Display error if null/unset
    if input.contains(":?") && input.starts_with("${") && input.ends_with("}") {
        return Some(
            Explanation::new(
                "Parameter Expansion: ${parameter:?word}",
                "Display Error if Null/Unset",
                "If parameter is unset or null, print 'word' to stderr and exit.\nUseful for required parameters."
            )
            .with_example("  $ unset var\n  $ echo \"${var:?Variable not set}\"  # Exits with error")
        );
    }

    // ${var:+alternate} - Use alternate value
    if input.contains(":+") && input.starts_with("${") && input.ends_with("}") {
        return Some(
            Explanation::new(
                "Parameter Expansion: ${parameter:+word}",
                "Use Alternate Value",
                "If parameter is set and non-null, expand to 'word'.\nOtherwise expand to nothing."
            )
            .with_example("  $ var=\"set\"\n  $ echo \"${var:+present}\"  # Outputs: present\n  $ unset var\n  $ echo \"${var:+present}\"  # Outputs: (nothing)")
        );
    }

    // ${#var} - String length
    if input.starts_with("${#") && input.ends_with("}") {
        return Some(
            Explanation::new(
                "Parameter Expansion: ${#parameter}",
                "String Length",
                "Expands to the length of the parameter's value in characters."
            )
            .with_example("  $ var=\"hello\"\n  $ echo \"${#var}\"  # Outputs: 5")
        );
    }

    None
}

/// Explain control flow constructs
fn explain_control_flow(input: &str) -> Option<Explanation> {
    // for loop
    if input.starts_with("for ") {
        return Some(
            Explanation::new(
                "For Loop: for name in words",
                "Iterate Over List",
                "Loop variable 'name' takes each value from the word list.\nExecutes commands for each iteration."
            )
            .with_example("  for file in *.txt; do\n    echo \"Processing: $file\"\n  done")
        );
    }

    // if statement
    if input.starts_with("if ") {
        return Some(
            Explanation::new(
                "If Statement: if condition; then commands; fi",
                "Conditional Execution",
                "Execute commands only if condition succeeds (exit status 0).\nOptional elif and else clauses for alternatives."
            )
            .with_example("  if [ -f file.txt ]; then\n    echo \"File exists\"\n  fi")
        );
    }

    // while loop
    if input.starts_with("while ") {
        return Some(
            Explanation::new(
                "While Loop: while condition; do commands; done",
                "Conditional Loop",
                "Execute commands repeatedly while condition succeeds.\nChecks condition before each iteration."
            )
            .with_example("  counter=0\n  while [ $counter -lt 5 ]; do\n    echo $counter\n    counter=$((counter + 1))\n  done")
        );
    }

    // case statement
    if input.starts_with("case ") {
        return Some(
            Explanation::new(
                "Case Statement: case word in pattern) commands;; esac",
                "Pattern Matching",
                "Match 'word' against patterns and execute corresponding commands.\nSupports glob patterns and multiple alternatives."
            )
            .with_example("  case $var in\n    start) echo \"Starting...\";;\n    stop)  echo \"Stopping...\";;\n    *)     echo \"Unknown\";;\n  esac")
        );
    }

    None
}

/// Explain redirection constructs
fn explain_redirection(input: &str) -> Option<Explanation> {
    // Output redirection >
    if input.contains(" > ") || input.ends_with(">") {
        return Some(
            Explanation::new(
                "Output Redirection: command > file",
                "Redirect Standard Output",
                "Redirects stdout to a file, overwriting existing content.\nUse >> to append instead."
            )
            .with_example("  echo \"text\" > file.txt   # Overwrite\n  echo \"more\" >> file.txt  # Append")
        );
    }

    // Input redirection <
    if input.contains(" < ") {
        return Some(
            Explanation::new(
                "Input Redirection: command < file",
                "Redirect Standard Input",
                "Redirects stdin to read from a file instead of keyboard."
            )
            .with_example("  while read line; do\n    echo \"Line: $line\"\n  done < file.txt")
        );
    }

    // Pipe |
    if input.contains(" | ") {
        return Some(
            Explanation::new(
                "Pipe: command1 | command2",
                "Connect Commands",
                "Redirects stdout of command1 to stdin of command2.\nEnables chaining multiple commands together."
            )
            .with_example("  cat file.txt | grep pattern | wc -l")
        );
    }

    // Here document <<
    if input.contains("<<") {
        return Some(
            Explanation::new(
                "Here Document: command << DELIMITER",
                "Multi-line Input",
                "Redirects multiple lines of input to a command.\nEnds at line containing only DELIMITER."
            )
            .with_example("  cat << EOF\n  Line 1\n  Line 2\n  EOF")
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== RED PHASE: Unit Tests (These should FAIL initially) =====

    #[test]
    fn test_REPL_005_002_explain_parameter_expansion_use_default() {
        let result = explain_bash("${var:-default}");

        assert!(result.is_some(), "Should recognize parameter expansion");
        let explanation = result.unwrap();

        assert!(explanation.title.contains(":-"));
        assert!(explanation.description.contains("Default"));
        assert!(explanation.details.contains("unset or null"));
    }

    #[test]
    fn test_REPL_005_002_explain_parameter_expansion_assign_default() {
        let result = explain_bash("${var:=default}");

        assert!(result.is_some(), "Should recognize assign default");
        let explanation = result.unwrap();

        assert!(explanation.title.contains(":="));
        assert!(explanation.description.contains("Assign"));
    }

    #[test]
    fn test_REPL_005_002_explain_parameter_expansion_error() {
        let result = explain_bash("${var:?error message}");

        assert!(result.is_some(), "Should recognize error expansion");
        let explanation = result.unwrap();

        assert!(explanation.title.contains(":?"));
        assert!(explanation.description.contains("Error"));
    }

    #[test]
    fn test_REPL_005_002_explain_parameter_expansion_alternate() {
        let result = explain_bash("${var:+alternate}");

        assert!(result.is_some(), "Should recognize alternate expansion");
        let explanation = result.unwrap();

        assert!(explanation.title.contains(":+"));
        assert!(explanation.description.contains("Alternate"));
    }

    #[test]
    fn test_REPL_005_002_explain_string_length() {
        let result = explain_bash("${#var}");

        assert!(result.is_some(), "Should recognize string length");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("#"));
        assert!(explanation.description.contains("Length"));
    }

    #[test]
    fn test_REPL_005_002_explain_for_loop() {
        let result = explain_bash("for i in *.txt");

        assert!(result.is_some(), "Should recognize for loop");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("For"));
        assert!(explanation.description.contains("Iterate"));
    }

    #[test]
    fn test_REPL_005_002_explain_if_statement() {
        let result = explain_bash("if [ -f file ]");

        assert!(result.is_some(), "Should recognize if statement");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("If"));
        assert!(explanation.description.contains("Conditional"));
    }

    #[test]
    fn test_REPL_005_002_explain_while_loop() {
        let result = explain_bash("while true");

        assert!(result.is_some(), "Should recognize while loop");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("While"));
    }

    #[test]
    fn test_REPL_005_002_explain_case_statement() {
        let result = explain_bash("case $var in");

        assert!(result.is_some(), "Should recognize case statement");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("Case"));
        assert!(explanation.description.contains("Pattern"));
    }

    #[test]
    fn test_REPL_005_002_explain_output_redirection() {
        let result = explain_bash("echo test > file.txt");

        assert!(result.is_some(), "Should recognize output redirection");
        let explanation = result.unwrap();

        assert!(explanation.title.contains(">"));
        assert!(explanation.description.contains("Output"));
    }

    #[test]
    fn test_REPL_005_002_explain_input_redirection() {
        let result = explain_bash("cat < file.txt");

        assert!(result.is_some(), "Should recognize input redirection");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("<"));
        assert!(explanation.description.contains("Input"));
    }

    #[test]
    fn test_REPL_005_002_explain_pipe() {
        let result = explain_bash("cat file | grep pattern");

        assert!(result.is_some(), "Should recognize pipe");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("|") || explanation.title.contains("Pipe"));
        assert!(explanation.description.contains("Connect") || explanation.description.contains("Chain"));
    }

    #[test]
    fn test_REPL_005_002_explain_here_document() {
        let result = explain_bash("cat << EOF");

        assert!(result.is_some(), "Should recognize here document");
        let explanation = result.unwrap();

        assert!(explanation.title.contains("<<") || explanation.title.contains("Here"));
    }

    #[test]
    fn test_REPL_005_002_explain_unknown_returns_none() {
        let result = explain_bash("unknown_construct_xyz_123");

        assert!(result.is_none(), "Should return None for unknown constructs");
    }

    #[test]
    fn test_REPL_005_002_explanation_format() {
        let explanation = Explanation::new(
            "Test Construct",
            "Brief description",
            "Detailed explanation here"
        )
        .with_example("  $ example command");

        let formatted = explanation.format();

        assert!(formatted.contains("📖 Test Construct"));
        assert!(formatted.contains("Brief description"));
        assert!(formatted.contains("Detailed explanation"));
        assert!(formatted.contains("Example:"));
        assert!(formatted.contains("$ example command"));
    }
}
