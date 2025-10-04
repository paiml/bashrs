// Chapter 4, Example 4: Integer Comparison Operators
// All numeric comparison operators

fn main() {
    let x = 10;
    let y = 20;

    if x == y {
        echo("equal");
    }

    if x != y {
        echo("not equal");
    }

    if x < y {
        echo("less than");
    }

    if x > y {
        echo("greater than");
    }
}

fn echo(msg: &str) {}
