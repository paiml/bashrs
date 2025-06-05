// Test SC2068: Double quote array expansions to avoid re-splitting elements

#[rash::main]
fn main() {
    // Find files with multiple extensions
    find_files_by_ext("*.txt");
    find_files_by_ext("*.rs");
    find_files_by_ext("*.md");
    find_files_by_ext("*.toml");
    
    // Search in multiple paths
    list_path("/usr/bin");
    list_path("/usr/local/bin");
    list_path("/bin");
    
    // Compile with multiple flags
    compile_with_flags();
}

fn find_files_by_ext(ext: &str) {
    // Find files with proper array quoting
}

fn list_path(path: &str) {
    // List directory with proper quoting
}

fn compile_with_flags() {
    // Compile with proper flag expansion
}