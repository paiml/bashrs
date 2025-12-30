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

#[cfg(test)]
mod tests {
    use super::*;

    // ===== CompressionLevel Tests =====

    #[test]
    fn test_compression_level_fast() {
        assert_eq!(CompressionLevel::Fast.level(), 3);
    }

    #[test]
    fn test_compression_level_balanced() {
        assert_eq!(CompressionLevel::Balanced.level(), 11);
    }

    #[test]
    fn test_compression_level_best() {
        assert_eq!(CompressionLevel::Best.level(), 19);
    }

    // ===== RuntimeType Tests =====

    #[test]
    fn test_runtime_type_debug() {
        let dash = RuntimeType::Dash;
        let busybox = RuntimeType::Busybox;
        let minimal = RuntimeType::Minimal;

        // Just test that Debug trait is implemented
        let _ = format!("{:?}", dash);
        let _ = format!("{:?}", busybox);
        let _ = format!("{:?}", minimal);
    }

    #[test]
    fn test_runtime_type_clone() {
        let dash = RuntimeType::Dash;
        let cloned = dash;
        assert!(matches!(cloned, RuntimeType::Dash));
    }

    // ===== StripLevel Tests =====

    #[test]
    fn test_strip_level_debug() {
        let none = StripLevel::None;
        let debug = StripLevel::Debug;
        let all = StripLevel::All;

        let _ = format!("{:?}", none);
        let _ = format!("{:?}", debug);
        let _ = format!("{:?}", all);
    }

    // ===== BinaryCompiler Tests =====

    #[test]
    fn test_binary_compiler_new() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash);
        assert!(matches!(compiler.runtime, RuntimeType::Dash));
        assert!(matches!(compiler.compression, CompressionLevel::Balanced));
        assert!(matches!(compiler.strip_level, StripLevel::All));
    }

    #[test]
    fn test_binary_compiler_with_compression() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash)
            .with_compression(CompressionLevel::Best);
        assert!(matches!(compiler.compression, CompressionLevel::Best));
    }

    #[test]
    fn test_binary_compiler_with_strip_level() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash)
            .with_strip_level(StripLevel::None);
        assert!(matches!(compiler.strip_level, StripLevel::None));
    }

    #[test]
    fn test_binary_compiler_builder_chain() {
        let compiler = BinaryCompiler::new(RuntimeType::Busybox)
            .with_compression(CompressionLevel::Fast)
            .with_strip_level(StripLevel::Debug);

        assert!(matches!(compiler.runtime, RuntimeType::Busybox));
        assert!(matches!(compiler.compression, CompressionLevel::Fast));
        assert!(matches!(compiler.strip_level, StripLevel::Debug));
    }

    #[test]
    fn test_binary_compiler_minimal_runtime_error() {
        let compiler = BinaryCompiler::new(RuntimeType::Minimal);
        let result = compiler.compile("echo hello");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Minimal runtime not yet implemented"));
    }

    #[test]
    fn test_inject_section() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash);
        let mut binary = vec![0u8; 100];
        let data = b"test data";

        let offset = compiler.inject_section(&mut binary, ".test", data).unwrap();
        assert_eq!(offset, 100); // Should append at the end

        // Binary should be extended with header (8 bytes) + data
        assert_eq!(binary.len(), 100 + 8 + data.len());
    }

    #[test]
    fn test_patch_entrypoint() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash);
        let mut binary = vec![0u8; 100];

        let result = compiler.patch_entrypoint(&mut binary, 50);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strip_binary_none() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash)
            .with_strip_level(StripLevel::None);
        let mut binary = vec![0u8; 100];

        let result = compiler.strip_binary(&mut binary);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strip_binary_debug() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash)
            .with_strip_level(StripLevel::Debug);
        let mut binary = vec![0u8; 100];

        let result = compiler.strip_binary(&mut binary);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strip_binary_all() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash)
            .with_strip_level(StripLevel::All);
        let mut binary = vec![0u8; 100];

        let result = compiler.strip_binary(&mut binary);
        assert!(result.is_ok());
    }

    // ===== compress_gzip Tests =====

    #[test]
    fn test_compress_gzip_empty() {
        let result = compress_gzip(&[]);
        // Gzip header is at least 10 bytes
        assert!(result.len() >= 10);
    }

    #[test]
    fn test_compress_gzip_small_data() {
        let data = b"hello world";
        let compressed = compress_gzip(data);
        // Compressed data should exist
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_compress_gzip_large_data() {
        let data = vec![b'A'; 10000];
        let compressed = compress_gzip(&data);
        // Highly repetitive data should compress well
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_gzip_decompresses_correctly() {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let original = b"test data for compression";
        let compressed = compress_gzip(original);

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        assert_eq!(decompressed, original);
    }
}
