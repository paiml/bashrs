# PMAT-340 — corpus-score's sandbox masked the repository it was told to run from

`make release-gate` calls `corpus-score`, which runs the corpus through `bwrap`.
The sandbox mounted `--tmpfs /tmp` over the whole of `/tmp` and then executed
`$(PWD)/target/debug/bashrs`. For a checkout under `/tmp` — every worktree this
fleet builds in — that binary is behind the tmpfs, so the gate could not run at
all from a worktree.

## RED

From a worktree under `/tmp`, main's command:

```
bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp --bind $E $E --chdir $E \
      --unshare-net --die-with-parent $(PWD)/target/debug/bashrs corpus run
bwrap: execvp <worktree>/target/debug/bashrs: No such file or directory      # rc=1
```

Not a corpus failure and not a scoring failure: the sandbox hid the artifact it
was pointed at. A gate that cannot start is not a gate that passes.

## GREEN

```
bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp --ro-bind $(PWD) $(PWD) \
      --bind $E $E --chdir $E --unshare-net --die-with-parent \
      $(PWD)/target/debug/bashrs corpus run                                  # rc=0
  B3 Behavioral   18574/18814 (98.7%)   C Coverage avg 100.0%
  D Lint clean    18811/18814 (100.0%)  E Deterministic 18814/18814 (100.0%)
  F Metamorphic   18745/18814 (99.6%)   G Cross-shell   18758/18814 (99.7%)
```

Re-exposing `$(PWD)` over the tmpfs restores exactly what `--ro-bind / /` already
granted everywhere outside `/tmp`, and nothing more.

## Why read-only, measured rather than argued

The first version of this fix wrote `--bind $(PWD) $(PWD)`, which is a
read-WRITE mount: it hands the host repository to the corpus runner, which
executes 18,814 untrusted shell entries. That is a larger grant than the bug
needs, and it is a grant the sandbox did not previously make — under
`--ro-bind / /` the repository was readable and not writable.

The question "is the write access load-bearing?" was settled by running both:
the read-write form scores the corpus (rc=0) and so does the read-only form
(rc=0, the run above). Two runs, same verdict, so the weaker mount is the
correct one. `--chdir $E` already puts the run in a writable scratch directory
bound in separately, which is where a corpus entry's writes belong.

Found by review, not by me: two of three lanes refused the read-write form.

## What this branch does NOT carry

`.pmat/baseline.json` is regenerated and staged by the repository's own
post-commit hook, so it lands in the NEXT commit on the branch without anyone
touching it. The first version of this branch carried 89,008 lines of it, which
is not review material and was correctly refused by both failing lanes. The
branch was rebuilt from `main` and the baseline restored after each commit;
`git diff origin/main...HEAD --numstat` names the Makefile line and this receipt,
and nothing else.

## Scope

One line of `Makefile`, plus this receipt. The roadmap row for PMAT-340 is NOT
in this diff and does not need to be: it reached `main` on the PMAT-339 branch
(#342, merged), byte-identical to the copy this branch had added but for its two
timestamps, so the base merge resolved the roadmap to main's copy whole and this
branch's roadmap contribution is a no-op. Three review lanes refused an earlier
version of this receipt for claiming a roadmap entry the diff no longer carried,
which is the claim this paragraph replaces. The `else`
branch — no `bwrap` on the box — is untouched and still carries its PMAT-256
warning that the runner then executes every Bash entry with the caller's cwd,
HOME and PATH.
