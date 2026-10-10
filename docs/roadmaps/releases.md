# Releases

This is the plan `release-lint.sh` reads (PMAT-049, rules R1–R4). Every open roadmap entry names exactly one release. `unscheduled` is reserved for blocked entries. A tag is cut only when its release has no open entry.

## v7.1.0

Goal: the five v7.1.0 issues fixed and measured against their own reproducers, the publish race from v7.0.4 closed, the first mechanical dogfood gates, a corpus grown where the sovereign repos are and the corpus is not, and dependencies current.

| entry | what |
|---|---|
| PMAT-254 | triage: release labels on every open entry, this plan |
| PMAT-255 | the release: #293 #294 #295 #265 #232, PMAT-244, PMAT-250, dependencies, comply, corpus growth, PMAT-253 phases 3a and 5 |
| PMAT-244 | SC2105 reported for a `break` inside a one-line loop |
| PMAT-250 | `shell_words` exposes command substitutions, so SC2046 drops its private scanner |

Issues on the GitHub milestone `v7.1.0`: #293, #294, #295, #265, #232.

## v7.2.0 (released 2026-09-11)

| entry | what |
|---|---|
| PMAT-253 | forjar parity, the phases not shipped in v7.1.0: 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b (`docs/specifications/pr-dogfood-parity-forjar.md`), with §4 settled by quorum |
| PMAT-243 | COV-MEASURE: measure line coverage, name the top ten gaps, record both |
| PMAT-246 | work contracts regenerated so pv validates all 103; the CB-1305 half is upstream (paiml-mcp-agent-toolkit#1306) |
| PMAT-256 | corpus runner sandbox: temporary cwd, HOME and PATH everywhere, bwrap where Linux has it (make corpus-score sandboxes the release run; the runner itself is v7.3.0) |
| PMAT-257 | the release itself: ten defects, pv gate 4 green, coverage 95.01 percent, the Pareto gates, the book |

## v7.3.0 (released 2026-09-12)

Goal: the lowerings the v7.2.0 corpus removals exposed, the loop-containment rule that dogfooding keeps asking for, and the sandbox that stops a test writing into the repository. Coverage stays at or above 95 percent and every ticket carries a falsification test the contract verifier checks.

| entry | what |
|---|---|
| PMAT-258 | the release: #303, #316, #318, deps, comply, corpus growth, the book |
| PMAT-259 | corpus_converged gains a _with_log twin, so a test does not depend on the checkout |
| PMAT-260 | regenerate Cargo.lock for 7.3.0, without which --locked builds and the publish script fail |
| PMAT-261 | cover the convergence check, restoring coverage above the 95 percent gate |
| PMAT-262 | the close-out: statuses, the verified release record, the v7.4.0 plan |

Issues on this release: #303 (SC2106 not implemented), #316 (53 corpus entries exercised std methods with no lowering), #318 (an untracked copy of the source tree written by a test).

## v7.5.0

Goal: the forjar-parity phases that have been carried since v7.1.0, and the shellcheck-parity gaps #236 still names after v7.4.0 re-measured it. Coverage stays at or above 95 percent, every ticket carries a falsification test, and every pull request carries three independent quorum verdicts.

| entry | what |
|---|---|
| PMAT-253 | forjar parity, the phases not shipped in v7.1.0 through v7.4.0: 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b (decision D4, gate S, shipped in v7.4.0) |

Issues on this release: #236 (SC2086 in a heredoc body, SC2154 on a sourced variable, SC1004 across a multi-line quoted argument — measured in `docs/audits/sc-vs-shellcheck-v7.4.0.md`), #331 (SC2242 reads nested loops through three booleans and never closes a one-line `case … esac`), #234 (the TDG grade gate: 120 functions below grade A). Backlog: none — every open issue names a release.

## v7.4.0 (released 2026-09-12)

Goal: the forjar-parity phases that keep being carried, and the corpus runner's own sandbox rather than the release target's. Coverage stays at or above 95 percent, every ticket carries a falsification test, and every pull request carries three independent quorum verdicts.

| entry | what |
|---|---|
| PMAT-263 | the release: #233 (the required check lints the workspace, a nightly runs every test target), gate S with the per-file self-lint ratchet (PMAT-253 decision D4), deps, 203 corpus entries, the book |
| PMAT-253 | forjar parity, the phases not shipped in v7.1.0 through v7.3.0: 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b; decision D4 (gate S) shipped here under PMAT-263 |
| PMAT-256 | corpus runner sandbox: the runner itself, not only `make corpus-score` — shipped in v7.3.0 under PMAT-258 (#318); the close-out records the measurement of its third criterion |
| PMAT-265 | `std::env::var` is an environment read; its Result methods, a default spelled for the expansion it sits in, arithmetic in `capture()` — found by validating the release's own corpus candidates |
| PMAT-264 | the close-out: statuses, the verified release record, the v7.5.0 plan |

Issues on this release: #233 (CI lints only the bashrs-specs stub) — the clippy-scoping half closed here; the integration-target half was closed in v6.67.0 and re-measured at 0 errors. Backlog: #234.

Released: tag `v7.4.0` on 4aa24ca271, `bashrs 7.4.0` on crates.io, `cargo install bashrs --version 7.4.0` verified (reports `bashrs 7.4.0`, corpus 18,814 entries at 99.4/100 A+).

## v7.4.2

Goal: the defects found after v7.4.1 released as a patch, so forjar's bashrs pin can move off 6.68.0 and `ci-disk-watch-timer-enable` converges. The first cut (2026-09-20, #356) carried four fixes and was not tagged; nine more merged before the second cut (2026-10-04) and two after it, so the release carries fifteen fixes and two changes to the nightly build.

| entry | what |
|---|---|
| PMAT-354 | the release: version, changelog, notes, release gate, clean-room, publish, and the forjar pin PR that proves it |
| PMAT-350 | #351: SEC005 matched `sk-` as a bare substring, so `ci-disk-watch` read as an OpenAI key; a prefix counts only at a token start |
| PMAT-352 | #353: the x86_64-only `renacer` dev-dependency broke every aarch64 build of a crate nothing links |
| PMAT-343 | #332: SC2242's `in_case`/`in_loop`/`in_function` are depth counters, so a one-line case ends where it ends (closes #331) |
| PMAT-355 | #355: counting then made the LITERAL case worse — SC2242 read raw source, so `echo "could not break the matcher — this case discriminates nothing"` opened a case depth on the word "case" and found `break` in "break the matcher". It joins `QUOTE_SENSITIVE_RULES` and is wired into the allowlist's `(code, check_fn)` list, without which nothing ever ran it over a literal. Found by `make release-gate` ON this cut, so it ships in the release rather than after it |
| PMAT-362 | #362: SC2086/SC2154 fired on `$name` inside `'...'` and after a trailing `#`; both are linted against `mask_inert` |
| PMAT-364 | #364: eight rules (SC2204, SC1075, SC2297, SC2102, SC1099, and SC1012/SC2025/SC2112 at info) read a single-quoted awk program as shell; they join `QUOTE_SENSITIVE_RULES` |
| PMAT-366 | #366: SC2107 read `[ "$(a \|\| b)" = x ]` as `[ a \|\| b ]`; the inside of `$( )`, `$(( ))` and backticks is blanked before matching |
| PMAT-370 | #370: the SC2210/SC2225 assignment regex crossed word boundaries (`pid= --ppid`); assignments are found by word position |
| PMAT-371 | #371: SC2058 matched `test`/`[` outside command position (`cargo test -q`); only the command name of a simple command counts |
| PMAT-376 | #376: DET002 lost the sink of a `\`-continued command; continued lines are joined before the destination is resolved |
| PMAT-380 | #380, #385: the nightly's Linux assets needed glibc 2.39 and there was no aarch64-linux leg; both legs build on glibc 2.31 and are checked to run on 2.35 |
| PMAT-386 | #386: DET002 reported `date -d`/`-r`/`-f`, which convert a time they are given and read no clock |
| PMAT-388 | #388: SC1066 read `$h=` inside a double-quoted string as an assignment; it joins `QUOTE_SENSITIVE_RULES` |
| PMAT-428 | #430: SEC012 missed `eval "$X"` when `X` was assigned from `$(yq ...)`/`$(jq ...)`/`$(curl ...)` on an earlier line; the assignment is remembered and the later `eval` fires, naming its line |
| PMAT-439 | #441: SC2075 read `'\\'` followed by another single-quoted string as an escaped quote and reported an Error on a correct script, which made forjar's I8 gate refuse fw16's `fw16-wired-10g-nm-owner` completion check; `echo 'don\'t'` still fires |

Also on this release, with no roadmap entry: #375 (#399), where SC1087 read `$s[0]` in a single-quoted jq program as an array expansion and SEC012 read `.eval_count` as `eval`; and #397, which builds the nightly whenever the `nightly` tag is not at HEAD.

Issues on this release: #331, #350, #352, #354, #355, #362, #364, #366, #370, #371, #375, #376, #380, #385, #386, #388, #428, #439.

## v7.4.1

Goal: the four defects found after v7.4.0, each fixed with a test that fails without the fix, released as a patch.

| entry | what |
|---|---|
| PMAT-338 | the release: version, changelog, book, release gate, clean-room, `cargo install` dogfood of the packaged crate, publish |
| PMAT-335 | #335: `bashrs fix` rewrote a working script into a different program; quote segments are paired inside words, SC2081's fix needs `--assumptions`, and `apply_fixes` refuses a fix that changes the program (#337) |
| PMAT-266 | the security gate read a missing `cargo-deny` as a violation; the nightly installs it (#334) |
| PMAT-339 + PMAT-341 | #339, #341: the keyring tests wrote the process environment and raced (61/150 -> 0/150); `keyring init` built a keyring and dropped it (#342) |
| PMAT-340 | #340: `corpus-score`'s bwrap masked /tmp, so the release gate could not run from a worktree under it (#343) |

Issues on this release: #335, #338, #339, #340, #341.

## Unscheduled (blocked)

Nothing. The four entries that sat here blocked on definition were decided by quorum on 2026-09-11: PMAT-241 and PMAT-242 closed as completed in v6.66.0, PMAT-243 redefined as a measurement in v7.2.0, PMAT-246 closed by regenerating the work contracts. See `docs/audits/quorum-decisions-v7.2.0.md`.

## Backlog issues

#236, #234 and #233 sit on the GitHub milestone `Backlog`; they are issues, not roadmap entries, so no release names them.
