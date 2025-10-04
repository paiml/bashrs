// Chapter 2, Example 4: Multiple Variable Types
// Mixing different variable types in one function

fn main() {
    let app_name = "myapp";
    let version = "1.0.0";
    let port = 3000;
    let ssl_enabled = true;

    echo("Application configuration:");
    echo("Name: myapp");
    echo("Version: 1.0.0");
    echo("Port: 3000");
    echo("SSL: enabled");
}

fn echo(msg: &str) {}
