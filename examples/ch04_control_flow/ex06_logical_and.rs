// Chapter 4, Example 6: Logical AND (&&)
// Multiple conditions must all be true

fn main() {
    let x = 10;
    let y = 20;

    if x > 5 && y > 15 {
        echo("Both conditions true");
    }
}

fn echo(msg: &str) {}
