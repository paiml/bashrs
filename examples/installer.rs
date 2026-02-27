fn main() {
    let prefix = "/usr/local";
    let version = "1.0.0";

    echo("Installing version");
    echo(version);

    mkdir(prefix);

    let install_path = concat(prefix, "/bin/tool");
    touch(install_path);
}

fn echo(msg: &str) {}
fn mkdir(path: &str) {}
fn touch(path: &str) {}
fn concat(a: &str, b: &str) -> &str {
    a
}
