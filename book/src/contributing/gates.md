# Quality Gates: PR and Release

bashrs runs two gates, and they are deliberately different sizes. The numbers below were measured on the v7.2.0 branch with a warm target directory; re-measure them when they matter, and update this page when they move.

## Why two gates

The full workspace test run spends almost all of its wall time compiling and linking test binaries, not running tests. Across more than a hundred integration binaries the run executes about 41 seconds of tests and spends the other fifteen minutes linking. The library target alone holds 97.2 percent of the tests and links once.

| gate | command | wall time | tests run |
|---|---|---|---|
| PR | `make pr-gate` | 111s | 15,282 |
| pre-release | `make release-gate` | 944s | 15,721 across 30 or more targets |

So the PR gate is 88 percent faster and runs 97 percent of the tests. That is the Pareto split: a pull request gets fast, honest feedback; a tag gets everything.

## `make pr-gate`

Format check, clippy on the library, the library tests, and `pv lint contracts`. If `cargo-nextest` is installed the tests run under it, which is faster still on a many-core machine.

```bash
make pr-gate
```

## `make release-gate`

Everything the PR gate runs, plus the full workspace test run, the coverage gate, the corpus score and the book check. Run it before cutting a tag, never instead of the PR gate.

```bash
make release-gate
```

## The coverage gate: 95 percent

Line coverage of the `bashrs` library must be at least 95 percent, measured with `cargo llvm-cov --lib -p bashrs`. The gate fails the release below that.

```bash
make coverage-gate
```

Two things about that measurement matter when you add tests:

- It measures the **library test binary only**. A test under `rash/tests/` runs in the release gate but moves the coverage number by nothing. Coverage tests live in `#[cfg(test)]` modules inside the library, next to the code they cover.
- A CLI handler that loads the whole corpus cannot be unit-tested as it stands: it would execute 18,000 shell snippets per test. The convention is to move the handler's body into a `<name>_with(registry: &CorpusRegistry, ...)` twin, have `<name>` call `load_full()` and then the twin, and test the twin with a three-entry registry. One test then covers the whole handler in milliseconds. v7.2.0 did this across the corpus CLI to take the library from 91.72 to 95.01 percent.

Never `cargo tarpaulin`; use `pmat query --coverage-gaps` to find what to cover next.

## The contract gate: pv

`pv lint contracts` must pass, and its gate 4 (verify) is the one that bites: every `falsification_tests[].test` field in a contract must name exactly one real test function, in the form `cargo test -p bashrs --lib <module>::<test_fn>`. pv takes the text after the last `::` and looks for a bare `#[test]` function with that name. A comma-joined list, a name followed by prose, or a prefix of the real name all count as a missing reference, and a missing reference means a contract claim nothing can falsify.

Every fix that ships names its falsification test in the contract that owns the rule. Sixteen such entries were repaired in v7.2.0 and ten were added, one per fix.

## The corpus and the sandbox

`make corpus-score` scores the whole corpus. The runner executes every Bash entry, so the target runs it inside `bwrap` when the sandbox exists and says so plainly when it does not. Entries are screened before they are added: no network, no writes outside the working directory, no package managers, no `sudo`, no `crontab`.

## The book

A tagged release updates this book. `scripts/check-book-updated.sh` builds the book and runs its examples as part of the release gate, so a release whose book does not build does not ship.
