// User-Defined Functions
// Demonstrates: function definitions, parameters, return values
// Use case: Organizing complex logic into reusable functions

fn main() {
    echo("=== User-Defined Functions Demo ===");
    echo("");

    // Simple function calls
    echo("[1/3] Testing simple functions...");
    greet("Alice");
    greet("Bob");

    // Functions with return values
    echo("");
    echo("[2/3] Testing functions with return values...");
    let sum = add(10, 20);
    echo("10 + 20 = {sum}");

    let product = multiply(5, 7);
    echo("5 * 7 = {product}");

    // Complex function composition
    echo("");
    echo("[3/3] Testing function composition...");
    let result = calculate_total(100, 25);
    echo("Total after 25% increase: {result}");

    echo("");
    echo("All function tests passed");
}

fn greet(name: &str) {
    echo("Hello, {name}!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

fn calculate_total(base: i32, percent: i32) -> i32 {
    let increase = multiply(base, percent);
    let increase_amount = increase / 100;
    add(base, increase_amount)
}

fn echo(msg: &str) {}
