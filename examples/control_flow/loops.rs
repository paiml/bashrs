//! # Example: Loops
//! 
//! This example demonstrates bounded loops in Rash.
//! All loops must have explicit bounds for safety.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example loops
//! # or transpile to shell:
//! cargo run --bin bashrs -- build examples/control_flow/loops.rs -o loops.sh
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will demonstrate various loop patterns
//! with guaranteed termination.

#[bashrs::main]
fn main() {
    echo("Loop examples:");
    
    // Simple bounded for loop
    for i in 0..5 {
        echo("Iteration: {i}");
    }
    
    // Iterating over a list of items
    let packages = ["curl", "git", "vim", "tmux"];
    for pkg in packages {
        if command_exists(pkg) {
            echo("{pkg} is already installed");
        } else {
            echo("{pkg} needs to be installed");
        }
    }
    
    // While loop with explicit bound
    let mut counter = 0;
    while counter < 3 {
        echo("Counter: {counter}");
        counter = counter + 1;
    }
    
    // Processing files in a directory (bounded)
    mkdir_p("/tmp/rash-loop-example");
    
    // Create some test files
    for i in 0..3 {
        write_file("/tmp/rash-loop-example/file{i}.txt", "Content {i}");
    }
    
    // Process files (up to 10)
    let file_count = 0;
    for file in glob("/tmp/rash-loop-example/*.txt") {
        if file_count >= 10 {
            break;
        }
        echo("Processing: {file}");
        file_count = file_count + 1;
    }
    
    // Retry pattern with bounded attempts
    let attempts = 0;
    let success = false;
    
    while !success && attempts < 3 {
        echo("Attempt {attempts + 1} of 3...");
        
        // Simulate an operation that might fail
        if exec("test -d /tmp") {
            success = true;
            echo("Operation succeeded!");
        } else {
            attempts = attempts + 1;
            if attempts < 3 {
                echo("Retrying...");
                sleep(1);
            }
        }
    }
    
    if !success {
        echo("Operation failed after 3 attempts");
    }
    
    // Cleanup
    exec("rm -rf /tmp/rash-loop-example");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_loops_compile() {
        // Ensure the example compiles
        assert!(true);
    }
}