// Test SC2115: Use ${var:?} to ensure variable is not empty before rm


fn main() {
    // Safe removal operations - should validate variables are not empty
    let temp_dir = "/tmp";
    let build_dir = "./build";
    let cache_dir = "./cache";
    
    // Use safe rm operations that validate variables
    rm_safe(temp_dir);
    rm_build_dir(build_dir);
    find_and_delete(cache_dir);
}

fn rm_safe(dir: &str) {
    // Safe rm operation
}

fn rm_build_dir(dir: &str) {
    // Safe build directory removal
}

fn find_and_delete(dir: &str) {
    // Safe find and delete operation
}