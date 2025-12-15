#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)] // Examples can use unwrap() for simplicity
//! Example: Makefile Purification with Test Generation
//!
//! This example demonstrates the `bashrs make purify --with-tests` feature,
//! which generates comprehensive test suites for purified Makefiles.
//!
//! ## Features Demonstrated
//! - Makefile parsing and purification
//! - Automatic test suite generation
//! - Determinism testing
//! - Idempotency testing
//! - POSIX compliance testing
//!
//! ## Usage
//! ```bash
//! cargo run --example makefile_purify_with_tests
//! ```

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Makefile Purification with Test Generation Example");
    println!("======================================================\n");

    // Create temporary directory for our example
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Step 1: Create a sample Makefile with non-deterministic issues
    println!("📝 Step 1: Creating sample Makefile with issues...");
    let makefile_path = temp_path.join("Makefile");
    let makefile_content = r#"# Example Makefile with non-deterministic behavior

.PHONY: all build test clean install

# Variables
PROJECT = myapp
VERSION = 1.0.0
BUILD_DIR = build
INSTALL_DIR = /usr/local/bin

all: build test

# Build target with timestamp (non-deterministic)
build:
	@echo "Building $(PROJECT) v$(VERSION)"
	mkdir $(BUILD_DIR)
	@echo "Build timestamp: $(shell date)" > $(BUILD_DIR)/build.txt
	@echo "Build ID: $$RANDOM" >> $(BUILD_DIR)/build.txt
	gcc -o $(BUILD_DIR)/$(PROJECT) src/main.c

# Test target
test:
	@echo "Running tests..."
	./$(BUILD_DIR)/$(PROJECT) --test

# Install target (not idempotent)
install:
	cp $(BUILD_DIR)/$(PROJECT) $(INSTALL_DIR)/$(PROJECT)
	chmod +x $(INSTALL_DIR)/$(PROJECT)

# Clean target
clean:
	rm -rf $(BUILD_DIR)
"#;
    fs::write(&makefile_path, makefile_content)?;
    println!("✅ Created Makefile at: {}", makefile_path.display());
    println!("   Issues: Uses $RANDOM, timestamps, mkdir without -p\n");

    // Step 2: Run bashrs make purify with --with-tests
    println!("🔄 Step 2: Purifying Makefile with test generation...");
    let output_makefile = temp_path.join("Makefile.purified");
    let test_file = temp_path.join("Makefile.purified.test.sh");

    let status = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "bashrs",
            "--",
            "make",
            "purify",
            makefile_path.to_str().unwrap(),
            "--with-tests",
            "-o",
            output_makefile.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        eprintln!("❌ Failed to purify Makefile");
        return Err("Purification failed".into());
    }

    println!("✅ Purification complete!");
    println!("   Output: {}", output_makefile.display());
    println!("   Tests:  {}\n", test_file.display());

    // Step 3: Show the purified Makefile
    println!("📄 Step 3: Purified Makefile contents:");
    println!("─────────────────────────────────────");
    let purified_content = fs::read_to_string(&output_makefile)?;
    for (i, line) in purified_content.lines().enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }
    println!("─────────────────────────────────────\n");

    // Step 4: Show the generated test suite
    println!("🧪 Step 4: Generated test suite:");
    println!("─────────────────────────────────────");
    let test_content = fs::read_to_string(&test_file)?;
    let test_lines: Vec<&str> = test_content.lines().collect();

    // Show first 50 lines of test suite
    let lines_to_show = test_lines.len().min(50);
    for (i, line) in test_lines.iter().take(lines_to_show).enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }

    if test_lines.len() > 50 {
        println!("... ({} more lines)", test_lines.len() - 50);
    }
    println!("─────────────────────────────────────\n");

    // Step 5: Verify test file is valid shell
    println!("✅ Step 5: Validating test suite syntax...");
    let validate_status = Command::new("sh")
        .args(["-n", test_file.to_str().unwrap()])
        .status()?;

    if validate_status.success() {
        println!("✅ Test suite has valid POSIX shell syntax\n");
    } else {
        println!("❌ Test suite has syntax errors\n");
    }

    // Step 6: Show key improvements
    println!("🎯 Step 6: Key Improvements Made:");
    println!("─────────────────────────────────────");

    let improvements = vec![
        ("Determinism", "Removed $RANDOM and timestamps", "✅"),
        ("Idempotency", "Added -p flag to mkdir", "✅"),
        ("Safety", "Added -f flag to rm", "✅"),
        ("Testing", "Generated comprehensive test suite", "✅"),
        ("POSIX", "Ensured POSIX compliance", "✅"),
    ];

    for (category, improvement, status) in improvements {
        println!("{} {:12} - {}", status, category, improvement);
    }
    println!("─────────────────────────────────────\n");

    // Step 7: Explain the test suite
    println!("📚 Step 7: Test Suite Includes:");
    println!("─────────────────────────────────────");
    println!("✓ Determinism Test");
    println!("  - Runs make twice and compares outputs");
    println!("  - Ensures same input produces same output");
    println!();
    println!("✓ Idempotency Test");
    println!("  - Runs make multiple times");
    println!("  - Ensures safe to re-run without errors");
    println!();
    println!("✓ POSIX Compliance Test");
    println!("  - Verifies Makefile works with POSIX make");
    println!("  - Tests cross-platform compatibility");
    println!();
    println!("✓ Test Runner");
    println!("  - Orchestrates all tests");
    println!("  - Provides summary report");
    println!("─────────────────────────────────────\n");

    // Step 8: Show how to run the tests
    println!("🚀 Step 8: Running the Tests (Optional)");
    println!("─────────────────────────────────────");
    println!("To run the generated tests:");
    println!();
    println!("  cd {}", temp_path.display());
    println!(
        "  chmod +x {}",
        test_file.file_name().unwrap().to_str().unwrap()
    );
    println!("  ./{}", test_file.file_name().unwrap().to_str().unwrap());
    println!();
    println!("Or with sh:");
    println!("  sh {}", test_file.display());
    println!("─────────────────────────────────────\n");

    // Step 9: Summary statistics
    println!("📊 Step 9: Summary Statistics:");
    println!("─────────────────────────────────────");
    let original_lines = makefile_content.lines().count();
    let purified_lines = purified_content.lines().count();
    let test_lines_count = test_content.lines().count();

    println!("Original Makefile:  {:3} lines", original_lines);
    println!("Purified Makefile:  {:3} lines", purified_lines);
    println!("Generated Tests:    {:3} lines", test_lines_count);
    println!();
    println!("Test Coverage:");
    println!("  - Core tests: 3 (determinism, idempotency, POSIX)");
    println!("  - Test runner: 1 orchestration script");
    println!("  - Total test functions: 4");
    println!("─────────────────────────────────────\n");

    // Step 10: Next steps
    println!("🎓 Step 10: Learn More:");
    println!("─────────────────────────────────────");
    println!("Command-line usage:");
    println!("  bashrs make purify Makefile --with-tests -o output/Makefile");
    println!();
    println!("With property-based tests:");
    println!("  bashrs make purify Makefile --with-tests --property-tests -o output/Makefile");
    println!();
    println!("Documentation:");
    println!("  - README.md: Main documentation");
    println!("  - CHANGELOG.md: Feature details");
    println!("  - book/: Full user guide");
    println!("─────────────────────────────────────\n");

    println!("✨ Example Complete!");
    println!();
    println!("📁 Files generated in: {}", temp_path.display());
    println!("   (Will be cleaned up automatically on exit)");

    Ok(())
}
