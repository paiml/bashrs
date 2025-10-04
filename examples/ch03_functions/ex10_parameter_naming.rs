// Chapter 3, Example 10: Parameter Naming Patterns
// Clear, descriptive parameter names

fn main() {
    create_database("mydb", "localhost", 5432, "admin", "secure_pass");
}

fn create_database(
    database_name: &str,
    host: &str,
    port: i32,
    admin_user: &str,
    admin_password: &str
) {
    connect(host, port);
    authenticate(admin_user, admin_password);
    initialize(database_name);
}

fn connect(h: &str, p: i32) {
    echo("Connecting to database");
}

fn authenticate(u: &str, pwd: &str) {
    echo("Authenticating");
}

fn initialize(db: &str) {
    echo("Initializing database");
}

fn echo(msg: &str) {}
