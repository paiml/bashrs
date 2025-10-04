// Chapter 4, Example 14: Boolean Variable Conditions
// Using boolean variables directly

fn main() {
    let ssl_enabled = true;
    let debug_mode = false;

    if ssl_enabled {
        echo("SSL is enabled");
    }

    if !debug_mode {
        echo("Debug mode is off");
    }

    if ssl_enabled && !debug_mode {
        echo("Production-ready");
    }
}

fn echo(msg: &str) {}
