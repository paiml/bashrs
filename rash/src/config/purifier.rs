//! Shell configuration file purifier
//!
//! Applies automatic fixes to shell configuration files.

use crate::config::{aliaser, deduplicator, quoter};

/// Purify a shell configuration file
pub fn purify_config(source: &str) -> String {
    let mut result = source.to_string();

    // Apply CONFIG-001: Deduplicate PATH entries
    result = deduplicator::deduplicate_path_entries(&result);

    // Apply CONFIG-002: Quote variable expansions
    result = quoter::quote_variables(&result);

    // Apply CONFIG-003: Consolidate duplicate aliases
    result = aliaser::consolidate_aliases(&result);

    // TODO: Apply more purification rules
    // - CONFIG-004: Remove non-deterministic constructs
    // - CONFIG-005: Lazy-load expensive operations

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purify_config_deduplicates_paths() {
        // ARRANGE
        let source = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH""#;

        let expected = r#"export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH""#;

        // ACT
        let result = purify_config(source);

        // ASSERT
        assert_eq!(result, expected);
    }

    #[test]
    fn test_purify_config_preserves_other_content() {
        // ARRANGE
        let source = r#"# My .bashrc
export EDITOR="vim"
export PATH="/usr/local/bin:$PATH"
alias ll='ls -la'
export PATH="/usr/local/bin:$PATH"
echo "Welcome!""#;

        let expected = r#"# My .bashrc
export EDITOR="vim"
export PATH="/usr/local/bin:$PATH"
alias ll='ls -la'
echo "Welcome!""#;

        // ACT
        let result = purify_config(source);

        // ASSERT
        assert_eq!(result, expected);
    }
}
