// Chapter 3, Example 7: Nested Function Calls
// Deep function call chains for complex logic

fn main() {
    deploy();
}

fn deploy() {
    prepare();
}

fn prepare() {
    validate();
}

fn validate() {
    check_requirements();
}

fn check_requirements() {
    echo("Requirements checked");
}

fn echo(msg: &str) {}
