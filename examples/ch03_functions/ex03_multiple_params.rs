// Chapter 3, Example 3: Multiple Parameters (Same Type)
// Functions can accept multiple parameters of the same type

fn main() {
    show_info("myapp", "1.0.0", "/usr/local");
}

fn show_info(name: &str, version: &str, prefix: &str) {
    echo(name);
    echo(version);
    echo(prefix);
}

fn echo(msg: &str) {}
