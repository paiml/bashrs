// Complex installer script that tests multiple ShellCheck rules


fn main() {
    let version = "1.0.0";
    let binary_name = "myapp";
    let prefix = "/usr/local";
    let temp_dir = "/tmp";
    let user = "unknown";
    
    // Installation steps
    show_install_info(binary_name, version, user);
    create_directories(prefix, binary_name);
    download_binary(temp_dir, binary_name);
    install_binary(prefix, binary_name);
    cleanup(temp_dir);
}

fn show_install_info(name: &str, version: &str, user: &str) {
    // Show installation information
}

fn create_directories(prefix: &str, name: &str) {
    // Create installation directories
}

fn download_binary(temp_dir: &str, name: &str) {
    // Download binary to temp directory
}

fn install_binary(prefix: &str, name: &str) {
    // Install binary to prefix
}

fn cleanup(temp_dir: &str) {
    // Clean up temporary files
}