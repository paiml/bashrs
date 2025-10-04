// Chapter 2, Example 2: Integer Variables
// Rash handles numeric literals safely

fn main() {
    let port = 8080;
    let workers = 4;
    let timeout = 30;

    // Demonstrate integer variables work
    echo("Configuration:");
    echo("Port: 8080");
    echo("Workers: 4");
    echo("Timeout: 30");
}

fn echo(msg: &str) {}
