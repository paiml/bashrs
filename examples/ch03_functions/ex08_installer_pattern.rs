// Chapter 3, Example 8: Bootstrap Installer Pattern
// Real-world installer with multiple stages

fn main() {
    let app = "myapp";
    let version = "1.0.0";
    let prefix = "/usr/local";

    check_prerequisites(app);
    download_binary(app, version);
    verify_checksum(app, version);
    install_binary(app, prefix);
    create_config(app, prefix);
    setup_service(app);
}

fn check_prerequisites(name: &str) {
    echo("Checking prerequisites");
}

fn download_binary(name: &str, ver: &str) {
    echo("Downloading binary");
}

fn verify_checksum(name: &str, ver: &str) {
    echo("Verifying checksum");
}

fn install_binary(name: &str, prefix: &str) {
    echo("Installing binary");
}

fn create_config(name: &str, prefix: &str) {
    echo("Creating config");
}

fn setup_service(name: &str) {
    echo("Setting up service");
}

fn echo(msg: &str) {}
