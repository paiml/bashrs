// Chapter 4, Example 7: Logical OR (||)
// At least one condition must be true

fn main() {
    let mode = "debug";

    if mode == "debug" || mode == "test" {
        echo("Development mode");
    }
}

fn echo(msg: &str) {}
