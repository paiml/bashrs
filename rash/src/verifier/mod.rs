pub mod properties;

#[cfg(kani)]
pub mod kani_harnesses;

use crate::ir::ShellIR;
use crate::models::{VerificationLevel, Result};

/// Verify that the given IR satisfies safety properties
pub fn verify(ir: &ShellIR, level: VerificationLevel) -> Result<()> {
    match level {
        VerificationLevel::None => Ok(()),
        VerificationLevel::Basic => verify_basic(ir),
        VerificationLevel::Strict => verify_strict(ir),
        VerificationLevel::Paranoid => verify_paranoid(ir),
    }
}

fn verify_basic(ir: &ShellIR) -> Result<()> {
    // Basic verification: check for obvious safety issues
    properties::verify_no_command_injection(ir)?;
    Ok(())
}

fn verify_strict(ir: &ShellIR) -> Result<()> {
    // Strict verification: all basic checks plus determinism
    verify_basic(ir)?;
    properties::verify_deterministic(ir)?;
    Ok(())
}

fn verify_paranoid(ir: &ShellIR) -> Result<()> {
    // Paranoid verification: all checks plus formal verification
    verify_strict(ir)?;
    properties::verify_idempotency(ir)?;
    properties::verify_resource_safety(ir)?;
    Ok(())
}