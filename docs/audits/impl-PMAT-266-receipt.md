# PMAT-266 receipt

## Identity

| field | value |
|---|---|
| ticket | PMAT-266, kind=code, `release:v7.5.0` |
| branch | PMAT-266-security-gate-probe, from main at 4aa24ca271 |
| gate_cmd | `make pr-gate` |
| required_check | `gate` |
| author model | claude-opus-5 |

orch_model: opus-5 [V]   orch_class: opus   orch_decision: admit   orch_basis: file
fable_binding: false   quota_age_h: absent   quota_mark: -   k_measured_at_set: refused (one ticket per session; PMAT-248 holds the goal file)

routes:
  ph1  class=impl  route=self  w=100.00  basis=absent   the three-outcome probe and its stub-cargo tests
  ph2  class=orchestration  route=self  w=100.00  basis=absent   the nightly installs cargo-deny; contract, CHANGELOG, receipt

verification:
  cmd="make pr-gate"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-266/pr-gate.log  sha256=a56de6ee69b8

## How it was found

The nightly full gate v7.4.0 added ran for the first time (`workflow_dispatch` on main, run 34689967236) and its `full-gate` job failed. The failing test was `test_PMAT257_cov_run_security_gate_none_and_disabled`, and the reason was not the test: the runner has cargo and not cargo-deny, so `cargo deny check` exits 101 with `error: no such command: \`deny\``, which `run_security_gate`'s `Ok(s) => s.success()` arm read as "the security check found violations". The `Err(_)` arm that prints "(cargo-deny not found, skipping)" is reached only when the `cargo` binary itself cannot be spawned.

That is a gate reporting a verdict about something it never measured — the class of #284 (a corpus that loads nothing scores clean) and of PMAT-258's bwrap probe (an unusable sandbox read as success).

## The fix

`cargo_deny_outcome(cargo)` probes `cargo deny --version` first and returns one of three outcomes:

| outcome | when | gate |
|---|---|---|
| `Absent` | the probe fails to spawn or exits non-zero | skip, and say so |
| `Clean` | the check ran and exited 0 | pass |
| `Violations` | the check ran and exited non-zero | **fail** |

Neither absence nor a real advisory is softened into the other.

`.github/workflows/nightly-full-gate.yml` installs cargo-deny (pinned by commit), so the nightly measures the same gate the sovereign CI container does rather than skipping it.

## Verification: claimed against my own rerun

| check | result |
|---|---|
| `cargo test -p bashrs --lib PMAT266` | 3 passed |
| the same three tests against the OLD two-arm match | **2 failed** (restored the old body, ran, restored the new) — the tests are red against the defect |
| `make pr-gate` | green: **15,696 library tests** (three more than v7.4.0's 15,693), fmt, clippy lib, `pv lint contracts`; 61s |
| `pv lint contracts` gate 4 | **88 refs, 88 found, 0 missing** (86 before; F-DOG-006 and F-DOG-007 resolve) |
| `actionlint .github/workflows/nightly-full-gate.yml` | clean |
| the pinned action sha | `9534c84618278caac52cb373bb164ed464dbd8af` = `taiki-e/install-action` v2.87.11, resolved through the GitHub API |

The tests write a stub `cargo` into a `TempDir` and call `cargo_deny_outcome` with its path, so none of them depends on whether this machine has cargo-deny installed — which is exactly the dependency that hid the defect.

## Jidoka

| defect | owner | whys |
|---|---|---|
| a gate reported a violation for a check that never ran | `rash/src/cli/gate_commands.rs` | two outcomes for three cases → `Err` means "cargo missing", not "cargo-deny missing" → every machine that ran the test had cargo-deny → the required check's container has it → nothing ever ran the test without it until the nightly did, one day after it was added |

verdict: PASS — the defect is fixed, the tests are red against the old code, and the nightly that found it now installs the tool so the gate is measured rather than skipped.

IMPL-PMAT-266-RECEIPT-END
