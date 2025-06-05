// Test SC2046: Quote command substitutions to prevent word splitting

#[rash::main]
fn main() {
    // Command substitutions that should be quoted
    find_txt_files();
    show_current_dir();
    count_files();
}

fn find_txt_files() {
    // Find txt files with proper quoting
}

fn show_current_dir() {
    // Show current directory with proper quoting
}

fn count_files() {
    // Count files with proper quoting
}