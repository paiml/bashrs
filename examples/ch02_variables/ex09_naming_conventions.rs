// Chapter 2, Example 9: Variable Naming Conventions
// Rash preserves Rust naming conventions in generated shell

fn main() {
    let user_name = "alice";
    let home_directory = "/home/alice";
    let max_connections = 100;
    let is_admin = false;

    echo("User configuration:");
    echo(user_name);
    echo(home_directory);
    echo("Max connections: 100");
    echo("Admin: false");
}

fn echo(msg: &str) {}
