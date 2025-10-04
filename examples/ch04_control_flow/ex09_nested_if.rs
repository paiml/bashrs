// Chapter 4, Example 9: Nested If Statements
// Conditions within conditions

fn main() {
    let x = 15;
    let y = 20;

    if x > 10 {
        if y > 15 {
            echo("Both x > 10 and y > 15");
        } else {
            echo("x > 10 but y <= 15");
        }
    } else {
        echo("x <= 10");
    }
}

fn echo(msg: &str) {}
