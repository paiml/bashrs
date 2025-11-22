//! Shell compatibility rules for linter.
//!
//! Defines which shells each linter rule applies to, allowing bashrs to skip
//! rules that don't apply to a specific shell type.
//!
//! # Examples
//!
//! ## Checking if a rule applies to a shell
//!
//! ```
//! use bashrs::linter::{ShellCompatibility, ShellType};
//!
//! let compat = ShellCompatibility::BashOnly;
//! assert!(compat.applies_to(ShellType::Bash));
//! assert!(!compat.applies_to(ShellType::Sh));
//! ```
//!
//! ## Getting human-readable descriptions
//!
//! ```
//! use bashrs::linter::ShellCompatibility;
//!
//! let compat = ShellCompatibility::Universal;
//! assert_eq!(compat.description(), "all shells (bash, zsh, sh, ksh)");
//! ```
//!
//! ## Using with rule registry
//!
//! ```
//! use bashrs::linter::{get_rule_compatibility, ShellType};
//!
//! // Check if SEC001 applies to POSIX sh
//! if let Some(compat) = get_rule_compatibility("SEC001") {
//!     if compat.applies_to(ShellType::Sh) {
//!         // Apply SEC001 to this sh script
//!     }
//! }
//! ```

use crate::linter::shell_type::ShellType;

/// Shell compatibility level for linter rules.
///
/// Specifies which shell types a linter rule applies to. Used by the rule registry
/// to filter rules based on the detected shell type.
///
/// # Variant Descriptions
///
/// - **Universal**: Rule applies to all shells (bash, zsh, sh, ksh)
/// - **BashOnly**: Rule only applies to bash scripts
/// - **ZshOnly**: Rule only applies to zsh scripts
/// - **ShOnly**: Rule only applies to POSIX sh scripts (strict)
/// - **BashZsh**: Rule applies to bash and zsh (modern shells)
/// - **NotSh**: Rule applies to bash/zsh/ksh but not POSIX sh
///
/// # Examples
///
/// ## Checking shell compatibility
///
/// ```
/// use bashrs::linter::{ShellCompatibility, ShellType};
///
/// let compat = ShellCompatibility::BashZsh;
/// assert!(compat.applies_to(ShellType::Bash));
/// assert!(compat.applies_to(ShellType::Zsh));
/// assert!(!compat.applies_to(ShellType::Sh));
/// ```
///
/// ## Using universal compatibility
///
/// ```
/// use bashrs::linter::{ShellCompatibility, ShellType};
///
/// let compat = ShellCompatibility::Universal;
/// // Universal rules apply to ALL shells
/// assert!(compat.applies_to(ShellType::Bash));
/// assert!(compat.applies_to(ShellType::Zsh));
/// assert!(compat.applies_to(ShellType::Sh));
/// assert!(compat.applies_to(ShellType::Ksh));
/// ```
///
/// ## Shell-specific rules
///
/// ```
/// use bashrs::linter::{ShellCompatibility, ShellType};
///
/// // BashOnly rules only apply to bash
/// let bash_rule = ShellCompatibility::BashOnly;
/// assert!(bash_rule.applies_to(ShellType::Bash));
/// assert!(!bash_rule.applies_to(ShellType::Zsh));
///
/// // ShOnly rules only apply to POSIX sh
/// let sh_rule = ShellCompatibility::ShOnly;
/// assert!(sh_rule.applies_to(ShellType::Sh));
/// assert!(!sh_rule.applies_to(ShellType::Bash));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellCompatibility {
    /// Works in all shells (bash, zsh, sh, ksh)
    Universal,
    /// Bash-specific features only
    BashOnly,
    /// Zsh-specific features only
    ZshOnly,
    /// POSIX sh only (strict)
    ShOnly,
    /// Works in bash and zsh
    BashZsh,
    /// Works in bash/zsh/ksh but not sh
    NotSh,
}

impl ShellCompatibility {
    /// Checks if this compatibility level applies to the given shell type.
    ///
    /// Returns `true` if a rule with this compatibility should be applied to the specified shell.
    ///
    /// # Arguments
    ///
    /// * `shell` - The shell type to check compatibility for
    ///
    /// # Returns
    ///
    /// `true` if the rule applies to this shell, `false` otherwise
    ///
    /// # Special Cases
    ///
    /// - `ShellType::Auto` defaults to bash behavior for compatibility purposes
    /// - `Universal` applies to all shells including `Auto`
    ///
    /// # Examples
    ///
    /// ## Universal compatibility
    ///
    /// ```
    /// use bashrs::linter::{ShellCompatibility, ShellType};
    ///
    /// let compat = ShellCompatibility::Universal;
    /// assert!(compat.applies_to(ShellType::Bash));
    /// assert!(compat.applies_to(ShellType::Zsh));
    /// assert!(compat.applies_to(ShellType::Sh));
    /// assert!(compat.applies_to(ShellType::Auto));
    /// ```
    ///
    /// ## Bash-only rules
    ///
    /// ```
    /// use bashrs::linter::{ShellCompatibility, ShellType};
    ///
    /// let compat = ShellCompatibility::BashOnly;
    /// assert!(compat.applies_to(ShellType::Bash));
    /// assert!(!compat.applies_to(ShellType::Zsh));
    /// assert!(!compat.applies_to(ShellType::Sh));
    /// // Auto defaults to bash
    /// assert!(compat.applies_to(ShellType::Auto));
    /// ```
    ///
    /// ## POSIX sh-only rules
    ///
    /// ```
    /// use bashrs::linter::{ShellCompatibility, ShellType};
    ///
    /// let compat = ShellCompatibility::ShOnly;
    /// assert!(!compat.applies_to(ShellType::Bash));
    /// assert!(compat.applies_to(ShellType::Sh));
    /// assert!(!compat.applies_to(ShellType::Auto));
    /// ```
    ///
    /// ## NotSh compatibility (bash/zsh/ksh but not sh)
    ///
    /// ```
    /// use bashrs::linter::{ShellCompatibility, ShellType};
    ///
    /// let compat = ShellCompatibility::NotSh;
    /// assert!(compat.applies_to(ShellType::Bash));
    /// assert!(compat.applies_to(ShellType::Zsh));
    /// assert!(compat.applies_to(ShellType::Ksh));
    /// assert!(!compat.applies_to(ShellType::Sh));
    /// ```
    pub fn applies_to(&self, shell: ShellType) -> bool {
        match (self, shell) {
            (ShellCompatibility::Universal, _) => true,
            (ShellCompatibility::BashOnly, ShellType::Bash) => true,
            (ShellCompatibility::ZshOnly, ShellType::Zsh) => true,
            (ShellCompatibility::ShOnly, ShellType::Sh) => true,
            (ShellCompatibility::BashZsh, ShellType::Bash | ShellType::Zsh) => true,
            (ShellCompatibility::NotSh, ShellType::Bash | ShellType::Zsh | ShellType::Ksh) => true,
            (ShellCompatibility::NotSh, ShellType::Auto) => true, // Auto defaults to bash
            (ShellCompatibility::BashOnly, ShellType::Auto) => true, // Auto defaults to bash
            (ShellCompatibility::BashZsh, ShellType::Auto) => true, // Auto defaults to bash
            _ => false,
        }
    }

    /// Returns a human-readable description of the shell compatibility level.
    ///
    /// Provides a concise description of which shells this compatibility level applies to,
    /// useful for displaying in error messages, documentation, or user interfaces.
    ///
    /// # Returns
    ///
    /// A static string describing the shell compatibility level
    ///
    /// # Examples
    ///
    /// ## Display compatibility descriptions
    ///
    /// ```
    /// use bashrs::linter::ShellCompatibility;
    ///
    /// assert_eq!(
    ///     ShellCompatibility::Universal.description(),
    ///     "all shells (bash, zsh, sh, ksh)"
    /// );
    ///
    /// assert_eq!(
    ///     ShellCompatibility::BashOnly.description(),
    ///     "bash only"
    /// );
    ///
    /// assert_eq!(
    ///     ShellCompatibility::BashZsh.description(),
    ///     "bash and zsh"
    /// );
    /// ```
    ///
    /// ## Use in error messages
    ///
    /// ```
    /// use bashrs::linter::ShellCompatibility;
    ///
    /// let compat = ShellCompatibility::NotSh;
    /// let msg = format!(
    ///     "This rule applies to {}",
    ///     compat.description()
    /// );
    /// assert_eq!(msg, "This rule applies to bash, zsh, ksh (not POSIX sh)");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            ShellCompatibility::Universal => "all shells (bash, zsh, sh, ksh)",
            ShellCompatibility::BashOnly => "bash only",
            ShellCompatibility::ZshOnly => "zsh only",
            ShellCompatibility::ShOnly => "POSIX sh only",
            ShellCompatibility::BashZsh => "bash and zsh",
            ShellCompatibility::NotSh => "bash, zsh, ksh (not POSIX sh)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first

    #[test]
    fn test_universal_applies_to_all_shells() {
        let compat = ShellCompatibility::Universal;
        assert!(compat.applies_to(ShellType::Bash));
        assert!(compat.applies_to(ShellType::Zsh));
        assert!(compat.applies_to(ShellType::Sh));
        assert!(compat.applies_to(ShellType::Ksh));
        assert!(compat.applies_to(ShellType::Auto));
    }

    #[test]
    fn test_bash_only_applies_to_bash() {
        let compat = ShellCompatibility::BashOnly;
        assert!(compat.applies_to(ShellType::Bash));
        assert!(!compat.applies_to(ShellType::Zsh));
        assert!(!compat.applies_to(ShellType::Sh));
        assert!(!compat.applies_to(ShellType::Ksh));
        assert!(compat.applies_to(ShellType::Auto)); // Auto defaults to bash
    }

    #[test]
    fn test_zsh_only_applies_to_zsh() {
        let compat = ShellCompatibility::ZshOnly;
        assert!(!compat.applies_to(ShellType::Bash));
        assert!(compat.applies_to(ShellType::Zsh));
        assert!(!compat.applies_to(ShellType::Sh));
        assert!(!compat.applies_to(ShellType::Ksh));
        assert!(!compat.applies_to(ShellType::Auto));
    }

    #[test]
    fn test_sh_only_applies_to_sh() {
        let compat = ShellCompatibility::ShOnly;
        assert!(!compat.applies_to(ShellType::Bash));
        assert!(!compat.applies_to(ShellType::Zsh));
        assert!(compat.applies_to(ShellType::Sh));
        assert!(!compat.applies_to(ShellType::Ksh));
        assert!(!compat.applies_to(ShellType::Auto));
    }

    #[test]
    fn test_bash_zsh_applies_to_both() {
        let compat = ShellCompatibility::BashZsh;
        assert!(compat.applies_to(ShellType::Bash));
        assert!(compat.applies_to(ShellType::Zsh));
        assert!(!compat.applies_to(ShellType::Sh));
        assert!(!compat.applies_to(ShellType::Ksh));
        assert!(compat.applies_to(ShellType::Auto));
    }

    #[test]
    fn test_not_sh_applies_to_non_posix() {
        let compat = ShellCompatibility::NotSh;
        assert!(compat.applies_to(ShellType::Bash));
        assert!(compat.applies_to(ShellType::Zsh));
        assert!(!compat.applies_to(ShellType::Sh));
        assert!(compat.applies_to(ShellType::Ksh));
        assert!(compat.applies_to(ShellType::Auto));
    }

    #[test]
    fn test_description_strings() {
        assert_eq!(
            ShellCompatibility::Universal.description(),
            "all shells (bash, zsh, sh, ksh)"
        );
        assert_eq!(ShellCompatibility::BashOnly.description(), "bash only");
        assert_eq!(ShellCompatibility::ZshOnly.description(), "zsh only");
        assert_eq!(ShellCompatibility::ShOnly.description(), "POSIX sh only");
        assert_eq!(ShellCompatibility::BashZsh.description(), "bash and zsh");
        assert_eq!(
            ShellCompatibility::NotSh.description(),
            "bash, zsh, ksh (not POSIX sh)"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property: Universal always applies
    proptest! {
        #[test]
        fn prop_universal_always_applies(shell_idx in 0..5usize) {
            let shell = match shell_idx {
                0 => ShellType::Bash,
                1 => ShellType::Zsh,
                2 => ShellType::Sh,
                3 => ShellType::Ksh,
                _ => ShellType::Auto,
            };
            prop_assert!(ShellCompatibility::Universal.applies_to(shell));
        }
    }

    // Property: BashOnly never applies to non-bash (except Auto)
    proptest! {
        #[test]
        fn prop_bash_only_exclusive(shell_idx in 1..4usize) {
            let shell = match shell_idx {
                1 => ShellType::Zsh,
                2 => ShellType::Sh,
                _ => ShellType::Ksh,
            };
            prop_assert!(!ShellCompatibility::BashOnly.applies_to(shell));
        }
    }

    // Property: ZshOnly only applies to zsh
    proptest! {
        #[test]
        fn prop_zsh_only_exclusive(shell_idx in 0..5usize) {
            let shell = match shell_idx {
                0 => ShellType::Bash,
                1 => ShellType::Zsh,
                2 => ShellType::Sh,
                3 => ShellType::Ksh,
                _ => ShellType::Auto,
            };
            let applies = ShellCompatibility::ZshOnly.applies_to(shell);
            prop_assert_eq!(applies, shell == ShellType::Zsh);
        }
    }

    // Property: NotSh never applies to sh
    proptest! {
        #[test]
        fn prop_not_sh_excludes_sh(shell_idx in 0..5usize) {
            let shell = match shell_idx {
                0 => ShellType::Bash,
                1 => ShellType::Zsh,
                2 => ShellType::Sh,
                3 => ShellType::Ksh,
                _ => ShellType::Auto,
            };
            let applies = ShellCompatibility::NotSh.applies_to(shell);
            if shell == ShellType::Sh {
                prop_assert!(!applies);
            }
        }
    }
}
