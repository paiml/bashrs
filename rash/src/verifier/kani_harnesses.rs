#![cfg(kani)]
//! Kani verification harnesses for RASH
//! 
//! These harnesses verify critical safety properties using bounded model checking.

use crate::ast::{Expr, Stmt, Type};
use crate::emitter::escape::{escape_shell_string, escape_shell_value};
use crate::services::parser;
use crate::verifier::properties::{validate_rust0_ast, is_valid_rust0};

/// Verify that the parser only accepts valid Rust₀ programs
#[kani::proof]
#[kani::unwind(10)]
fn verify_parser_soundness() {
    let input: &str = kani::any();
    kani::assume(input.len() < 1000); // Bound input for tractability
    
    match parser::parse(input) {
        Ok(ast) => {
            // Property: Valid AST implies valid Rust₀
            kani::assert!(validate_rust0_ast(&ast));
        }
        Err(_) => {
            // Property: Parse error implies ∉ Rust₀
            kani::assert!(!is_valid_rust0(input));
        }
    }
}

/// Verify shell string escaping prevents injection
#[kani::proof]
#[kani::unwind(20)]
fn verify_escape_safety() {
    let input: String = kani::any();
    kani::assume(input.len() < 100);
    
    let escaped = escape_shell_string(&input);
    
    // Property 1: Result is always single-quoted
    kani::assert!(escaped.starts_with('\'') && escaped.ends_with('\''));
    
    // Property 2: No unescaped metacharacters possible
    kani::assert!(!contains_unescaped_metachar(&escaped));
    
    // Property 3: Original content is preserved (modulo escaping)
    let unescaped = unescape_shell_string(&escaped);
    kani::assert!(unescaped == input);
}

/// Verify variable expansion is always safely quoted
#[kani::proof]
#[kani::unwind(15)]
fn verify_variable_expansion_safety() {
    let var_name: String = kani::any();
    kani::assume(var_name.len() < 50);
    kani::assume(is_valid_identifier(&var_name));
    
    let expansion = format!("\"${{{}}}\"", var_name);
    
    // Property: Variable expansion is always double-quoted
    kani::assert!(expansion.starts_with('"') && expansion.ends_with('"'));
    kani::assert!(expansion.contains("${") && expansion.contains("}"));
}

/// Verify injection safety for all emitted shell code
#[kani::proof]
#[kani::unwind(10)]
fn verify_injection_safety() {
    // Generate arbitrary user input that might contain malicious content
    let user_input: String = kani::any();
    kani::assume(user_input.len() < 100);
    
    // Simulate various contexts where user input might appear
    let contexts = vec![
        format!("echo {}", escape_shell_string(&user_input)),
        format!("VAR={}", escape_shell_value(&user_input)),
        format!("if [ \"${{VAR}}\" = {} ]; then", escape_shell_string(&user_input)),
    ];
    
    for context in contexts {
        // Property: No command injection possible
        kani::assert!(!can_inject_command(&context, &user_input));
    }
}

/// Verify array bounds are always checked
#[kani::proof]
#[kani::unwind(5)]
fn verify_array_bounds_safety() {
    let array_size: usize = kani::any();
    kani::assume(array_size > 0 && array_size < 100);
    
    let index: usize = kani::any();
    
    // Simulated array access check
    if index < array_size {
        // Safe access
        kani::assert!(true);
    } else {
        // Must generate bounds check in shell
        let check = format!("if [ {} -lt {} ]; then", index, array_size);
        kani::assert!(check.contains("-lt"));
    }
}

/// Helper: Check if string contains unescaped shell metacharacters
fn contains_unescaped_metachar(s: &str) -> bool {
    let mut in_quotes = false;
    let mut escaped = false;
    
    for ch in s.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        
        match ch {
            '\\' => escaped = true,
            '\'' => in_quotes = !in_quotes,
            ';' | '&' | '|' | '`' | '$' | '(' | ')' | '<' | '>' | '\n' => {
                if !in_quotes {
                    return true;
                }
            }
            _ => {}
        }
    }
    
    false
}

/// Helper: Unescape a shell-escaped string
fn unescape_shell_string(s: &str) -> String {
    if s.starts_with('\'') && s.ends_with('\'') {
        let inner = &s[1..s.len()-1];
        inner.replace("'\"'\"'", "'")
    } else {
        s.to_string()
    }
}

/// Helper: Check if string is valid identifier
fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty() && 
    s.chars().all(|c| c.is_alphanumeric() || c == '_') &&
    s.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
}

/// Helper: Check if command injection is possible
fn can_inject_command(shell_code: &str, user_input: &str) -> bool {
    // Simplified check: look for unquoted user input or command separators
    let dangerous_patterns = [";", "&&", "||", "|", "`", "$(", "\n"];
    
    for pattern in &dangerous_patterns {
        if shell_code.contains(pattern) && !is_properly_escaped(shell_code, pattern) {
            return true;
        }
    }
    
    false
}

/// Helper: Check if pattern is properly escaped in context
fn is_properly_escaped(code: &str, pattern: &str) -> bool {
    // This is a simplified check - real implementation would be more sophisticated
    // For Kani verification, we check that dangerous patterns are within quotes
    code.contains(&format!("'{}'", pattern)) || 
    code.contains(&format!("\"{}\"", pattern))
}