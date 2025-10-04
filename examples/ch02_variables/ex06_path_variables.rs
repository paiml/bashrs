// Chapter 2, Example 6: Path Variables
// Common pattern for installation scripts

fn main() {
    let prefix = "/usr/local";
    let bin_dir = "/usr/local/bin";
    let lib_dir = "/usr/local/lib";
    let config_dir = "/etc/myapp";

    echo("Installation paths:");
    echo(prefix);
    echo(bin_dir);
    echo(lib_dir);
    echo(config_dir);
}

fn echo(msg: &str) {}
