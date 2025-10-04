// Chapter 3, Example 4: Mixed Type Parameters
// Combining strings, integers, and booleans

fn main() {
    configure("myapp", 8080, true);
}

fn configure(name: &str, port: i32, enabled: bool) {
    echo(name);
    echo("Port: 8080");
    echo("Enabled: true");
}

fn echo(msg: &str) {}
