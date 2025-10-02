mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Rash MCP Server v0.1.0");
    println!("Rust-to-Shell transpiler as MCP tool");
    println!();
    println!("Available tools:");
    println!("  transpile(source, optimize?, strict?) - Transpile Rust code to POSIX shell");
    println!();
    println!("Example:");
    println!(r#"  {{ "source": "fn main() {{ let x = 42; }}" }}"#);
    println!();
    println!("Ready for MCP client connections!");

    // Demonstrate handler works
    let handler = handlers::transpile::TranspileHandler;
    let input = handlers::transpile::TranspileInput {
        source: r#"
            fn main() {
                println!("Hello from Rash MCP!");
            }
        "#
        .to_string(),
        optimize: false,
        strict: false,
    };

    match pforge_runtime::Handler::handle(&handler, input).await {
        Ok(result) => {
            println!("Test transpilation successful!");
            println!("Generated shell script:");
            println!("{}", "=".repeat(60));
            println!("{}", result.shell_script);
            println!("{}", "=".repeat(60));
        }
        Err(e) => {
            eprintln!("Test transpilation failed: {}", e);
        }
    }

    // In production, you would:
    // 1. Create HandlerRegistry and register TranspileHandler
    // 2. Create McpServer with config
    // 3. Run server.run().await for stdio transport

    Ok(())
}
