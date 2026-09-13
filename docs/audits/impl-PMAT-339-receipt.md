# impl receipt — PMAT-339 (with PMAT-341, which it cannot ship without)

verdict: PASS — the keyring tests no longer write the process environment, and `keyring init` no longer
reports success without writing anything.

## Why two tickets share one branch

They are one chain, and the second is the first's own falsifier.

1. Nine tests redirected the keyring with `std::env::set_var("XDG_CONFIG_HOME", …)`. Three of them
   discarded the result — `let _ = handle_installer_command(cmd)` with the comment "may fail due to env
   var race; exercises code path".
2. Fixing the race (PMAT-339) means passing the path in, which lets those three assert.
3. Asserting them turns two of them RED against the code as it stands, with `Keyring not initialized`
   printed one line under `✓ Initialized keyring at …`. That is PMAT-341.

So PMAT-339's own tests do not pass without PMAT-341's fix, and PMAT-341's falsifiers are PMAT-339's
tests. Splitting them would land a commit whose tests fail by construction. The third defect found in the
same session — `corpus-score`'s sandbox masking `/tmp` (PMAT-340) — has no such coupling and is a separate
PR.

## Verification

| check | result |
|---|---|
| `cargo test -p bashrs --lib installer_keyring_cmd` | 13 passed, 0 failed |
| 150-run race loop, `--test-threads=9`, sandboxed HOME/XDG | **61 of 150 failed before, 0 of 150 after** |
| `cargo clippy -p bashrs --all-targets` | clean |
| `cargo fmt -p bashrs -- --check` | clean |
| PMAT-341 falsifiers at the parent commit | `add after init` and `remove after init` both RED |

## What a reader should check

- `keyring_path_from` is pure and total: three tests cover `$XDG_CONFIG_HOME`, the `$HOME/.config`
  fallback, and neither (which is `./bashrs/installer/keyring.json` — the relative form the original code
  produced, kept deliberately).
- No test in `rash/src` writes the process environment any more; the only remaining mention of
  `set_var("XDG_CONFIG_HOME")` in the file is the comment explaining why not to.
- `installer_run_logic` no longer carries its own copy of the keyring path.

## Gap

The race loop is probabilistic evidence, not a proof: 0 of 150 is the measurement, and the structural
claim under it is that no test writes the shared environment any more, which is checkable by reading the
file. Both are stated rather than one standing in for the other.
