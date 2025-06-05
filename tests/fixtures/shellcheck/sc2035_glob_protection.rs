// Test SC2035: Use ./* to prevent filenames starting with dashes being interpreted as options


fn main() {
    // These patterns could be dangerous without proper protection
    remove_dash_rf();
    remove_verbose();
    remove_dash_n();
    remove_help();
    
    // List files that might start with dashes
    list_files();
}

fn remove_dash_rf() {
    // Remove files matching -rf pattern with protection
}

fn remove_verbose() {
    // Remove files matching --verbose pattern with protection
}

fn remove_dash_n() {
    // Remove files matching -n pattern with protection
}

fn remove_help() {
    // Remove files matching --help pattern with protection
}

fn list_files() {
    // List files with proper protection
}