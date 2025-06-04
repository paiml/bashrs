//! Example demonstrating the formal verification module

use rash::formal::semantics::{posix_semantics, rash_semantics};
use rash::formal::{AbstractState, FormalEmitter, TinyAst};

fn main() -> anyhow::Result<()> {
    println!("=== Formal Verification Example ===\n");

    // Create a bootstrap script AST
    let bootstrap_ast = TinyAst::Sequence {
        commands: vec![
            // Set installation directory
            TinyAst::SetEnvironmentVariable {
                name: "INSTALL_DIR".to_string(),
                value: "/opt/rash".to_string(),
            },
            // Create directories
            TinyAst::ExecuteCommand {
                command_name: "mkdir".to_string(),
                args: vec!["-p".to_string(), "/opt/rash/bin".to_string()],
            },
            // Echo status
            TinyAst::ExecuteCommand {
                command_name: "echo".to_string(),
                args: vec!["Creating installation directory...".to_string()],
            },
            // Change to install directory
            TinyAst::ChangeDirectory {
                path: "/opt/rash".to_string(),
            },
            // Echo completion
            TinyAst::ExecuteCommand {
                command_name: "echo".to_string(),
                args: vec!["Installation directory ready".to_string()],
            },
        ],
    };

    // Verify the AST is valid
    if !bootstrap_ast.is_valid() {
        anyhow::bail!("Invalid AST");
    }

    // Emit POSIX shell code
    let shell_script = FormalEmitter::emit(&bootstrap_ast);
    println!("Generated Shell Script:");
    println!("```bash");
    println!("{}", shell_script);
    println!("```\n");

    // Verify semantic equivalence
    println!("Verifying Semantic Equivalence...");

    let mut initial_state = AbstractState::new();
    // Add /opt directory for the test
    initial_state.filesystem.insert(
        std::path::PathBuf::from("/opt"),
        rash::formal::FileSystemEntry::Directory,
    );

    // Evaluate rash AST
    let rash_result = rash_semantics::eval_rash(&bootstrap_ast, initial_state.clone())
        .map_err(|e| anyhow::anyhow!("Rash evaluation failed: {}", e))?;

    // Evaluate POSIX code
    let posix_result = posix_semantics::eval_posix(&shell_script, initial_state)
        .map_err(|e| anyhow::anyhow!("POSIX evaluation failed: {}", e))?;

    // Check equivalence
    if rash_result.is_equivalent(&posix_result) {
        println!("✓ Semantic equivalence verified!");
        println!("\nFinal state:");
        println!(
            "  Environment: INSTALL_DIR = {:?}",
            rash_result.get_env("INSTALL_DIR")
        );
        println!("  Current directory: {:?}", rash_result.cwd);
        println!("  Created directories: /opt/rash/bin");
        println!("  Output:");
        for line in &rash_result.stdout {
            println!("    {}", line);
        }
    } else {
        println!("✗ Semantic equivalence failed!");
        println!("Rash state: {:?}", rash_result);
        println!("POSIX state: {:?}", posix_result);
    }

    Ok(())
}
