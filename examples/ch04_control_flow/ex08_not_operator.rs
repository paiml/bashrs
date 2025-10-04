// Chapter 4, Example 8: NOT Operator (!)
// Negating conditions

fn main() {
    let enabled = false;

    if !enabled {
        echo("Feature is disabled");
    }
}

fn echo(msg: &str) {}
