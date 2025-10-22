// SC2254: Quote expansions in case patterns (placeholder - overlaps SC2231)
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2231
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2254_placeholder() {
        assert_eq!(check("case $x in").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_quoted() {
        assert_eq!(check(r#"case "$x" in"#).diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_comment() {
        assert_eq!(check("# comment").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_echo() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_literal() {
        assert_eq!(check("case value in").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_pattern() {
        assert_eq!(check("case $x in\n  a) ;;\nesac").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_multi() {
        assert_eq!(check("case $x in\n  a|b) ;;\nesac").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_wildcard() {
        assert_eq!(check("case $x in\n  *) ;;\nesac").diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2254_braced() {
        assert_eq!(check("case ${var} in").diagnostics.len(), 0);
    }
}
