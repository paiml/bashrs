// Chapter 3, Example 5: Function Calling Another Function
// Function composition - the foundation of structured scripts

fn main() {
    install_app("myapp", "1.0.0");
}

fn install_app(name: &str, version: &str) {
    download(name, version);
    extract(name);
    configure_app(name);
}

fn download(n: &str, v: &str) {
    echo("Downloading...");
}

fn extract(n: &str) {
    echo("Extracting...");
}

fn configure_app(n: &str) {
    echo("Configuring...");
}

fn echo(msg: &str) {}
