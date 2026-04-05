//! Rule Registry - Central metadata for all linter rules.
//!
//! Provides a central registry of all linter rules with their metadata,
//! including shell compatibility information. Use this to query which rules
//! apply to specific shell types.
//!
//! # Examples
//!
//! ## Checking rule compatibility
//!
//! ```
//! use bashrs::linter::rule_registry;
//! use bashrs::linter::ShellType;
//!
//! // Check if a rule applies to bash
//! assert!(rule_registry::should_apply_rule("SEC001", ShellType::Bash));
//!
//! // Check if a rule applies to POSIX sh
//! assert!(rule_registry::should_apply_rule("IDEM001", ShellType::Sh));
//! ```
//!
//! ## Getting rule metadata
//!
//! ```
//! use bashrs::linter::rule_registry;
//!
//! if let Some(compat) = rule_registry::get_rule_compatibility("SEC001") {
//!     println!("SEC001 compatibility: {:?}", compat);
//! }
//! ```

use crate::linter::shell_compatibility::ShellCompatibility;
use std::collections::HashMap;

/// Metadata for a linter rule, including shell compatibility.
///
/// Each rule has a unique ID, descriptive name, and compatibility specification
/// indicating which shell types the rule applies to.
///
/// # Examples
///
/// ## Accessing metadata from registry
///
/// ```
/// use bashrs::linter::rule_registry;
///
/// // Get compatibility for a security rule
/// let compat = rule_registry::get_rule_compatibility("SEC001");
/// assert!(compat.is_some());
/// ```
///
/// # Fields
///
/// * `id` - Unique rule identifier (e.g., "SEC001", "DET001", "IDEM001")
/// * `name` - Human-readable rule description
/// * `compatibility` - Shell compatibility specification
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// Unique rule identifier (e.g., "SEC001", "DET001").
    pub id: &'static str,

    /// Human-readable description of the rule.
    pub name: &'static str,

    /// Shell compatibility specification.
    ///
    /// Determines which shell types this rule applies to:
    /// - `Universal`: Applies to all shells
    /// - `BashOnly`: Applies only to bash
    /// - `PosixOnly`: Applies only to POSIX sh
    /// - `BashAndZsh`: Applies to bash and zsh
    pub compatibility: ShellCompatibility,
}

/// Gets the shell compatibility for a specific rule ID.
///
/// Returns the compatibility specification if the rule exists in the registry.
///
/// # Arguments
///
/// * `rule_id` - The rule identifier (e.g., "SEC001", "DET001")
///
/// # Returns
///
/// * `Some(ShellCompatibility)` - If rule exists in registry
/// * `None` - If rule ID not found
///
/// # Examples
///
/// ## Check security rule compatibility
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellCompatibility;
///
/// let compat = rule_registry::get_rule_compatibility("SEC001");
/// assert_eq!(compat, Some(ShellCompatibility::Universal));
/// ```
///
/// ## Handle unknown rules
///
/// ```
/// use bashrs::linter::rule_registry;
///
/// let compat = rule_registry::get_rule_compatibility("UNKNOWN");
/// assert!(compat.is_none());
/// ```
pub fn get_rule_compatibility(rule_id: &str) -> Option<ShellCompatibility> {
    RULE_REGISTRY.get(rule_id).map(|meta| meta.compatibility)
}

/// Returns metadata for a specific rule by ID.
pub fn get_rule_metadata(rule_id: &str) -> Option<&RuleMetadata> {
    RULE_REGISTRY.get(rule_id)
}

/// Returns all rule metadata entries sorted by ID.
pub fn all_rules() -> Vec<&'static RuleMetadata> {
    let mut rules: Vec<&RuleMetadata> = RULE_REGISTRY.values().collect();
    rules.sort_by_key(|r| r.id);
    rules
}

/// Checks if a rule should be applied for a given shell type.
///
/// Queries the rule registry and checks if the rule's compatibility
/// specification matches the target shell type.
///
/// # Arguments
///
/// * `rule_id` - The rule identifier to check
/// * `shell` - The target shell type
///
/// # Returns
///
/// * `true` - If rule applies to the shell type (or rule not in registry)
/// * `false` - If rule explicitly doesn't apply to the shell type
///
/// # Conservative Default
///
/// If a rule is not found in the registry, this function returns `true`
/// (conservative approach - assume rule applies unless explicitly excluded).
///
/// # Examples
///
/// ## Security rules (universal)
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellType;
///
/// // Security rules apply to all shells
/// assert!(rule_registry::should_apply_rule("SEC001", ShellType::Bash));
/// assert!(rule_registry::should_apply_rule("SEC001", ShellType::Sh));
/// assert!(rule_registry::should_apply_rule("SEC001", ShellType::Zsh));
/// ```
///
/// ## Filtering by shell type
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellType;
///
/// // Only apply rules that match the target shell
/// let rules_to_check = vec!["SEC001", "DET001", "IDEM001"];
/// let bash_rules: Vec<_> = rules_to_check
///     .into_iter()
///     .filter(|&rule| rule_registry::should_apply_rule(rule, ShellType::Bash))
///     .collect();
///
/// assert_eq!(bash_rules.len(), 3); // All universal rules
/// ```
///
/// ## Unknown rules default to applying
///
/// ```
/// use bashrs::linter::rule_registry;
/// use bashrs::linter::ShellType;
///
/// // Unknown rules conservatively apply
/// assert!(rule_registry::should_apply_rule("UNKNOWN", ShellType::Bash));
/// ```
pub fn should_apply_rule(rule_id: &str, shell: crate::linter::shell_type::ShellType) -> bool {
    if let Some(compat) = get_rule_compatibility(rule_id) {
        compat.applies_to(shell)
    } else {
        // If rule not in registry, assume universal (conservative approach)
        true
    }
}

lazy_static::lazy_static! {
    static ref RULE_REGISTRY: HashMap<&'static str, RuleMetadata> = {
        let mut registry = HashMap::new();
        super::rule_registry_data_1::register(&mut registry);
        super::rule_registry_data_2::register(&mut registry);
        super::rule_registry_data_3::register(&mut registry);
        super::rule_registry_data_4::register(&mut registry);
        super::rule_registry_data_11::register(&mut registry);
        super::rule_registry_data_12::register(&mut registry);
        registry
    };
}

#[cfg(test)]
#[path = "rule_registry_tests.rs"]
mod tests;
