// Integration test for TestGenerator
// Tests the complete workflow: bash script -> AST -> generated tests

use bashrs::bash_parser::{BashParser, ast::*};
use bashrs::test_generator::{TestGenerator, TestGenOptions};
use std::fs;

#[test]
fn test_factorial_integration() {
    // Read the factorial bash script
    let bash_code = fs::read_to_string("../examples/factorial.sh")
        .expect("Failed to read factorial.sh");

    // Parse the bash script
    let mut parser = BashParser::new(&bash_code).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse bash script");

    // Verify AST structure
    assert!(!ast.statements.is_empty(), "AST should have statements");

    // Find the factorial function
    let has_factorial = ast.statements.iter().any(|stmt| {
        matches!(stmt, BashStmt::Function { name, .. } if name == "factorial")
    });
    assert!(has_factorial, "Should find factorial function");

    // Generate tests using TestGenerator
    let options = TestGenOptions::default();
    let mut generator = TestGenerator::new(options);
    let result = generator.generate(&ast);

    // Verify generation succeeded
    assert!(result.is_ok(), "Test generation should succeed");

    let test_suite = result.unwrap();

    // Verify we generated tests
    println!("Generated {} unit tests", test_suite.unit_tests.len());
    println!("Generated {} property tests", test_suite.property_tests.len());
    println!("Generated {} doctests", test_suite.doctests.len());

    assert!(!test_suite.unit_tests.is_empty(), "Should generate unit tests");

    // Verify doctest extraction from comments
    assert!(!test_suite.doctests.is_empty(), "Should extract doctests from comments");

    // Check for factorial-specific doctests
    let has_factorial_doctest = test_suite.doctests.iter().any(|dt| {
        dt.function_name == "factorial" && dt.example.contains("5")
    });
    assert!(has_factorial_doctest, "Should have factorial(5) => 120 doctest");
}

#[test]
fn test_generated_tests_compile() {
    // Read the factorial Rash script
    let bash_code = fs::read_to_string("../examples/factorial.sh")
        .expect("Failed to read factorial.sh");

    // Parse and generate
    let mut parser = BashParser::new(&bash_code).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    let options = TestGenOptions::default();
    let mut generator = TestGenerator::new(options);
    let test_suite = generator.generate(&ast).expect("Generation failed");

    // Generate Rust test code
    let unit_tests_code = test_suite.unit_tests.iter()
        .map(|test| test.to_rust_code())
        .collect::<Vec<_>>()
        .join("\n\n");

    // Verify test code contains expected patterns
    assert!(unit_tests_code.contains("#[test]"), "Should have test annotations");
    assert!(unit_tests_code.contains("fn test_"), "Should have test functions");

    println!("Generated unit test code sample:");
    println!("{}", unit_tests_code.lines().take(20).collect::<Vec<_>>().join("\n"));
}

#[test]
fn test_mutation_config_generation() {
    // Read the factorial Rash script
    let bash_code = fs::read_to_string("../examples/factorial.sh")
        .expect("Failed to read factorial.sh");

    // Parse
    let mut parser = BashParser::new(&bash_code).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    // Generate test suite (mutation config is part of it)
    let options = TestGenOptions::default();
    let mut generator = TestGenerator::new(options);
    let test_suite = generator.generate(&ast).expect("Generation failed");
    let config = &test_suite.mutation_config;

    // Verify config structure
    assert!(config.contains("timeout"), "Should have timeout setting");
    assert!(config.contains("jobs"), "Should have jobs setting");
    assert!(config.contains("exclude_globs"), "Should have exclude patterns");

    println!("Generated mutation config:");
    println!("{}", config);
}

#[test]
fn test_property_test_generation() {
    let bash_code = fs::read_to_string("../examples/factorial.sh")
        .expect("Failed to read factorial.sh");

    let mut parser = BashParser::new(&bash_code).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    let options = TestGenOptions::default();
    let mut generator = TestGenerator::new(options);
    let test_suite = generator.generate(&ast).expect("Generation failed");

    // Verify property tests were generated
    assert!(!test_suite.property_tests.is_empty(), "Should generate property tests");

    // Check for determinism property
    let has_determinism_test = test_suite.property_tests.iter().any(|pt| {
        matches!(pt.property, bashrs::test_generator::Property::Determinism)
    });

    assert!(has_determinism_test, "Should have determinism property test");

    println!("Generated {} property tests", test_suite.property_tests.len());
}
