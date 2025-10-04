// Chapter 4, Example 5: String Comparison
// Comparing string values

fn main() {
    let env = "production";

    if env == "production" {
        echo("Running in production");
    }

    if env != "development" {
        echo("Not in development");
    }
}

fn echo(msg: &str) {}
