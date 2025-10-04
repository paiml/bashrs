// Chapter 2, Example 5: Variables with Special Characters
// Rash automatically escapes special characters in variable values

fn main() {
    let path = "/usr/local/bin";
    let price = "$100";
    let message = "He said \"hello\"";
    let pattern = "*.txt";

    echo("Special characters test:");
    echo(path);
    echo(price);
    echo(message);
    echo(pattern);
}

fn echo(msg: &str) {}
