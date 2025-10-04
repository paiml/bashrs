// Chapter 2, Example 7: Paths with Spaces
// Rash handles the notorious "path with spaces" problem

fn main() {
    let user_dir = "/home/user/My Documents";
    let app_name = "My App";
    let install_path = "/Program Files/MyApp";

    echo("Paths with spaces:");
    echo(user_dir);
    echo(app_name);
    echo(install_path);
}

fn echo(msg: &str) {}
