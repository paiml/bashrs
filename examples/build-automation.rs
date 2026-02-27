// Build Automation Script
// Demonstrates: multi-step workflows, build patterns
// Use case: CI/CD build and deployment automation

fn main() {
    let build_type = env_var_or("BUILD_TYPE", "release");
    let target = env_var_or("TARGET", "x86_64-unknown-linux-gnu");

    echo("=== Build Automation ===");
    echo("Build Type: {build_type}");
    echo("Target:     {target}");
    echo("");

    // Clean previous builds
    echo("[1/5] Cleaning previous builds...");
    clean_build();

    // Check dependencies
    echo("[2/5] Checking dependencies...");
    if !check_dependencies() {
        echo("✗ Dependency check failed");
        exit(1);
    }
    echo("✓ All dependencies available");

    // Compile
    echo("[3/5] Compiling...");
    compile(build_type);

    // Run tests
    echo("[4/5] Running tests...");
    if !run_tests() {
        echo("✗ Tests failed");
    } else {
        echo("✓ All tests passed");
    }

    // Package
    echo("[5/5] Packaging...");
    package(build_type, target);

    echo("");
    echo("Build completed successfully");
    echo("Artifacts: ./target/{target}/{build_type}/");
}

fn clean_build() {
    echo("  Removing ./target directory...");
    echo("  → Clean simulated");
}

fn check_dependencies() -> bool {
    echo("  Checking build tools...");
    echo("  ✓ Compiler found");
    echo("  ✓ Linker found");
    echo("  ✓ Test framework found");
    true
}

fn compile(build_type: &str) {
    if build_type == "release" {
        echo("  Compiling with optimizations...");
    } else {
        echo("  Compiling in debug mode...");
    }
    echo("  → Compilation simulated");
}

fn run_tests() -> bool {
    echo("  Running unit tests...");
    echo("  Running integration tests...");
    echo("  → All tests simulated as passing");
    true
}

fn package(build_type: &str, target: &str) {
    echo("  Creating package for {target}...");
    echo("  Build type: {build_type}");
    echo("  → Packaging simulated");
}

fn env_var_or(key: &str, default: &str) -> String {
    let value = env(key);
    if value == "" {
        default.to_string()
    } else {
        value
    }
}

fn env(key: &str) -> String {
    String::new()
}
fn echo(msg: &str) {}
