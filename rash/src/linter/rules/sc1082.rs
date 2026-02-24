// SC1082: This file has a UTF-8 BOM (Byte Order Mark).
//
// UTF-8 files do not need a BOM. The BOM character (U+FEFF, encoded as
// EF BB BF in UTF-8) at the start of a file can cause the shebang line
// to be misinterpreted, making the script unrunnable.
//
// Examples:
// Bad:
//   \xEF\xBB\xBF#!/bin/sh     (BOM before shebang)
//   echo "hello"
//
// Good:
//   #!/bin/sh                   (no BOM)
//   echo "hello"
//
// Fix: sed -i '1s/^\xEF\xBB\xBF//' script.sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    if source.starts_with('\u{feff}') {
        let diagnostic = Diagnostic::new(
            "SC1082",
            Severity::Warning,
            "This file has a UTF-8 BOM. Remove it with: sed -i '1s/^\\xEF\\xBB\\xBF//'",
            Span::new(1, 1, 1, 4),
        );
        result.add(diagnostic);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1082_detects_bom() {
        let script = "\u{feff}#!/bin/sh\necho hello";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1082");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("UTF-8 BOM"));
    }

    #[test]
    fn test_sc1082_no_bom() {
        let script = "#!/bin/sh\necho hello";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1082_bom_only_checked_at_start() {
        // BOM in the middle of file should not trigger
        let script = "#!/bin/sh\necho \"\u{feff}hello\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1082_empty_source() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1082_bom_before_content() {
        let script = "\u{feff}echo hello";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1082_bom_only() {
        let script = "\u{feff}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
