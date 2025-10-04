// Conditional Logic Demonstration
// Demonstrates: if/else branching, boolean logic, comparisons
// Use case: Making decisions based on environment or configuration

fn main() {
    let mode = env_var_or("MODE", "production");
    let debug = env_var_or("DEBUG", "false");
    let verbose = env_var_or("VERBOSE", "false");

    echo("=== Conditional Logic Demo ===");
    echo("");

    // Mode-based configuration
    echo("Checking deployment mode...");
    if mode == "production" {
        echo("✓ Running in PRODUCTION mode");
        echo("  - Optimizations enabled");
        echo("  - Debug logging disabled");
        echo("  - Strict error handling");
    } else if mode == "development" {
        echo("✓ Running in DEVELOPMENT mode");
        echo("  - Optimizations disabled");
        echo("  - Debug logging enabled");
        echo("  - Relaxed error handling");
    } else {
        echo("⚠ Unknown mode: {mode}");
        echo("  Defaulting to production settings");
    }

    echo("");

    // Debug flag checking
    echo("Checking debug settings...");
    if debug == "true" {
        echo("✓ Debug mode ENABLED");
        echo("  - Verbose output active");
        echo("  - Stack traces shown");
    } else {
        echo("✓ Debug mode DISABLED");
        echo("  - Minimal output");
    }

    echo("");

    // Verbose flag checking
    echo("Checking verbosity...");
    if verbose == "true" {
        echo("✓ Verbose mode ENABLED");
        echo("  - Detailed logging");
        echo("  - Progress indicators");
    } else {
        echo("  Standard verbosity");
    }

    echo("");
    echo("Configuration validated");
}

fn env_var_or(key: &str, default: &str) -> String {
    let value = env(key);
    if value == "" {
        default.to_string()
    } else {
        value
    }
}

fn env(key: &str) -> String { String::new() }
fn echo(msg: &str) {}
