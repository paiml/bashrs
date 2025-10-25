//! I/O Streams for WASM Bash Runtime
//!
//! Provides stdout/stderr capture for bash command execution in WASM.
//!
//! # Example
//!
//! ```rust
//! use bashrs::wasm::io::IoStreams;
//!
//! let mut io = IoStreams::new_capture();
//! io.stdout.write_all(b"Hello, World!\n").unwrap();
//! assert_eq!(io.get_stdout(), "Hello, World!\n");
//! ```

use std::io::{self, Write};

use std::sync::{Arc, Mutex};

/// I/O streams for bash execution
pub struct IoStreams {
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
    stdout_capture: Arc<Mutex<CaptureWriter>>,
    stderr_capture: Arc<Mutex<CaptureWriter>>,
    stdin: Arc<Mutex<String>>,
}

impl IoStreams {
    /// Create new I/O streams with capture buffers
    pub fn new_capture() -> Self {
        let stdout_capture = Arc::new(Mutex::new(CaptureWriter::new()));
        let stderr_capture = Arc::new(Mutex::new(CaptureWriter::new()));
        let stdin = Arc::new(Mutex::new(String::new()));

        Self {
            stdout: Box::new(SharedWriter(stdout_capture.clone())),
            stderr: Box::new(SharedWriter(stderr_capture.clone())),
            stdout_capture,
            stderr_capture,
            stdin,
        }
    }

    /// Get captured stdout as UTF-8 string
    pub fn get_stdout(&self) -> String {
        self.stdout_capture.lock().unwrap().as_string()
    }

    /// Get captured stderr as UTF-8 string
    pub fn get_stderr(&self) -> String {
        self.stderr_capture.lock().unwrap().as_string()
    }

    /// Get stdin content
    pub fn get_stdin(&self) -> String {
        self.stdin.lock().unwrap().clone()
    }

    /// Set stdin content (for pipelines)
    pub fn set_stdin(&mut self, content: &str) {
        *self.stdin.lock().unwrap() = content.to_string();
    }

    /// Clear stdin
    pub fn clear_stdin(&mut self) {
        self.stdin.lock().unwrap().clear();
    }
}

impl Clone for IoStreams {
    fn clone(&self) -> Self {
        Self {
            stdout: Box::new(SharedWriter(self.stdout_capture.clone())),
            stderr: Box::new(SharedWriter(self.stderr_capture.clone())),
            stdout_capture: self.stdout_capture.clone(),
            stderr_capture: self.stderr_capture.clone(),
            stdin: self.stdin.clone(),
        }
    }
}

/// Shared writer that wraps Arc<Mutex<CaptureWriter>>
struct SharedWriter(Arc<Mutex<CaptureWriter>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

/// Writer that captures output to an in-memory buffer
pub struct CaptureWriter {
    buffer: Vec<u8>,
}

impl CaptureWriter {
    /// Create new capture writer
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Get captured content as UTF-8 string
    pub fn as_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Nothing to flush for in-memory buffer
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_stdout() {
        // ARRANGE
        let mut io = IoStreams::new_capture();

        // ACT
        io.stdout.write_all(b"hello\n").unwrap();

        // ASSERT
        assert_eq!(io.get_stdout(), "hello\n");
    }

    #[test]
    fn test_capture_stderr() {
        // ARRANGE
        let mut io = IoStreams::new_capture();

        // ACT
        io.stderr.write_all(b"error\n").unwrap();

        // ASSERT
        assert_eq!(io.get_stderr(), "error\n");
    }

    #[test]
    fn test_capture_both_streams() {
        // ARRANGE
        let mut io = IoStreams::new_capture();

        // ACT
        io.stdout.write_all(b"output\n").unwrap();
        io.stderr.write_all(b"error\n").unwrap();

        // ASSERT
        assert_eq!(io.get_stdout(), "output\n");
        assert_eq!(io.get_stderr(), "error\n");
    }

    #[test]
    fn test_capture_writer_basic() {
        // ARRANGE
        let mut writer = CaptureWriter::new();

        // ACT
        writer.write_all(b"test").unwrap();

        // ASSERT
        assert_eq!(writer.as_string(), "test");
    }

    #[test]
    fn test_capture_writer_multiple_writes() {
        // ARRANGE
        let mut writer = CaptureWriter::new();

        // ACT
        writer.write_all(b"hello ").unwrap();
        writer.write_all(b"world").unwrap();

        // ASSERT
        assert_eq!(writer.as_string(), "hello world");
    }

    #[test]
    fn test_capture_writer_utf8() {
        // ARRANGE
        let mut writer = CaptureWriter::new();

        // ACT - UTF-8 with emoji
        writer.write_all("Hello 🚀 World".as_bytes()).unwrap();

        // ASSERT
        assert_eq!(writer.as_string(), "Hello 🚀 World");
    }
}
