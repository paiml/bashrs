// Chapter 3, Example 6: Multiple Helper Functions
// Organizing code with helper functions

fn main() {
    setup_environment();
    install_dependencies();
    configure_system();
    start_services();
}

fn setup_environment() {
    echo("Setting up environment");
}

fn install_dependencies() {
    echo("Installing dependencies");
}

fn configure_system() {
    echo("Configuring system");
}

fn start_services() {
    echo("Starting services");
}

fn echo(msg: &str) {}
