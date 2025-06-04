// Integration tests for ShellCheck validation of generated scripts

use std::process::Command;
use std::path::Path;
use std::fs;

#[test]
fn test_shellcheck_installation() {
    let output = Command::new("which")
        .arg("shellcheck")
        .output();
    
    if output.is_err() || !output.unwrap().status.success() {
        // Try to install via make target
        let install_result = Command::new("make")
            .arg("shellcheck-install")
            .status();
            
        assert!(install_result.is_ok(), "Failed to install ShellCheck");
        assert!(install_result.unwrap().success(), "ShellCheck installation failed");
    }
}

#[test]
fn test_variable_quoting_sc2086() {
    let test_file = "tests/fixtures/shellcheck/sc2086_variable_quoting.rs";
    let output_file = "tests/shellcheck-output/sc2086_variable_quoting.sh";
    
    // Ensure output directory exists
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    // Build the test script
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok(), "Failed to execute rash build command");
    assert!(build_result.unwrap().success(), "Rash build failed for SC2086 test");
    
    // Verify file was created
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    // Run ShellCheck validation
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok(), "Failed to run ShellCheck");
    assert!(shellcheck_result.unwrap().success(), "ShellCheck validation failed for SC2086 test");
}

#[test]
fn test_command_substitution_sc2046() {
    let test_file = "tests/fixtures/shellcheck/sc2046_command_substitution.rs";
    let output_file = "tests/shellcheck-output/sc2046_command_substitution.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for SC2046 test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for SC2046 test");
}

#[test]
fn test_glob_protection_sc2035() {
    let test_file = "tests/fixtures/shellcheck/sc2035_glob_protection.rs";
    let output_file = "tests/shellcheck-output/sc2035_glob_protection.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for SC2035 test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for SC2035 test");
}

#[test]
fn test_cd_safety_sc2164() {
    let test_file = "tests/fixtures/shellcheck/sc2164_cd_safety.rs";
    let output_file = "tests/shellcheck-output/sc2164_cd_safety.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for SC2164 test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for SC2164 test");
}

#[test]
fn test_array_expansion_sc2068() {
    let test_file = "tests/fixtures/shellcheck/sc2068_array_expansion.rs";
    let output_file = "tests/shellcheck-output/sc2068_array_expansion.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for SC2068 test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for SC2068 test");
}

#[test]
fn test_modern_substitution_sc2006() {
    let test_file = "tests/fixtures/shellcheck/sc2006_modern_substitution.rs";
    let output_file = "tests/shellcheck-output/sc2006_modern_substitution.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for SC2006 test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for SC2006 test");
}

#[test]
fn test_safe_rm_sc2115() {
    let test_file = "tests/fixtures/shellcheck/sc2115_safe_rm.rs";
    let output_file = "tests/shellcheck-output/sc2115_safe_rm.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for SC2115 test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for SC2115 test");
}

#[test]
fn test_complex_installer() {
    let test_file = "tests/fixtures/shellcheck/complex_installer.rs";
    let output_file = "tests/shellcheck-output/complex_installer.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for complex installer test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for complex installer test");
}

#[test]
fn test_error_handling() {
    let test_file = "tests/fixtures/shellcheck/error_handling.rs";
    let output_file = "tests/shellcheck-output/error_handling.sh";
    
    fs::create_dir_all("tests/shellcheck-output").unwrap();
    
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "rash", "--", "build", test_file, "-o", output_file])
        .status();
        
    assert!(build_result.is_ok() && build_result.unwrap().success(), 
           "Rash build failed for error handling test");
    
    assert!(Path::new(output_file).exists(), "Output shell script was not created");
    
    let shellcheck_result = Command::new("shellcheck")
        .args(&["-s", "sh", output_file])
        .status();
        
    assert!(shellcheck_result.is_ok() && shellcheck_result.unwrap().success(), 
           "ShellCheck validation failed for error handling test");
}

#[test]
fn test_make_shellcheck_validate() {
    // Test the make target itself
    let result = Command::new("make")
        .arg("shellcheck-validate")
        .status();
        
    assert!(result.is_ok(), "Failed to execute make shellcheck-validate");
    assert!(result.unwrap().success(), "make shellcheck-validate failed");
}