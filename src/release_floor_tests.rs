//! Shape tests for the glibc floor in `.github/workflows/binary-release.yml`
//! (#527). They read the workflow text; they cannot prove the tools exist on
//! the runners or the real symbols of a built binary (only a release run can).

const WF: &str = include_str!("../.github/workflows/binary-release.yml");

/// Text of the step whose `- name:` line starts with `prefix`.
fn step(prefix: &str) -> &'static str {
    let marker = format!("      - name: {prefix}");
    let start = WF
        .find(&marker)
        .unwrap_or_else(|| panic!("no step named `{prefix}...`"));
    let rest = &WF[start + 1..];
    let len = rest.find("\n      - name: ").map_or(rest.len(), |i| i + 1);
    &WF[start..start + 1 + len]
}

fn line_with<'a>(text: &'a str, needle: &str) -> &'a str {
    text.lines()
        .find(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no line containing `{needle}`"))
}

#[test]
fn gnu_build_uses_zigbuild_with_2_31_suffix() {
    let l = line_with(step("Build bashrs (gnu"), "cargo zigbuild");
    assert!(l.contains(".2.31"), "gnu build line lacks .2.31: {l}");
    assert!(l.contains("-p bashrs --bin bashrs"), "wrong package: {l}");
}

#[test]
fn gnu_build_is_locked() {
    let l = line_with(step("Build bashrs (gnu"), "cargo zigbuild");
    assert!(l.contains("--locked"), "gnu build line lacks --locked: {l}");
}

#[test]
fn floor_step_reads_symbols_and_is_gnu_gated() {
    let s = step("Glibc floor");
    assert!(
        line_with(s, "if:").contains("-gnu"),
        "floor step not gnu-gated"
    );
    assert!(
        s.contains("objdump -T"),
        "floor step does not run objdump -T"
    );
    assert!(
        s.contains("GLIBC_"),
        "floor step does not read GLIBC_ symbols"
    );
    assert!(
        s.contains("sort -V"),
        "floor compare must be numeric (sort -V)"
    );
    assert!(
        s.contains("GLIBC_ABI_"),
        "floor step must refuse GLIBC_ABI_ needs"
    );
}

#[test]
fn floor_literal_is_exactly_2_31() {
    let floors: Vec<&str> = step("Glibc floor")
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("FLOOR="))
        .collect();
    assert_eq!(floors, ["FLOOR=2.31"]);
}

#[test]
fn floor_step_fails_closed_and_runs_binary() {
    let s = step("Glibc floor");
    assert!(s.contains("exit 1"), "floor step has no `exit 1`");
    assert!(s.contains("--version"), "floor step does not run --version");
}

#[test]
fn workflow_has_no_soft_failure() {
    assert!(
        !WF.contains("continue-on-error"),
        "continue-on-error present"
    );
    assert!(!WF.contains("|| true"), "`|| true` present");
}

#[test]
fn tool_step_names_each_required_tool_and_is_gnu_gated() {
    let s = step("Require build tools");
    assert!(
        line_with(s, "if:").contains("-gnu"),
        "tool step not gnu-gated"
    );
    for tool in ["zig", "cargo-zigbuild", "objdump"] {
        assert!(s.contains(tool), "tool step does not check {tool}");
    }
    assert!(s.contains("exit 1"), "tool step has no `exit 1`");
}
