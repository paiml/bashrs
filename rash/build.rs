fn main() {
    // Allow kani cfg for verification
    println!("cargo::rustc-check-cfg=cfg(kani)");
}
