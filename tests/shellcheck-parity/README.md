# shellcheck parity: recorded expectations (RA-1, bashrs#635)

Records what the pinned shellcheck says over `corpus/*.sh` and replays it, instead of hand-written expectations.

- `PIN`: the shellcheck version this corpus is recorded against (`0.8.0`). A different version is UNMEASURED (exit 2), never a pass.
- `record.sh`: runs shellcheck over the corpus and writes `expected.tsv` (`file<TAB>SC-code<TAB>count`).
- `replay.sh`: runs shellcheck again and prints `SCxxxx matched N/M` per rule. Exits non-zero on any mismatch or any new finding.
- `selftest.sh`: plants a wrong expectation and a dropped one; the replay must fail on both.

Run: `bash tests/shellcheck-parity/replay.sh` then `bash tests/shellcheck-parity/selftest.sh`.
Not wired into CI (no CI or gate change in this PR).
