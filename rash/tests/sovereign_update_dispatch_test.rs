//! `bashrs update` must be dispatched to sovereign_update before the clap parser
//! (#377). `update --help` is handled entirely by `update_main` and never
//! touches the network, so its usage text proves the guard ran first.

use std::process::Command;

#[test]
fn update_dispatch_is_reached_before_cli_parser() {
    let out = Command::new(env!("CARGO_BIN_EXE_bashrs"))
        .args(["update", "--help"])
        .env("SOVEREIGN_DISABLE_UPDATE_CHECK", "1")
        .output()
        .expect("run bashrs update --help");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("usage: bashrs update [--check]"),
        "update dispatch not reached before parser; stdout was: {stdout}"
    );
    assert_eq!(out.status.code(), Some(0));
}
