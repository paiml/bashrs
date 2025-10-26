//! Compiler module for bashrs
//!
//! ## Safety Note
//! Compiler uses unwrap() on validated AST operations and checked invariants.
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use crate::models::{Error, Result};
use std::io::Write;

pub mod loader;
pub mod optimize;

#[derive(Debug, Clone, Copy)]
pub enum RuntimeType {
    Dash,    // 180KB static binary
    Busybox, // 900KB with coreutils
    Minimal, // 50KB custom interpreter
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionLevel {
    Fast,
    Balanced,
    Best,
}

impl CompressionLevel {
    pub fn level(&self) -> i32 {
        match self {
            CompressionLevel::Fast => 3,
            CompressionLevel::Balanced => 11,
            CompressionLevel::Best => 19,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StripLevel {
    None,
    Debug,
    All,
}

pub struct BinaryCompiler {
    runtime: RuntimeType,
    compression: CompressionLevel,
    strip_level: StripLevel,
}

impl BinaryCompiler {
    pub fn new(runtime: RuntimeType) -> Self {
        Self {
            runtime,
            compression: CompressionLevel::Balanced,
            strip_level: StripLevel::All,
        }
    }

    pub fn with_compression(mut self, level: CompressionLevel) -> Self {
        self.compression = level;
        self
    }

    pub fn with_strip_level(mut self, level: StripLevel) -> Self {
        self.strip_level = level;
        self
    }

    pub fn compile(&self, script: &str) -> Result<Vec<u8>> {
        // Step 1: Compress script with Zstandard
        let compressed = zstd::encode_all(script.as_bytes(), self.compression.level())
            .map_err(|e| Error::Internal(format!("Compression failed: {e}")))?;

        // Step 2: Load base runtime
        let mut binary = self.load_runtime()?;

        // Step 3: Inject script as ELF section
        let section_offset = self.inject_section(&mut binary, ".rash_script", &compressed)?;

        // Step 4: Patch entrypoint to our loader
        self.patch_entrypoint(binary.as_mut_slice(), section_offset)?;

        // Step 5: Strip unnecessary symbols
        self.strip_binary(binary.as_mut_slice())?;

        Ok(binary)
    }

    fn load_runtime(&self) -> Result<Vec<u8>> {
        let runtime_path = match self.runtime {
            RuntimeType::Dash => "/usr/bin/dash",
            RuntimeType::Busybox => "/bin/busybox",
            RuntimeType::Minimal => {
                return Err(Error::Unsupported(
                    "Minimal runtime not yet implemented".to_string(),
                ));
            }
        };

        // For now, use the system's dash binary
        if !std::path::Path::new(runtime_path).exists() {
            return Err(Error::Internal(format!("{runtime_path} not found")));
        }

        std::fs::read(runtime_path)
            .map_err(|e| Error::Internal(format!("Failed to read runtime: {e}")))
    }

    fn inject_section(&self, binary: &mut Vec<u8>, _name: &str, data: &[u8]) -> Result<usize> {
        // Simplified: append data at the end and return offset
        let offset = binary.len();

        // Add section header
        binary.extend_from_slice(&(data.len() as u32).to_le_bytes());
        binary.extend_from_slice(&(data.len() as u32).to_le_bytes()); // uncompressed size
        binary.extend_from_slice(data);

        Ok(offset)
    }

    fn patch_entrypoint(&self, _binary: &mut [u8], _section_offset: usize) -> Result<()> {
        // TODO: Implement actual ELF patching
        // For now, we'll create a wrapper script
        Ok(())
    }

    fn strip_binary(&self, _binary: &mut [u8]) -> Result<()> {
        match self.strip_level {
            StripLevel::None => Ok(()),
            StripLevel::Debug | StripLevel::All => {
                // TODO: Implement actual stripping
                Ok(())
            }
        }
    }
}

/// Create a self-extracting shell script
pub fn create_self_extracting_script(script: &str, output_path: &str) -> Result<()> {
    let compressed = zstd::encode_all(script.as_bytes(), 11)
        .map_err(|e| Error::Internal(format!("Compression failed: {e}")))?;

    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &compressed);

    let self_extract = format!(
        r#"#!/bin/sh
# Self-extracting RASH script
set -euf

# Embedded compressed script
PAYLOAD='{}'

# Extract and decompress
if command -v base64 >/dev/null && command -v zstd >/dev/null; then
    echo "$PAYLOAD" | base64 -d | zstd -d | sh -euf
elif command -v base64 >/dev/null && command -v gzip >/dev/null; then
    # Fallback to gzip if zstd not available
    echo "Warning: zstd not found, using gzip (larger size)" >&2
    GZIP_PAYLOAD='{}'
    echo "$GZIP_PAYLOAD" | base64 -d | gzip -d | sh -euf
else
    echo "Error: base64 and compression tools required" >&2
    exit 1
fi
"#,
        encoded,
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            compress_gzip(script.as_bytes())
        )
    );

    std::fs::write(output_path, self_extract)
        .map_err(|e| Error::Internal(format!("Failed to write output: {e}")))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(output_path)
            .map_err(|e| Error::Internal(format!("Failed to get metadata: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(output_path, perms)
            .map_err(|e| Error::Internal(format!("Failed to set permissions: {e}")))?;
    }

    Ok(())
}

fn compress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}
