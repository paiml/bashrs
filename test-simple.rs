fn main() {
    echo("Hello from Rash!");
    echo("This is a test");
}

fn echo(msg: &str) {
    // This will be converted to shell echo
    println!("{}", msg);
}