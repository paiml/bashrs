// Chapter 4, Example 13: Complex Boolean Logic
// Combining AND, OR, and NOT

fn main() {
    let x = 10;
    let y = 20;
    let z = 30;

    if (x > 5 && y < 25) || z == 30 {
        echo("Complex condition satisfied");
    }
}

fn echo(msg: &str) {}
