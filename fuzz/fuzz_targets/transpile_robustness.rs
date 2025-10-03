#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to UTF-8 string
    if let Ok(source) = std::str::from_utf8(data) {
        // Transpilation must never panic, regardless of input
        // This tests error handling robustness
        let _ = bashrs::transpile(source, bashrs::Config::default());
    }
});
