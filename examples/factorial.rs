// Factorial Calculator
// Demonstrates: recursive functions, conditionals, arithmetic
// Example: factorial(5) => 120
// Example: factorial(0) => 1

fn main() {
    echo("=== Factorial Calculator ===");
    echo("");

    // Test base cases
    echo("Testing base cases:");
    let result0 = factorial(0);
    echo("factorial(0) = {result0}");

    let result1 = factorial(1);
    echo("factorial(1) = {result1}");

    // Test standard cases
    echo("");
    echo("Testing standard cases:");
    let result5 = factorial(5);
    echo("factorial(5) = {result5}");

    let result10 = factorial(10);
    echo("factorial(10) = {result10}");
}

/// Calculate factorial recursively
/// Example: factorial(5) => 120
/// Example: factorial(0) => 1
/// Example: factorial(1) => 1
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        let prev = factorial(n - 1);
        n * prev
    }
}

fn echo(msg: &str) {}
