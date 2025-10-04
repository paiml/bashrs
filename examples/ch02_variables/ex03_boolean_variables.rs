// Chapter 2, Example 3: Boolean Variables
// Boolean values transpile to shell-friendly representations

fn main() {
    let enabled = true;
    let debug = false;
    let force = true;

    echo("Boolean variables:");
    echo("enabled=true");
    echo("debug=false");
    echo("force=true");
}

fn echo(msg: &str) {}
