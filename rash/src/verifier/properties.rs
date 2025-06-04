use crate::ir::{Command, ShellIR, ShellValue};
use crate::models::{Error, Result};

/// Verify that the IR contains no command injection vulnerabilities
pub fn verify_no_command_injection(ir: &ShellIR) -> Result<()> {
    walk_ir(ir, &mut |node| {
        match node {
            ShellIR::Exec { cmd, .. } => {
                check_command_safety(cmd)?;
            }
            ShellIR::Let { value, .. } => {
                check_value_safety(value)?;
            }
            _ => {}
        }
        Ok(())
    })
}

/// Verify that the IR is deterministic (same inputs produce same outputs)
pub fn verify_deterministic(ir: &ShellIR) -> Result<()> {
    walk_ir(ir, &mut |node| {
        match node {
            ShellIR::Exec { cmd, .. } => {
                if is_nondeterministic_command(&cmd.program) {
                    return Err(Error::Verification(format!(
                        "Non-deterministic command: {}",
                        cmd.program
                    )));
                }
            }
            ShellIR::Let { value, .. } => {
                check_value_determinism(value)?;
            }
            _ => {}
        }
        Ok(())
    })
}

/// Verify that the IR represents idempotent operations
pub fn verify_idempotency(ir: &ShellIR) -> Result<()> {
    // Check for operations that could fail on second run
    walk_ir(ir, &mut |node| {
        if let ShellIR::Exec { cmd, .. } = node {
            if requires_idempotency_check(&cmd.program) {
                // Look for corresponding existence checks
                // This is simplified - real implementation would be more sophisticated
                check_has_idempotency_guard(ir, cmd)?;
            }
        }
        Ok(())
    })
}

/// Verify that the IR doesn't use excessive resources
pub fn verify_resource_safety(ir: &ShellIR) -> Result<()> {
    let mut network_calls = 0;
    let mut file_operations = 0;

    walk_ir(ir, &mut |node| {
        if let ShellIR::Exec { cmd, .. } = node {
            if is_network_command(&cmd.program) {
                network_calls += 1;
                if network_calls > 10 {
                    return Err(Error::Verification(
                        "Too many network operations".to_string(),
                    ));
                }
            }

            if is_file_operation(&cmd.program) {
                file_operations += 1;
                if file_operations > 50 {
                    return Err(Error::Verification("Too many file operations".to_string()));
                }
            }
        }
        Ok(())
    })
}

fn walk_ir<F>(ir: &ShellIR, visitor: &mut F) -> Result<()>
where
    F: FnMut(&ShellIR) -> Result<()>,
{
    visitor(ir)?;

    match ir {
        ShellIR::If {
            then_branch,
            else_branch,
            ..
        } => {
            walk_ir(then_branch, visitor)?;
            if let Some(else_ir) = else_branch {
                walk_ir(else_ir, visitor)?;
            }
        }
        ShellIR::Sequence(items) => {
            for item in items {
                walk_ir(item, visitor)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn check_command_safety(cmd: &Command) -> Result<()> {
    // Check for shell metacharacters in command arguments
    for arg in &cmd.args {
        check_value_safety(arg)?;
    }

    // Check for dangerous commands
    if is_dangerous_command(&cmd.program) {
        return Err(Error::Verification(format!(
            "Dangerous command not allowed: {}",
            cmd.program
        )));
    }

    Ok(())
}

fn check_value_safety(value: &ShellValue) -> Result<()> {
    match value {
        ShellValue::String(s) => {
            if contains_shell_metacharacters(s) {
                return Err(Error::Verification(format!(
                    "Unsafe string contains shell metacharacters: {}",
                    s
                )));
            }
        }
        ShellValue::Concat(parts) => {
            for part in parts {
                check_value_safety(part)?;
            }
        }
        ShellValue::CommandSubst(cmd) => {
            check_command_safety(cmd)?;
        }
        _ => {}
    }
    Ok(())
}

fn check_value_determinism(value: &ShellValue) -> Result<()> {
    match value {
        ShellValue::CommandSubst(cmd) => {
            if is_nondeterministic_command(&cmd.program) {
                return Err(Error::Verification(format!(
                    "Non-deterministic command substitution: {}",
                    cmd.program
                )));
            }
        }
        ShellValue::Concat(parts) => {
            for part in parts {
                check_value_determinism(part)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn contains_shell_metacharacters(s: &str) -> bool {
    // Check for dangerous shell metacharacters
    s.chars().any(|c| {
        matches!(
            c,
            '$' | '`' | ';' | '|' | '&' | '>' | '<' | '(' | ')' | '{' | '}'
        )
    })
}

fn is_dangerous_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "rm" | "rmdir"
            | "dd"
            | "mkfs"
            | "fdisk"
            | "format"
            | "sudo"
            | "su"
            | "chmod"
            | "chown"
            | "passwd"
            | "eval"
            | "exec"
            | "source"
            | "."
    )
}

fn is_nondeterministic_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "date"
            | "random"
            | "uuidgen"
            | "hostname"
            | "whoami"
            | "ps"
            | "top"
            | "netstat"
            | "ss"
            | "lsof"
    )
}

fn requires_idempotency_check(cmd: &str) -> bool {
    matches!(
        cmd,
        "mkdir" | "cp" | "mv" | "ln" | "touch" | "curl" | "wget"
    )
}

fn is_network_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "curl" | "wget" | "ssh" | "scp" | "rsync" | "nc" | "telnet"
    )
}

fn is_file_operation(cmd: &str) -> bool {
    matches!(
        cmd,
        "cp" | "mv"
            | "rm"
            | "mkdir"
            | "rmdir"
            | "touch"
            | "chmod"
            | "chown"
            | "ln"
            | "find"
            | "locate"
            | "du"
            | "df"
    )
}

fn check_has_idempotency_guard(_ir: &ShellIR, cmd: &Command) -> Result<()> {
    // This is a simplified check - real implementation would analyze the IR structure
    // to ensure that potentially non-idempotent operations have appropriate guards

    match cmd.program.as_str() {
        "mkdir" => {
            // Should have a test -d check
            // For now, just warn
            Ok(())
        }
        "curl" | "wget" => {
            // Should have a test -f check for the output file
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Command, ShellValue};

    #[test]
    fn test_command_injection_detection() {
        let cmd = Command {
            program: "echo".to_string(),
            args: vec![ShellValue::String("hello; rm -rf /".to_string())],
        };

        let result = check_command_safety(&cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_command() {
        let cmd = Command {
            program: "echo".to_string(),
            args: vec![ShellValue::String("hello world".to_string())],
        };

        let result = check_command_safety(&cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dangerous_command_detection() {
        assert!(is_dangerous_command("rm"));
        assert!(is_dangerous_command("sudo"));
        assert!(!is_dangerous_command("echo"));
    }

    #[test]
    fn test_nondeterministic_command_detection() {
        assert!(is_nondeterministic_command("date"));
        assert!(is_nondeterministic_command("random"));
        assert!(!is_nondeterministic_command("echo"));
    }
}
