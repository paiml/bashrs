# PMAT-338 — the bashrs v7.4.1 cut

## This diff contains no code, by construction

A release cut bumps the version and writes down what already shipped. Every fix
this release names was reviewed, quorum-agreed and merged on its own PR; the code
is on `main`, not here. A reviewer looking for `keyring_path_from` or the `bwrap`
line in this diff will not find them, and should not: finding them here would
mean the cut had smuggled unreviewed code into a release PR.

Where each claim is backed, by commit on `main`:

| claim in the CHANGELOG | commit on main | PR | what it touched |
|---|---|---|---|
| `bashrs fix` rewrote working scripts into different programs | `c6a804cf01` | #337 | `rash/src/linter/**`, `rash/tests/cli_fix_335.rs` (+56) |
| a missing `cargo-deny` read as a security violation | `90b2be2511` | #334 | `rash/src/cli/gate_commands.rs` (+126/-2) |
| the keyring tests raced on the process environment; `keyring init` wrote nothing | `270d7035fa` | #342 | `rash/src/cli/installer_commands.rs`, `installer_run_logic.rs`, `rash/src/installer/signature.rs`, 13 tests |
| `corpus-score`'s sandbox masked the repository | `0fb9efa113` | #343 | `Makefile:337`, `docs/audits/impl-PMAT-340-receipt.md` |

`git log v7.4.0..origin/main --oneline` returns exactly those four plus the
v7.4.0 close-out, which is the same set the `[7.4.1]` CHANGELOG section
describes and no others.

## What this diff DOES carry

- `Cargo.toml` / `Cargo.lock`: 7.4.0 → 7.4.1
- `CHANGELOG.md`: the `[7.4.1]` section, four entries
- `docs/release-notes.md`, `docs/roadmaps/releases.md`: the same four, for the
  two audiences those files have
- `book/src/reference/cli.md`: what a fix may not change — the user-visible half
  of #335, which is why `check-book-updated.sh` passes rather than being skipped
- `docs/roadmaps/roadmap.yaml`: the rows

`docs/audits/quorum-PMAT-335.json` was in the first draft of this cut and is
dropped: it is #337's review artifact and belongs to that PR.

## The gate

`make release-gate`, run from a worktree under `/tmp`, exit 0:

```
✅ Release gate green.
```

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -D
warnings`, `cargo test --workspace`, `pv lint contracts`, `coverage-gate`,
`dogfood-selflint`, `corpus-score`, `check-book-updated.sh`.

`corpus-score`: **V2 Corpus Score 99.4/100 (A+)** — bash 16970/16970, makefile
1049/1049, dockerfile 795/795; determinism 18814/18814, lint-clean 18811/18814,
cross-shell 18758/18814.

**This is the first release cut whose gate ran end to end from a worktree.**
Until `0fb9efa113` it could not: `bwrap` mounted a tmpfs over the whole of `/tmp`
and then executed `$(PWD)/target/debug/bashrs`, so from any checkout under `/tmp`
the gate died at `bwrap: execvp …/target/debug/bashrs: No such file or directory`
before scoring a single entry. The fix re-exposes `$(PWD)` READ-ONLY, which is
what `--ro-bind / /` already granted everywhere outside `/tmp`.

## Disclosed

**A lane refused an earlier round of this PR for exactly the reason above** — it
read the CHANGELOG's claims about `keyring_path_from`, `linter::quoted_segments`
and the `bwrap` line, found no such code in the diff, and applied the rule that
an unbacked receipt claim is a FAIL. The rule is right and the application was a
category error, but the gap it found was real: the cut carried no document
saying where its claims are backed. This receipt is that document, and the table
above is what a lane needs in order to check the claims instead of refusing them.

**No falsification test is added by this diff and none is owed.** Each of the
four fixes carries its own, on its own branch: `rash/tests/cli_fix_335.rs` runs
the issue's script under bash before and after `bashrs fix` and requires the same
output; the keyring change replaced 13 environment-writing tests with 13 that
pass the path in (61/150 failures before, 0/150 after); PMAT-340's is the RED
recorded in `docs/audits/impl-PMAT-340-receipt.md`, the `execvp` failure itself.

IMPL-PMAT-338-RECEIPT-END
