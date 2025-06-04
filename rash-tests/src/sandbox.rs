use anyhow::Result;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

pub struct Sandbox {
    temp_dir: TempDir,
}

impl Sandbox {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    pub fn run_shell_script(&self, script: &str) -> Result<Output> {
        let script_path = self.temp_dir.path().join("test_script.sh");
        std::fs::write(&script_path, script)?;

        let output = Command::new("sh")
            .arg(&script_path)
            .current_dir(&self.temp_dir)
            .output()?;

        Ok(output)
    }

    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }
}
