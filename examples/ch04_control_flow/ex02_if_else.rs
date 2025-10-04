// Chapter 4, Example 2: If-Else
// Basic branching with two paths

fn main() {
    let enabled = true;
    if enabled {
        echo("Feature enabled");
    } else {
        echo("Feature disabled");
    }
}

fn echo(msg: &str) {}
