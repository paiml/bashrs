//! GH-438: SC2168/SC2151/SC2152 must key on COMMAND position, not the word anywhere.
use crate::linter::rules::{sc2151, sc2152, sc2168};

#[test]
fn gh438_sc2168_prose_forms_are_silent() {
    for src in [
        "\t@# touches the local registry",
        "\t-# touches the local registry",
        "\t+# touches the local registry",
        "\t@-# touches the local registry",
        "# the local registry",
        "\t: touches the local docker registry, which; \\",
        "\techo local x",
        "\techo \"a\" 'b local x'",
    ] {
        assert_eq!(sc2168::check(src).diagnostics.len(), 0, "{src:?}");
    }
}

#[test]
fn gh438_sc2168_real_local_still_reports() {
    for src in [
        "local x=1",
        "\tlocal x=1",
        "\t@local x=1",
        "true; local y=2",
    ] {
        assert_eq!(sc2168::check(src).diagnostics.len(), 1, "{src:?}");
    }
}

#[test]
fn gh438_sc2151_sc2152_args_are_silent_commands_report() {
    assert_eq!(sc2151::check("echo return 256").diagnostics.len(), 0);
    assert_eq!(sc2151::check("\t@# return 256").diagnostics.len(), 0);
    assert_eq!(sc2151::check("return 256").diagnostics.len(), 1);
    assert_eq!(
        sc2151::check("[ -z \"$x\" ] && return 300")
            .diagnostics
            .len(),
        1
    );
    assert_eq!(sc2152::check("echo exit 999").diagnostics.len(), 0);
    assert_eq!(sc2152::check("\t@# exit 999").diagnostics.len(), 0);
    assert_eq!(sc2152::check("exit 999").diagnostics.len(), 1);
    assert_eq!(sc2152::check("\t@exit -1").diagnostics.len(), 1);
}
