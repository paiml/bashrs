// Chapter 4, Example 3: If-Else-If-Else Chain
// Multiple conditions with fallback

fn main() {
    let x = 42;
    if x < 0 {
        echo("negative");
    } else if x == 0 {
        echo("zero");
    } else if x < 100 {
        echo("positive, less than 100");
    } else {
        echo("100 or greater");
    }
}

fn echo(msg: &str) {}
