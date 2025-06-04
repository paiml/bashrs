#[rash::main]
fn install() -> Result<(), &'static str> {
    let message = "Hello from Rash!";
    echo(message);
    Ok(())
}

fn echo(msg: &str) {
    // This will be converted to shell echo command
}