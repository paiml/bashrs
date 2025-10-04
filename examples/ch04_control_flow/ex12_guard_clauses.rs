// Chapter 4, Example 12: Guard Clauses Pattern
// Multiple guard clauses for validation

fn main() {
    deploy("myapp", "production");
}

fn deploy(app: &str, env: &str) {
    if app == "" {
        echo("Error: app name required");
        return;
    }

    if env != "production" && env != "staging" {
        echo("Error: invalid environment");
        return;
    }

    echo("Deploying...");
    execute_deploy(app, env);
}

fn execute_deploy(a: &str, e: &str) {
    echo("Deploy complete");
}

fn echo(msg: &str) {}
