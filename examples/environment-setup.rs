// Development Environment Setup
// Demonstrates: environment variable management, PATH manipulation
// Use case: Setting up development environment for a project

fn main() {
    let project_name = "my-project";
    let project_root = env_var_or("PROJECT_ROOT", "/opt/projects/my-project");

    echo("=== Development Environment Setup ===");
    echo("Project: {project_name}");
    echo("Root:    {project_root}");
    echo("");

    // Create project structure
    echo("[1/4] Creating project structure...");
    create_project_structure(project_root);

    // Set environment variables
    echo("[2/4] Setting environment variables...");
    setup_environment(project_root);

    // Create activation script
    echo("[3/4] Creating activation script...");
    create_activation_script(project_root);

    // Summary
    echo("[4/4] Environment setup complete");
    echo("");
    echo("To activate this environment, run:");
    echo("  source {project_root}/bin/activate.sh");
}

fn create_project_structure(root: &str) {
    let bin_dir = "{root}/bin";
    let lib_dir = "{root}/lib";
    let include_dir = "{root}/include";

    mkdir_p(bin_dir);
    mkdir_p(lib_dir);
    mkdir_p(include_dir);

    echo("  ✓ Created bin/");
    echo("  ✓ Created lib/");
    echo("  ✓ Created include/");
}

fn setup_environment(root: &str) {
    echo("  Setting PROJECT_ROOT={root}");
    echo("  Adding {root}/bin to PATH");
    echo("  Setting LD_LIBRARY_PATH={root}/lib");
}

fn create_activation_script(root: &str) {
    let script_path = "{root}/bin/activate.sh";
    let script_content = "#!/bin/sh\nexport PROJECT_ROOT={root}\nexport PATH={root}/bin:$PATH\nexport LD_LIBRARY_PATH={root}/lib:$LD_LIBRARY_PATH\necho \"Environment activated for {root}\"\n";

    write_file(script_path, script_content);
    echo("  ✓ Created activation script: {script_path}");
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
fn mkdir_p(path: &str) {}
fn write_file(path: &str, content: &str) {}
