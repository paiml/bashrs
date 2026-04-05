fn test_t091_file_read() {
    // STMT: let _ = std::fs::read_to_string("f"); - file read
    let (ok, output) = transpile_stmt(r#"let _ = std::fs::read_to_string("f");"#);
    if ok && !output.contains("cat") && !output.contains("<") {
        println!("T091: WARNING - File read should produce cat or <");
    }
}

#[test]
fn test_t092_file_write() {
    // STMT: std::fs::write("f", "x"); - file write
    let (ok, output) = transpile_stmt(r#"std::fs::write("f", "x");"#);
    if ok && !output.contains(">") && !output.contains("echo") {
        println!("T092: WARNING - File write should produce > redirect");
    }
}

#[test]
fn test_t093_env_get() {
    // STMT: let _ = std::env::var("X"); - env get
    let (ok, output) = transpile_stmt(r#"let _ = std::env::var("X");"#);
    if ok && !output.contains("$") && !output.contains("X") {
        println!("T093: WARNING - Env get should produce $X or similar");
    }
}

#[test]
fn test_t094_env_set() {
    // STMT: std::env::set_var("X", "v"); - env set
    let (ok, output) = transpile_stmt(r#"std::env::set_var("X", "v");"#);
    if ok && !output.contains("export") && !output.contains("X=") {
        println!("T094: WARNING - Env set should produce export or X=");
    }
}

#[test]
fn test_t095_process_exit() {
    // STMT: std::process::exit(0); - should produce exit
    let (ok, output) = transpile_stmt("std::process::exit(0);");
    if ok && !output.contains("exit") {
        println!("T095: WARNING - exit() should produce shell exit");
    }
}

#[test]
fn test_t096_remove_file() {
    // STMT: std::fs::remove_file("f"); - delete file
    let (ok, output) = transpile_stmt(r#"std::fs::remove_file("f");"#);
    if ok && !output.contains("rm") {
        println!("T096: WARNING - remove_file should produce rm");
    }
}

#[test]
fn test_t097_create_dir() {
    // STMT: std::fs::create_dir("d"); - mkdir
    let (ok, output) = transpile_stmt(r#"std::fs::create_dir("d");"#);
    if ok && !output.contains("mkdir") {
        println!("T097: WARNING - create_dir should produce mkdir");
    }
}

#[test]
fn test_t098_path_new() {
    // STMT: std::path::Path::new("p"); - path creation
    let (ok, output) = transpile_stmt(r#"let _ = std::path::Path::new("p");"#);
    if ok {
        // Path is just a wrapper, should produce string
        if !output.contains("p") {
            println!("T098: WARNING - Path should preserve string");
        }
    }
}

#[test]
fn test_t099_sleep() {
    // STMT: std::thread::sleep(std::time::Duration::from_secs(1)); - sleep
    let (ok, output) = transpile_stmt("std::thread::sleep(std::time::Duration::from_secs(1));");
    if ok && !output.contains("sleep") {
        println!("T099: WARNING - sleep should produce shell sleep");
    }
}

#[test]
fn test_t100_command() {
    // STMT: std::process::Command::new("ls"); - subprocess
    let (ok, output) = transpile_stmt(r#"let _ = std::process::Command::new("ls");"#);
    if ok && !output.contains("ls") {
        println!("T100: WARNING - Command should produce shell command");
    }
}

#[test]
fn test_t101_instant() {
    // STMT: std::time::Instant::now(); - timing
    let (ok, output) = transpile_stmt("let _ = std::time::Instant::now();");
    if ok && !output.contains("date") && !output.contains("$(") {
        println!("T101: WARNING - Instant::now should produce date +%s or similar");
    }
}

#[test]
fn test_t102_stdin() {
    // STMT: std::io::stdin(); - stdin access
    let (ok, output) = transpile_stmt("let _ = std::io::stdin();");
    if ok && !output.contains("read") && !output.contains("stdin") {
        println!("T102: WARNING - stdin should produce read or stdin reference");
    }
}

#[test]
fn test_t103_stdout() {
    // STMT: std::io::stdout(); - stdout access
    let (ok, output) = transpile_stmt("let _ = std::io::stdout();");
    if ok && !output.contains("stdout") && !output.contains("/dev/stdout") {
        println!("T103: INFO - stdout access may be implicit");
    }
}

#[test]
fn test_t104_cli_args() {
    // STMT: std::env::args(); - CLI arguments
    let (ok, output) = transpile_stmt("let _ = std::env::args();");
    if ok && !output.contains("$@") && !output.contains("$*") {
        println!("T104: WARNING - args() should produce $@ or $*");
    }
}

#[test]
fn test_t105_current_dir() {
    // STMT: std::env::current_dir(); - CWD
    let (ok, output) = transpile_stmt("let _ = std::env::current_dir();");
    if ok && !output.contains("pwd") && !output.contains("PWD") {
        println!("T105: WARNING - current_dir should produce pwd");
    }
}

// ============================================================================
// SECTION 4.7: Advanced & Error Handling (T106-T120)
// ============================================================================

#[test]
fn test_t106_option_some() {
    // STMT: let _ = Option::Some(1); - Option wrap
    let (ok, output) = transpile_stmt("let _ = Option::Some(1);");
    if !ok {
        println!("T106: Option::Some unsupported: {}", output);
    }
}

#[test]
fn test_t107_option_none() {
    // STMT: let _ = Option::<i32>::None; - Option none
    let (ok, output) = transpile_stmt("let _ = Option::<i32>::None;");
    if !ok {
        println!("T107: Option::None unsupported: {}", output);
    }
}

#[test]
fn test_t108_result_ok() {
    // STMT: let _ = Result::<i32, &str>::Ok(1); - Result ok
    let (ok, output) = transpile_stmt("let _ = Result::<i32, &str>::Ok(1);");
    if !ok {
        println!("T108: Result::Ok unsupported: {}", output);
    }
}

#[test]
fn test_t109_result_err() {
    // STMT: let _ = Result::<i32, &str>::Err("e"); - Result err
    let (ok, output) = transpile_stmt(r#"let _ = Result::<i32, &str>::Err("e");"#);
    if !ok {
        println!("T109: Result::Err unsupported: {}", output);
    }
}

#[test]
fn test_t110_unwrap() {
    // STMT: let _ = opt.unwrap(); - unwrap
    let (ok, output) = transpile_stmt("let opt = Some(1); let _ = opt.unwrap();");
    if !ok {
        println!("T110: unwrap unsupported: {}", output);
    }
}

#[test]
fn test_t111_expect() {
    // STMT: let _ = opt.expect("msg"); - expect
    let (ok, output) = transpile_stmt(r#"let opt = Some(1); let _ = opt.expect("msg");"#);
    if !ok {
        println!("T111: expect unsupported: {}", output);
    }
}

#[test]
fn test_t112_try_operator() {
    // PROG: fn f() -> Option<i32> { Some(1)? } - try operator
    let code = "fn f() -> Option<i32> { let x = Some(1)?; Some(x) } fn main() {}";
    let (ok, output) = transpile_prog(code);
    if !ok {
        println!("T112: Try operator (?) unsupported: {}", output);
    }
}

#[test]
fn test_t113_panic() {
    // STMT: panic!("msg"); - panic
    let (ok, output) = transpile_stmt(r#"panic!("msg");"#);
    if ok && !output.contains("exit") && !output.contains("1") {
        println!("T113: WARNING - panic should produce exit 1");
    }
}

#[test]
fn test_t114_assert() {
    // STMT: assert!(x == 10); - assert
    let (ok, output) = transpile_stmt("assert!(x == 10);");
    if ok && !output.contains("if") && !output.contains("[") && !output.contains("exit") {
        println!("T114: WARNING - assert should produce condition check");
    }
}

#[test]
fn test_t115_assert_eq() {
    // STMT: assert_eq!(x, 10); - assert_eq
    let (ok, output) = transpile_stmt("assert_eq!(x, 10);");
    if ok && !output.contains("if") && !output.contains("[") && !output.contains("exit") {
        println!("T115: WARNING - assert_eq should produce equality check");
    }
}

#[test]
fn test_t116_vec_macro() {
    // STMT: let _ = vec![1, 2, 3]; - vec macro
    let (ok, output) = transpile_stmt("let _ = vec![1, 2, 3];");
    if !ok {
        println!("T116: vec! macro unsupported: {}", output);
    }
}

#[test]
fn test_t117_vec_push() {
    // STMT: let mut v = vec![]; v.push(1); - vec push
    let (ok, output) = transpile_stmt("let mut v = vec![]; v.push(1);");
    if !ok {
        println!("T117: vec push unsupported: {}", output);
    }
}

#[test]

include!("transpiler_tcode_tests_tests_cont.rs");
