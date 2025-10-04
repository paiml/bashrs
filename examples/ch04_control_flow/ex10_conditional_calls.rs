// Chapter 4, Example 10: Conditional Function Calls
// Using if to control which functions execute

fn main() {
    let mode = "install";

    if mode == "install" {
        install();
    } else if mode == "uninstall" {
        uninstall();
    } else if mode == "upgrade" {
        upgrade();
    } else {
        show_help();
    }
}

fn install() {
    echo("Installing...");
}

fn uninstall() {
    echo("Uninstalling...");
}

fn upgrade() {
    echo("Upgrading...");
}

fn show_help() {
    echo("Usage: command [install|uninstall|upgrade]");
}

fn echo(msg: &str) {}
