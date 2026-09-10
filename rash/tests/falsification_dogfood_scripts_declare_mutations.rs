//! Falsification: every dogfood gate script declares its own mutation.
//!
//! PMAT-255 (PMAT-253 phase 3a). Each hermetic gate under `scripts/dogfood/`
//! (D `docs.sh`, G `contracts.sh`, K `corpus.sh`, and any future sibling) is
//! required to carry a `# mutation: <sed expression>` line: a single,
//! machine-applicable transformation that, applied to a COPY of the script,
//! turns that gate's own verdict red. A gate with no declared mutation could
//! be vacuous — always PASS, checking nothing — and nothing else in this
//! repository would notice. `scripts/dogfood/lib/` is excluded: it is
//! SOURCED, never RUN standalone (see `lib/window.sh`'s own header), so it
//! has no gate verdict of its own to falsify.
//!
//! This test only checks the DECLARATION exists. Whether the declared
//! mutation actually turns the gate red is checked by
//! `/tmp/.../scratchpad/a-dogfood.sh` (the acceptance script for PMAT-255),
//! which applies each mutation to a copy and runs it — that check needs a
//! built release binary and, for gate K, bwrap and ~140s per corpus run, none
//! of which belong in `cargo test`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root, found from this test binary's own manifest dir
/// (`rash/`) rather than the process's current directory — `cargo test` sets
/// cwd to the crate root, but a caller that `cd`s first must not change what
/// this test inspects.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rash/ has a parent directory")
        .to_path_buf()
}

/// Every `scripts/dogfood/*.sh` outside `lib/`, sorted for a stable failure
/// message. Empty is refused below — a rule that ran over zero files would
/// vacuously "pass" the day the directory was renamed out from under it.
fn dogfood_gate_scripts() -> Vec<PathBuf> {
    let dogfood_dir = repo_root().join("scripts").join("dogfood");
    let mut scripts: Vec<PathBuf> = fs::read_dir(&dogfood_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dogfood_dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sh"))
        .collect();
    scripts.sort();
    scripts
}

#[test]
fn test_dogfood_scripts_directory_is_not_empty() {
    let scripts = dogfood_gate_scripts();
    assert!(
        !scripts.is_empty(),
        "scripts/dogfood/*.sh holds no gate script outside lib/ — this test would \
         otherwise pass vacuously over an empty directory"
    );
}

#[test]
fn test_every_dogfood_gate_script_declares_a_mutation() {
    let scripts = dogfood_gate_scripts();
    assert!(
        !scripts.is_empty(),
        "no dogfood gate scripts found to check"
    );

    let mut missing = Vec::new();
    for script in &scripts {
        let text = fs::read_to_string(script)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", script.display()));
        let declares_mutation = text
            .lines()
            .any(|line| line.trim_start().starts_with("# mutation: "));
        if !declares_mutation {
            missing.push(script.display().to_string());
        }
    }

    assert!(
        missing.is_empty(),
        "the following scripts/dogfood/*.sh script(s) lack a `# mutation: <sed expression>` \
         line and could therefore be vacuous gates: {missing:?}"
    );
}

#[test]
fn test_every_declared_mutation_actually_changes_the_file() {
    // A weaker, cheap-to-run companion to the full red-check in
    // a-dogfood.sh: the sed expression a script declares must at least
    // transform SOME line of that script, or the "mutation" is a no-op that
    // could never turn anything red no matter what a-dogfood.sh measures.
    let scripts = dogfood_gate_scripts();
    assert!(
        !scripts.is_empty(),
        "no dogfood gate scripts found to check"
    );

    for script in &scripts {
        let text = fs::read_to_string(script)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", script.display()));
        let mutation_line = text
            .lines()
            .find(|line| line.trim_start().starts_with("# mutation: "))
            .unwrap_or_else(|| panic!("{} declares no mutation", script.display()));
        let sed_expr = mutation_line
            .trim_start()
            .strip_prefix("# mutation: ")
            .expect("prefix just matched")
            .trim();
        assert!(
            !sed_expr.is_empty(),
            "{} declares an empty mutation expression",
            script.display()
        );

        let mutated = run_sed(sed_expr, &text);
        assert_ne!(
            mutated,
            text,
            "{}'s declared mutation `{sed_expr}` changes nothing when applied to the file — \
             a no-op mutation can never turn the gate red",
            script.display()
        );
    }
}

/// Apply a sed expression to `input` via the real `sed` binary (not
/// reimplemented here) and return stdout. Panics loudly on any I/O or
/// process failure — this helper has no gate verdict of its own to protect,
/// so a `sed` that cannot run is a test failure, not a skipped test.
fn run_sed(expr: &str, input: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("sed")
        .arg("-e")
        .arg(expr)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn sed");
    child
        .stdin
        .as_mut()
        .expect("sed stdin is piped")
        .write_all(input.as_bytes())
        .expect("failed to write to sed stdin");
    let output = child.wait_with_output().expect("failed to wait on sed");
    assert!(
        output.status.success(),
        "sed -e {expr:?} exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("sed produced non-UTF-8 output")
}
