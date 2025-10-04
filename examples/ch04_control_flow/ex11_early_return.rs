// Chapter 4, Example 11: Early Return Pattern
// Exit function early on condition

fn main() {
    validate_and_execute();
}

fn validate_and_execute() {
    let valid = check_preconditions();

    if !valid {
        echo("Preconditions failed");
        return;
    }

    echo("Executing main logic");
    execute();
}

fn check_preconditions() -> bool {
    true
}

fn execute() {
    echo("Execution complete");
}

fn echo(msg: &str) {}
