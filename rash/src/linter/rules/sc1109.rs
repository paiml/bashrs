// SC1109: Unquoted HTML entity found in script.
//
// HTML entities like &amp;, &lt;, &gt;, &quot; suggest the script was copied
// from a web page where the source was HTML-encoded. These entities are not
// valid shell syntax and will cause errors or unexpected behavior.
//
// Examples:
// Bad:
//   if [ "$a" -eq 1 ] &amp;&amp; [ "$b" -eq 2 ]; then
//   test "$x" &lt; "$y"
//   echo &quot;hello&quot;
//
// Good:
//   if [ "$a" -eq 1 ] && [ "$b" -eq 2 ]; then
//   test "$x" < "$y"
//   echo "hello"
//
// Fix: Replace HTML entities with the corresponding characters

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// HTML entities that should not appear in shell scripts
const HTML_ENTITIES: [&str; 4] = ["&amp;", "&lt;", "&gt;", "&quot;"];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comment lines
        if line.trim_start().starts_with('#') {
            continue;
        }

        if HTML_ENTITIES.iter().any(|entity| line.contains(entity)) {
            let diagnostic = Diagnostic::new(
                "SC1109",
                Severity::Error,
                "This is an unquoted HTML entity. Replace with the corresponding character",
                Span::new(line_num, 1, line_num, line.len() + 1),
            );
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1109_detects_amp() {
        let script = "test -f file &amp;&amp; echo ok";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1109");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("HTML entity"));
    }

    #[test]
    fn test_sc1109_detects_lt() {
        let script = "if [ \"$a\" &lt; \"$b\" ]; then echo less; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1109_detects_gt() {
        let script = "if [ \"$a\" &gt; \"$b\" ]; then echo more; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1109_detects_quot() {
        let script = "echo &quot;hello&quot;";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1109_no_html_entities() {
        let script =
            "#!/bin/sh\ntest -f file && echo ok\nif [ \"$a\" -lt \"$b\" ]; then echo less; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1109_skips_comments() {
        let script = "# This &amp; that in a comment";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1109_ampersand_alone_ok() {
        // Plain & is fine -- it's a shell background operator
        let script = "cmd &\ncmd1 && cmd2";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1109_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1109_multiple_entities_one_line() {
        // Only one diagnostic per line
        let script = "echo &quot;hello&quot; &amp;&amp; echo &lt;ok&gt;";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
