// Test SC2164: Use cd ... || exit in case cd fails


fn main() {
    let dir1 = "/tmp";
    let dir2 = "/nonexistent/path";
    let dir3 = "/var/log";
    let dir4 = "/home/user";
    
    cd_safe(dir1);
    cd_safe(dir2);
    cd_safe(dir3);
    cd_safe(dir4);
}

fn cd_safe(dir: &str) {
    // Change directory with error handling
}