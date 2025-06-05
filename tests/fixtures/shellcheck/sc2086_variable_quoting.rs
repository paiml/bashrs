// Test SC2086: Double quote to prevent globbing and word splitting

#[rash::main]
fn main() {
    let user = "unknown";
    let home = "/home/user";
    let path_with_spaces = "/path with spaces/file.txt";
    
    // These should all be properly quoted in the generated shell
    echo_user(user);
    echo_home(home);
    echo_path(path_with_spaces);
    
    // Command arguments that need quoting
    make_directory(path_with_spaces);
}

fn echo_user(user: &str) {
    // Echo user with proper quoting
}

fn echo_home(home: &str) {
    // Echo home with proper quoting
}

fn echo_path(path: &str) {
    // Echo path with proper quoting
}

fn make_directory(path: &str) {
    // Mkdir with proper quoting
}