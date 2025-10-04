// Chapter 3, Example 2: Function with One Parameter
// Single parameter functions demonstrate type-safe passing

fn main() {
    let name = "Alice";
    greet(name);
}

fn greet(name: &str) {
    echo(name);
}

fn echo(msg: &str) {}
