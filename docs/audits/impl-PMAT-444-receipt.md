# PMAT-444 — receipt: keyring CLI tests each own a TempDir keyring (bashrs#444)

**Ticket:** PMAT-444 (bashrs#444), kind code, epic #426, release v7.4.2.
**Branch:** `fix/444-keyring-test-isolation` from `6ba7beb647`; fix commit `0ea0f32770`.

## Defect
`test_INSTALLER_CLI_007_keyring_init` and `_list` ran the binary with the ambient environment, so both
resolved the real `~/.config/bashrs/installer/keyring.json`. In parallel, list read the file while init
was writing it: `Failed to parse keyring: EOF while parsing a value at line 1 column 0` (red v7.4.2
release gate on 0ae1212911).

## Fix
`keyring_cmd(&TempDir)` sets `XDG_CONFIG_HOME` and `HOME` to the test's own directory. init asserts
the file lands there; list on a fresh dir asserts `Keyring not initialized`; new `list_after_init`
asserts list reads what init wrote. Census of keyring-file touchers: the unit tests in
`command_tests_installer_cov_tests_cov_2.rs` already pass the path in (#339); `installer run` reads the
keyring only under `--verify-signatures`, which no test passes. No other test touches the file.

## Discriminator (t2build, `INSTALLER_CLI_007 --test-threads=8`, one shared scratch HOME per loop)
| binary | runs | failures | shared HOME got a keyring |
|---|---|---|---|
| before (origin/main) | 300 | 2 | yes |
| before (origin/main) | 1000 | 13 | yes |
| after | 1000 x3 | 0, 0, 0 | no |

Full `cli_installer_tests`: 20 passed. `cargo clippy -p bashrs --test cli_installer_tests -D warnings`: clean.

## Planted review quorum (degraded: same-family; author claude-opus-5-5, never a lane)
Plant: the same diff with `keyring_list` reverted to `bashrs_cmd()` (ambient HOME). Lanes saw each diff blind.
| lane | real diff (sha256 d21c0bafceaf29ed…) | plant (sha256 d8dd7cfda4e79692…) |
|---|---|---|
| claude-sonnet-5-5 #1 | PASS | FAIL (names keyring_list on the real home) |
| claude-sonnet-5-5 #2 | PASS | FAIL |
| claude-haiku-4-5 | PASS | PASS (missed) |

Real 3/3 PASS; plant caught 2/3 (threshold >=2/3). Lane caveats (other keyring touchers, `TempDir`
import) are answered by the census above and `use tempfile::TempDir;` at line 13.

## FR-4 over-cap round (prcap: paiml/bashrs 3 open, cap 3)
Open PRs #434 (linter, v7.4.3), #423 (dependabot), #422 (darwin nightly CI): none is alike, so no fold.
Planted over-cap quorum, same lanes. Plant: a v7.4.3 doc-comment reword with nothing red, which must FOLD_OR_WAIT.
| lane | PMAT-444 | plant |
|---|---|---|
| claude-sonnet-5-5 #1 | OPEN_OVER_CAP | FOLD_OR_WAIT |
| claude-sonnet-5-5 #2 | OPEN_OVER_CAP | FOLD_OR_WAIT |
| claude-haiku-4-5 | OPEN_OVER_CAP | FOLD_OR_WAIT |

3/3 OPEN_OVER_CAP; plant caught 3/3. Lane outputs: `/mnt/nvme-raid0/tmp/bashrs-444-r1/q/` on lambda-labs.
