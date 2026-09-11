# PMAT-260 receipt — the 7.3.0 lockfile

## What was wrong

The v7.3.0 release bumped the workspace manifest to `7.3.0` and did not regenerate `Cargo.lock`, which still named `7.2.0` for the five workspace members. Measured on `main` before this change:

```
$ cargo metadata --locked --format-version 1
error: cannot update the lock file … because --locked was passed to prevent this
$ cargo check --locked -p bashrs --lib
error: (the same)
```

`scripts/publish-from-tag.sh` publishes with `--locked`, so v7.3.0 could not have been published from `main` as it stood. This is a release blocker, not housekeeping.

## How it was found

A quorum lane reviewing an unrelated pull request asked why that diff carried a lock bump. The bump was indeed out of scope there and was removed; the question is what exposed the blocker.

## The change

`cargo metadata` regenerates the file. Five workspace members move 7.2.0 to 7.3.0: `bashrs`, `bashrs-oracle`, `bashrs-runtime`, `bashrs-specs`, `bashrs-wasm`. No dependency version changes.

## Verification

| check | before | after |
|---|---|---|
| `cargo metadata --locked` | exit 101 | exit 0 |
| workspace members at 7.3.0 in the lock | 0 of 5 | 5 of 5 |
| dependency versions changed | — | none |

## Why it is its own ticket

Two quorum rounds refused a diff that was judged against the release ticket it was cut from: a review compares a diff with its ticket's stated intent, so a one-file fix needs a one-file ticket. This is the third time that rule bit in this release, and each time the reviewer was right.
