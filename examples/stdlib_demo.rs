// Rash Standard Library Demo
// Demonstrates the stdlib functions available in v0.9.0

fn main() {
    // String operations
    demo_string_functions();

    // File system operations
    demo_file_operations();

    // Combined example
    demo_combined();
}

fn demo_string_functions() {
    echo("=== String Functions Demo ===");

    // string_trim: Remove whitespace
    let text = "  hello world  ";
    let trimmed = string_trim(text);
    echo(trimmed); // Outputs: "hello world"

    // string_contains: Check for substring
    if string_contains("hello world", "world") {
        echo("Contains 'world'!");
    }

    // string_len: Get length
    let length = string_len("hello");
    echo(length); // Outputs: "5"
}

fn demo_file_operations() {
    echo("=== File System Functions Demo ===");

    // fs_exists: Check if file exists
    if fs_exists("/etc/passwd") {
        echo("File exists!");
    }

    // fs_write_file: Write to file
    fs_write_file("/tmp/rash_demo.txt", "Hello from Rash!");

    // fs_read_file: Read from file
    let content = fs_read_file("/tmp/rash_demo.txt");
    echo(content);
}

fn demo_combined() {
    echo("=== Combined Example ===");

    // Read, trim, check, and write
    let data = "  important data  ";
    let cleaned = string_trim(data);

    if string_contains(cleaned, "important") {
        fs_write_file("/tmp/cleaned.txt", cleaned);
        echo("Data cleaned and saved!");
    }
}
