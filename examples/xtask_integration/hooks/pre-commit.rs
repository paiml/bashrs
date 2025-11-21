/// Example git pre-commit hook written in Rust
///
/// This will be transpiled to a POSIX shell script by bashrs.
///
/// Features demonstrated:
/// - Variable assignments
/// - Command execution (simulated with echo)
/// - Conditional logic
/// - Exit codes

fn main() {
    let hook_name = "pre-commit";
    echo("Running pre-commit hook...");

    // Check for debugging files
    let has_debug = false;

    if has_debug {
        eprintln("Error: Debug files detected");
        exit(1);
    }

    echo("✓ Pre-commit checks passed");
    exit(0);
}

fn echo(msg: &str) {}
fn eprintln(msg: &str) {}
fn exit(code: i32) {}
