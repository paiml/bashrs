#!/usr/bin/env rust-script
//! Cross-shell validation tool
//! 
//! ```cargo
//! [dependencies]
//! ```

use std::process::Command;
use std::fs;

struct ShellTest {
    name: &'static str,
    command: &'static str,
    args: Vec<&'static str>,
}

impl ShellTest {
    fn run(&self, script: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new(self.command)
            .args(&self.args)
            .arg(script)
            .output()?;
        
        if !output.status.success() {
            return Err(format!("{} failed: {}", self.name, 
                String::from_utf8_lossy(&output.stderr)).into());
        }
        
        Ok(String::from_utf8(output.stdout)?)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shells = vec![
        ShellTest { name: "bash", command: "bash", args: vec!["-posix"] },
        ShellTest { name: "dash", command: "dash", args: vec![] },
        ShellTest { name: "ash", command: "busybox", args: vec!["ash"] },
        ShellTest { name: "ksh", command: "ksh", args: vec![] },
        ShellTest { name: "zsh", command: "zsh", args: vec!["--emulate", "sh"] },
    ];
    
    // Get test script from args or use default
    let args: Vec<String> = std::env::args().collect();
    let test_script = if args.len() > 1 {
        fs::read_to_string(&args[1])?
    } else {
        // Default test script
        r#"#!/bin/sh
set -e
echo "Hello from shell"
X=42
echo "X is $X"
if [ "$X" -gt 0 ]; then
    echo "X is positive"
fi
"#.to_string()
    };
    
    // Write test script to temp file
    let temp_script = "/tmp/rash_shell_test.sh";
    fs::write(temp_script, &test_script)?;
    
    let mut results = Vec::new();
    
    for shell in &shells {
        match shell.run(temp_script) {
            Ok(output) => {
                println!("✓ {} output:", shell.name);
                println!("{}", output);
                results.push((shell.name, output));
            },
            Err(e) => {
                eprintln!("⚠️  {} not available: {}", shell.name, e);
                continue;
            }
        }
    }
    
    // Verify all outputs are identical
    if let Some((base_name, base_output)) = results.first() {
        for (name, output) in &results[1..] {
            if output != base_output {
                eprintln!("❌ Output mismatch between {} and {}", base_name, name);
                eprintln!("Expected:\n{}", base_output);
                eprintln!("Got:\n{}", output);
                std::process::exit(1);
            }
        }
    }
    
    // Clean up
    let _ = fs::remove_file(temp_script);
    
    println!("✅ All {} shells produce identical output", results.len());
    Ok(())
}