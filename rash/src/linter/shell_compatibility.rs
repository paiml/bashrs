// Shell Compatibility Module
// Defines which shells each linter rule applies to

use crate::linter::shell_type::ShellType;

/// Shell compatibility level for linter rules
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
    /// Check if this rule applies to the given shell type
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

    /// Get human-readable description
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
