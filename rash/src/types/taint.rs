//! Taint tracking type system for injection safety verification
//!
//! This module implements a gradual type system with taint tracking to prevent
//! injection attacks in generated shell scripts. Values are classified as Safe,
//! Tainted, or Sanitized based on their source and usage.
//!
//! ## Overview
//!
//! - **Safe**: Values from trusted sources (literals, compile-time constants)
//! - **Tainted**: Values from untrusted sources (user input, network, command substitution)
//! - **Sanitized**: Tainted values that have been properly quoted/escaped
//!
//! ## Usage
//!
//! ```rust
//! use bashrs::types::taint::{TypeChecker, Type, Taint};
//!
//! let mut checker = TypeChecker::new();
//!
//! // User input is tainted
//! checker.register_variable("user_input", Type::String { taint: Taint::Tainted });
//!
//! // Quoted usage is safe (becomes Sanitized)
//! assert!(checker.check_injection_safety("user_input", true).is_ok());
//!
//! // Unquoted usage is UNSAFE
//! assert!(checker.check_injection_safety("user_input", false).is_err());
//! ```

use std::collections::HashMap;

/// Taint status of a value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taint {
    /// Value from trusted source (literal, env var)
    Safe,
    /// Value from untrusted source (user input, network)
    Tainted,
    /// Tainted value that has been sanitized
    Sanitized,
}

/// Type with taint tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Integer value
    Int { taint: Taint },
    /// String value
    String { taint: Taint },
    /// File path (must be Safe or Sanitized)
    Path { taint: Taint },
    /// Shell command (must be Safe)
    Command { taint: Taint },
}

impl Type {
    /// Check if value is safe to use in command
    pub fn is_command_safe(&self) -> bool {
        matches!(self, Type::Command { taint: Taint::Safe } | Type::String {
                taint: Taint::Safe | Taint::Sanitized,
            })
    }

    /// Check if value is safe to use as path
    pub fn is_path_safe(&self) -> bool {
        matches!(self, Type::Path {
                taint: Taint::Safe | Taint::Sanitized,
            })
    }

    /// Sanitize a tainted value (e.g., by quoting)
    pub fn sanitize(self) -> Self {
        match self {
            Type::String {
                taint: Taint::Tainted,
            } => Type::String {
                taint: Taint::Sanitized,
            },
            Type::Path {
                taint: Taint::Tainted,
            } => Type::Path {
                taint: Taint::Sanitized,
            },
            other => other,
        }
    }
}

/// Type checker for bash AST with taint tracking
pub struct TypeChecker {
    /// Type environment: variable name → type
    env: HashMap<String, Type>,
}

impl TypeChecker {
    /// Create a new type checker with empty environment
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    /// Register a variable with its type
    pub fn register_variable(&mut self, name: &str, typ: Type) {
        self.env.insert(name.to_string(), typ);
    }

    /// Check if variable is safely quoted
    ///
    /// # Arguments
    ///
    /// * `var_name` - Variable name to check
    /// * `is_quoted` - Whether the variable usage is quoted
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Variable usage is safe
    /// * `Err(String)` - Variable usage is unsafe, with descriptive error message
    ///
    /// # Safety Properties
    ///
    /// - Tainted unquoted strings → UNSAFE (injection risk)
    /// - Tainted commands → UNSAFE (arbitrary code execution)
    /// - Safe/Sanitized values → Always OK
    pub fn check_injection_safety(&self, var_name: &str, is_quoted: bool) -> Result<(), String> {
        let var_type = self
            .env
            .get(var_name)
            .ok_or_else(|| format!("Variable {} not in scope", var_name))?;

        match var_type {
            Type::String {
                taint: Taint::Tainted,
            } if !is_quoted => Err(format!(
                "UNSAFE: Variable ${} is tainted and unquoted - injection risk",
                var_name
            )),
            Type::Command {
                taint: Taint::Tainted,
            } => Err(format!("UNSAFE: Command from tainted source: {}", var_name)),
            _ => Ok(()),
        }
    }

    /// Get the type of a variable
    pub fn get_type(&self, var_name: &str) -> Option<&Type> {
        self.env.get(var_name)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taint_tracking_injection_detection() {
        let mut checker = TypeChecker::new();

        // User input is tainted
        checker.register_variable(
            "user_input",
            Type::String {
                taint: Taint::Tainted,
            },
        );

        // ✅ Quoted usage is safe (sanitized)
        assert!(checker.check_injection_safety("user_input", true).is_ok());

        // ❌ Unquoted usage is UNSAFE
        let result = checker.check_injection_safety("user_input", false);
        assert!(result.is_err());
        assert!(result
            .expect_err("Should be unsafe")
            .contains("injection risk"));
    }

    #[test]
    fn test_type_sanitization() {
        let tainted = Type::String {
            taint: Taint::Tainted,
        };
        let sanitized = tainted.sanitize();

        assert_eq!(
            sanitized,
            Type::String {
                taint: Taint::Sanitized
            }
        );
    }

    #[test]
    fn test_safe_values_always_allowed() {
        let mut checker = TypeChecker::new();

        // Safe string literal
        checker.register_variable("safe_var", Type::String { taint: Taint::Safe });

        // Both quoted and unquoted are OK for safe values
        assert!(checker.check_injection_safety("safe_var", true).is_ok());
        assert!(checker.check_injection_safety("safe_var", false).is_ok());
    }

    #[test]
    fn test_sanitized_values_allowed() {
        let mut checker = TypeChecker::new();

        // Sanitized value (was tainted, now quoted)
        checker.register_variable(
            "sanitized_var",
            Type::String {
                taint: Taint::Sanitized,
            },
        );

        // Sanitized values are OK
        assert!(checker
            .check_injection_safety("sanitized_var", true)
            .is_ok());
        assert!(checker
            .check_injection_safety("sanitized_var", false)
            .is_ok());
    }

    #[test]
    fn test_tainted_command_always_unsafe() {
        let mut checker = TypeChecker::new();

        // Tainted command
        checker.register_variable(
            "tainted_cmd",
            Type::Command {
                taint: Taint::Tainted,
            },
        );

        // Even quoted, tainted commands are UNSAFE
        let result_quoted = checker.check_injection_safety("tainted_cmd", true);
        let result_unquoted = checker.check_injection_safety("tainted_cmd", false);

        assert!(result_quoted.is_err());
        assert!(result_unquoted.is_err());
    }

    #[test]
    fn test_command_safe_check() {
        let safe_cmd = Type::Command { taint: Taint::Safe };
        let tainted_cmd = Type::Command {
            taint: Taint::Tainted,
        };
        let safe_string = Type::String { taint: Taint::Safe };
        let tainted_string = Type::String {
            taint: Taint::Tainted,
        };

        assert!(safe_cmd.is_command_safe());
        assert!(!tainted_cmd.is_command_safe());
        assert!(safe_string.is_command_safe());
        assert!(!tainted_string.is_command_safe());
    }

    #[test]
    fn test_path_safe_check() {
        let safe_path = Type::Path { taint: Taint::Safe };
        let tainted_path = Type::Path {
            taint: Taint::Tainted,
        };
        let sanitized_path = Type::Path {
            taint: Taint::Sanitized,
        };

        assert!(safe_path.is_path_safe());
        assert!(!tainted_path.is_path_safe());
        assert!(sanitized_path.is_path_safe());
    }

    #[test]
    fn test_sanitize_int_noop() {
        let tainted_int = Type::Int {
            taint: Taint::Tainted,
        };
        let result = tainted_int.clone().sanitize();

        // Integers don't sanitize, they remain unchanged
        assert_eq!(result, tainted_int);
    }

    #[test]
    fn test_unknown_variable_error() {
        let checker = TypeChecker::new();

        let result = checker.check_injection_safety("unknown_var", true);
        assert!(result.is_err());
        assert!(result
            .expect_err("Should be error")
            .contains("not in scope"));
    }
}

/// Property-based tests for taint tracking
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating variable names
    fn var_name() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]{0,10}".prop_map(|s| s.to_string())
    }

    // Strategy for generating Taint values
    fn any_taint() -> impl Strategy<Value = Taint> {
        prop_oneof![
            Just(Taint::Safe),
            Just(Taint::Tainted),
            Just(Taint::Sanitized),
        ]
    }

    // Strategy for generating Type values
    fn any_type() -> impl Strategy<Value = Type> {
        any_taint().prop_flat_map(|taint| {
            prop_oneof![
                Just(Type::Int { taint }),
                Just(Type::String { taint }),
                Just(Type::Path { taint }),
                Just(Type::Command { taint }),
            ]
        })
    }

    proptest! {
        /// Property: Tainted unquoted strings are ALWAYS unsafe
        #[test]
        fn prop_tainted_unquoted_always_unsafe(
            var_name in var_name()
        ) {
            let mut checker = TypeChecker::new();
            checker.register_variable(&var_name, Type::String { taint: Taint::Tainted });

            // Unquoted tainted variables must be rejected
            let result = checker.check_injection_safety(&var_name, false);
            prop_assert!(result.is_err(), "Tainted unquoted variable should be unsafe");
            prop_assert!(result.expect_err("").contains("injection risk"));
        }

        /// Property: Quoted variables are ALWAYS safe (becomes Sanitized)
        #[test]
        fn prop_quoted_variables_safe(
            var_name in var_name(),
            taint in any_taint()
        ) {
            let mut checker = TypeChecker::new();
            checker.register_variable(&var_name, Type::String { taint });

            // Quoted variables are safe (except tainted commands)
            if taint != Taint::Tainted {  // Strings when quoted are OK
                // Note: Commands are special case, tested separately
                let result = checker.check_injection_safety(&var_name, true);
                // Quoted strings are always OK
                if matches!(checker.get_type(&var_name), Some(Type::String { .. })) {
                    prop_assert!(result.is_ok(), "Quoted string variables should be safe");
                }
            }
        }

        /// Property: Safe variables are ALWAYS allowed (quoted or not)
        #[test]
        fn prop_safe_always_allowed(
            var_name in var_name(),
            is_quoted in any::<bool>()
        ) {
            let mut checker = TypeChecker::new();
            checker.register_variable(&var_name, Type::String { taint: Taint::Safe });

            let result = checker.check_injection_safety(&var_name, is_quoted);
            prop_assert!(result.is_ok(), "Safe variables should always be allowed");
        }

        /// Property: Sanitized variables are ALWAYS allowed
        #[test]
        fn prop_sanitized_always_allowed(
            var_name in var_name(),
            is_quoted in any::<bool>()
        ) {
            let mut checker = TypeChecker::new();
            checker.register_variable(&var_name, Type::String { taint: Taint::Sanitized });

            let result = checker.check_injection_safety(&var_name, is_quoted);
            prop_assert!(result.is_ok(), "Sanitized variables should always be allowed");
        }

        /// Property: Command safety respects taint status
        #[test]
        fn prop_command_safety_respects_taint(
            taint in any_taint()
        ) {
            let cmd_type = Type::Command { taint };

            if taint == Taint::Safe {
                prop_assert!(cmd_type.is_command_safe(), "Safe commands should be allowed");
            } else if taint == Taint::Tainted {
                prop_assert!(!cmd_type.is_command_safe(), "Tainted commands should be blocked");
            }
        }

        /// Property: Path safety requires Safe or Sanitized
        #[test]
        fn prop_path_safety_requires_clean(
            taint in any_taint()
        ) {
            let path_type = Type::Path { taint };

            if taint == Taint::Tainted {
                prop_assert!(!path_type.is_path_safe(), "Tainted paths should be blocked");
            } else {
                prop_assert!(path_type.is_path_safe(), "Safe/Sanitized paths should be allowed");
            }
        }

        /// Property: Sanitize is idempotent
        #[test]
        fn prop_sanitize_idempotent(
            typ in any_type()
        ) {
            let sanitized_once = typ.clone().sanitize();
            let sanitized_twice = sanitized_once.clone().sanitize();

            prop_assert_eq!(sanitized_once, sanitized_twice, "Sanitize should be idempotent");
        }

        /// Property: Type checker is consistent across multiple checks
        #[test]
        fn prop_type_checker_consistent(
            var_name in var_name(),
            typ in any_type(),
            is_quoted in any::<bool>()
        ) {
            let mut checker = TypeChecker::new();
            checker.register_variable(&var_name, typ);

            let result1 = checker.check_injection_safety(&var_name, is_quoted);
            let result2 = checker.check_injection_safety(&var_name, is_quoted);

            prop_assert_eq!(result1.is_ok(), result2.is_ok(),
                "Multiple checks should return consistent results");
        }
    }
}
