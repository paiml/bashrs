// Chapter 3, Example 11: Variadic-Style Pattern (Fixed Parameters)
// Simulating variadic functions with fixed parameter count

fn main() {
    install_packages("pkg1", "pkg2", "pkg3", "pkg4", "pkg5");
}

fn install_packages(p1: &str, p2: &str, p3: &str, p4: &str, p5: &str) {
    install_one(p1);
    install_one(p2);
    install_one(p3);
    install_one(p4);
    install_one(p5);
}

fn install_one(pkg: &str) {
    echo(pkg);
}

fn echo(msg: &str) {}
