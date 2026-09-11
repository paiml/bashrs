// SC2041: Use while read, not read in for loop, to read lines from files.
//
// Using `read` inside a for loop doesn't work as expected. The `read` command
// reads ONE line from stdin each iteration, not from the loop's data source.
//
// Examples:
// Bad:
//   for line in $(cat file.txt); do
//     read -r data  // Reads from stdin, not from cat output!
//   done
//
//   for i in 1 2 3; do
//     read var      // Reads from stdin, unrelated to loop
//   done
//
// Good:
//   while IFS= read -r line; do
//     echo "$line"  // Correctly reads each line from file
//   done < file.txt
//
//   while read -r var; do
//     process "$var"
//   done < data.txt
//
// Note: This is a common mistake. `read` always reads from stdin (fd 0),
// not from the for loop's iteration variable.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static READ_IN_FOR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: for ... do ... read ...
    // We'll use a simpler heuristic: look for `read` command inside for loops
    Regex::new(r"\bread\s+").unwrap()
});

/// Check if line starts a for loop
fn is_for_loop_start(line: &str) -> bool {
    line.contains("for ") && line.contains(" in ")
}

/// Check if line is a single-line for loop (for x in y; do cmd; done)
fn is_single_line_for_loop(line: &str) -> bool {
    line.contains("; do ") && line.contains("done")
}

/// Check if position in line is inside quotes
fn is_inside_quotes(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let quote_count = before.matches('"').count() + before.matches('\'').count();
    quote_count % 2 == 1
}

/// Check if read command is part of while read (which is correct)
fn is_while_read(line: &str, read_pos: usize) -> bool {
    line.contains("while") && line.find("while").unwrap_or(usize::MAX) < read_pos
}

/// GH-310: Check if this `read` carries its own input redirection
/// (`read x < file` or `read x <<< "$s"`). When it does, `read` takes its
/// input from the redirection, not from the loop's stdin, so SC2041 must
/// not fire.
fn read_has_own_redirect(line: &str, read_pos: usize) -> bool {
    line[read_pos..].contains('<')
}

/// Check if read is between do and done in single-line for loop
fn is_read_in_single_line_loop(line: &str) -> Option<usize> {
    if !line.contains("read ") {
        return None;
    }

    let read_pos = line.find("read ")?;
    let do_pos = line.find("; do ").map(|p| p + 5)?;
    let done_pos = line.find("done")?;

    if read_pos >= do_pos && read_pos < done_pos {
        Some(read_pos)
    } else {
        None
    }
}

/// Create diagnostic for read in for loop
fn create_read_in_for_diagnostic(
    line_num: usize,
    read_pos: usize,
    read_len: usize,
    for_loop_start_line: usize,
) -> Diagnostic {
    let start_col = read_pos + 1;
    let end_col = start_col + read_len;

    Diagnostic::new(
        "SC2041",
        Severity::Warning,
        format!(
            "'read' in for loop reads from stdin, not loop data. Use 'while read' instead (for loop started at line {})",
            for_loop_start_line
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

/// Handle a single-line `for ... ; do ... done` loop: return a diagnostic if
/// it contains a bare `read` with no redirection of its own.
fn check_single_line_for(
    line: &str,
    line_num: usize,
    for_loop_start_line: usize,
) -> Option<Diagnostic> {
    let read_pos = is_read_in_single_line_loop(line)?;
    if read_has_own_redirect(line, read_pos) {
        return None;
    }
    Some(create_read_in_for_diagnostic(
        line_num,
        read_pos,
        5, // "read "
        for_loop_start_line,
    ))
}

/// Handle a `read` occurrence on a line inside a multi-line for loop body.
fn check_read_in_for_body(
    line: &str,
    line_num: usize,
    for_loop_start_line: usize,
) -> Option<Diagnostic> {
    let mat = READ_IN_FOR.find(line)?;
    let pos = mat.start();

    // Skip if inside quotes, part of `while read`, or the `read` carries its
    // own input redirection (GH-310).
    if is_inside_quotes(line, pos) || is_while_read(line, pos) || read_has_own_redirect(line, pos) {
        return None;
    }

    Some(create_read_in_for_diagnostic(
        line_num,
        pos,
        mat.as_str().len(),
        for_loop_start_line,
    ))
}

/// Add `diag` to `result` if it is `Some`. Keeps `check`'s control flow flat
/// (no nested `if let` blocks) so its cognitive complexity stays readable.
fn push_if_some(result: &mut LintResult, diag: Option<Diagnostic>) {
    if let Some(diag) = diag {
        result.add(diag);
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut in_for_loop = false;
    let mut for_loop_start_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Track for loop context
        if is_for_loop_start(line) {
            in_for_loop = true;
            for_loop_start_line = line_num;

            // Handle single-line for loops
            if is_single_line_for_loop(line) {
                push_if_some(
                    &mut result,
                    check_single_line_for(line, line_num, for_loop_start_line),
                );
                in_for_loop = false;
                continue;
            }
        }

        // Reset context on `done`
        if in_for_loop && line.contains("done") {
            in_for_loop = false;
        }

        // Check for `read` inside for loop
        if in_for_loop && line.contains("read ") {
            push_if_some(
                &mut result,
                check_read_in_for_body(line, line_num, for_loop_start_line),
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2041_read_in_for() {
        let code = r#"
for line in $(cat file.txt); do
  read -r data
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2041");
        assert!(result.diagnostics[0].message.contains("while read"));
    }

    #[test]
    fn test_sc2041_read_simple_for() {
        let code = r#"
for i in 1 2 3; do
  read var
  echo "$var"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2041_while_read_ok() {
        let code = r#"
while IFS= read -r line; do
  echo "$line"
done < file.txt
"#;
        let result = check(code);
        // while read is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_read_outside_for_ok() {
        let code = r#"
read -r input
echo "$input"
"#;
        let result = check(code);
        // read outside for loop is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_in_quotes_ok() {
        let code = r#"
for i in 1 2 3; do
  echo "read -r var"
done
"#;
        let result = check(code);
        // Inside quotes, not actual command
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_comment_ok() {
        let code = r#"
for i in 1 2 3; do
  # read -r var
  echo "$i"
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_nested_while_read_ok() {
        let code = r#"
for dir in /var/log/*; do
  while read -r line; do
    echo "$line"
  done < "$dir/access.log"
done
"#;
        let result = check(code);
        // while read inside for is OK (different pattern)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2041_single_line_for() {
        let code = r#"for x in a b c; do read y; echo "$y"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2041_multiple_reads() {
        let code = r#"
for item in list; do
  read first
  read second
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2041_read_with_options() {
        let code = r#"
for i in 1 2 3; do
  read -r -p "Enter: " value
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    /// GH-310: `read x < file` inside a for loop takes its input from the
    /// redirection, not from the loop's stdin. SC2041 must not fire.
    #[test]
    fn test_PMAT257_gh310_sc2041_read_with_its_own_redirection_is_not_stdin() {
        let redirect_from_file = r#"
for i in 1 2 3; do
  read -r data < input.txt
done
"#;
        let result = check(redirect_from_file);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2041 must NOT flag 'read' with its own input redirection: {:?}",
            result.diagnostics
        );

        let here_string = r#"
for i in 1 2 3; do
  read -r data <<< "$s"
done
"#;
        let result = check(here_string);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2041 must NOT flag 'read' with its own here-string redirection: {:?}",
            result.diagnostics
        );
    }

    /// Companion: a bare `read` (no redirection of its own) inside a for
    /// loop over a pipeline still reads from stdin, not the loop's data.
    #[test]
    fn test_PMAT257_gh310_sc2041_bare_read_in_loop_over_pipeline_still_fires() {
        let code = r#"
for line in $(cat file.txt | grep foo); do
  read -r data
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2041");
    }
}
