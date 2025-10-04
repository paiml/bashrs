// Chapter 4, Example 15: Installer Conditional Logic
// Real-world bootstrap installer pattern

fn main() {
    let mode = "install";
    let force = false;
    let version = "1.0.0";

    if mode == "install" {
        if !check_already_installed() {
            install_package(version);
        } else if force {
            echo("Forcing reinstall");
            reinstall_package(version);
        } else {
            echo("Already installed");
        }
    } else if mode == "uninstall" {
        if check_already_installed() {
            uninstall_package();
        } else {
            echo("Not installed");
        }
    }
}

fn check_already_installed() -> bool {
    false
}

fn install_package(v: &str) {
    echo("Installing package");
}

fn reinstall_package(v: &str) {
    echo("Reinstalling package");
}

fn uninstall_package() {
    echo("Uninstalling package");
}

fn echo(msg: &str) {}
