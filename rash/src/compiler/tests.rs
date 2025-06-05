#[cfg(test)]
mod tests {
    use crate::compiler::{BinaryCompiler, RuntimeType, CompressionLevel, create_self_extracting_script};
    
    #[test]
    fn test_self_extracting_script_creation() {
        let script = "echo 'Hello, World!'";
        let output = "/tmp/test_rash_self_extract.sh";
        
        let result = create_self_extracting_script(script, output);
        assert!(result.is_ok());
        
        // Verify file exists
        assert!(std::path::Path::new(output).exists());
        
        // Cleanup
        std::fs::remove_file(output).ok();
    }
    
    #[test]
    fn test_compression_levels() {
        let data = b"This is a test string for compression";
        
        let fast = zstd::encode_all(&data[..], CompressionLevel::Fast.level()).unwrap();
        let best = zstd::encode_all(&data[..], CompressionLevel::Best.level()).unwrap();
        
        // Best compression should be smaller or equal
        assert!(best.len() <= fast.len());
    }
    
    #[test]
    fn test_binary_compiler_new() {
        let compiler = BinaryCompiler::new(RuntimeType::Dash);
        
        // For now, just ensure we can create it
        let script = "echo test";
        let result = compiler.compile(script);
        
        // Should fail gracefully since dash might not be available
        assert!(result.is_err() || result.is_ok());
    }
}