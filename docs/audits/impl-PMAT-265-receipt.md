# PMAT-265 receipt — v7.4.0

## Identity

| field | value |
|---|---|
| ticket | PMAT-265, kind=code, `release:v7.4.0` |
| branch | PMAT-265-env-var-lowering, rebased onto main at 4a31214df8 (the v7.4.0 release merge) |
| gate_cmd | `make pr-gate` |
| required_check | `gate` |
| author model | claude-opus-5 |

orch_model: opus-5 [V]   orch_class: opus   orch_decision: admit   orch_basis: file
fable_binding: false   quota_age_h: absent   quota_mark: -   k_measured_at_set: refused (one ticket per session; PMAT-248 holds the goal file)

routes:
  ph1  class=impl  route=self  w=100.00  basis=absent   RED tests + contract entries
  ph2  class=impl  route=self  w=100.00  basis=absent   the lowerings, the quoting, the validator
  ph3  class=orchestration  route=self  w=100.00  basis=absent   corpus re-validation, docs, receipt

verification:
  cmd="make pr-gate"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-265/pr-gate.log  sha256=6ea9eaed65b5

## Where it came from

Not a report: the v7.4.0 corpus lanes wrote 240 candidates and 24 of them failed to transpile. Twenty named one defect (`std::env::var`), three named another (`$((…))` in a `capture()` string), and validating the survivors by hand under bash and dash found a third (the default's quoting). This diff fixes all three and adds the 19 candidates that transpile as a result.

## Verification: claimed against my own rerun

| check | result |
|---|---|
| `cargo test -p bashrs --lib pmat265` | 6 passed (all six were red before the GREEN commit; the RED commit is in this branch's history) |
| `make pr-gate` | green: **15,693 library tests**, fmt, clippy lib, `pv lint contracts`; 115s |
| `pv lint contracts` gate 4 | **86 refs, 86 found, 0 missing** (80 before; the six new entries resolve) |
| corpus | 18,795 → 18,814; the 19 added were each transpiled and run under bwrap with `timeout 2`, none timed out |
| shellcheck | `shellcheck -s sh` clean on a script exercising all four new spellings |
| cross-shell | the six defaults of F-TCORE-028 print byte-for-byte under bash and dash; measured by hand under busybox too |

## Quorum review, round 1: what it caught

Three lanes (gemini-3.1-pro-high, gemini-3.7-flash-high, gemini-3.8-flash-high), 1 PASS and 2 FAIL. Every cited location was re-read here; three of the four findings were right.

| finding | verdict here |
|---|---|
| the diff lowers `unwrap_or_default()` to `"${X-}"` while the ticket's text said `${X}` | **right about the contradiction, wrong about which side to change.** The generated script carries `set -u`, under which a bare `"${X}"` aborts on an unset name — measured, dash exits 2 with "X: parameter not set" — and `String::default()` is the empty string, so aborting is the wrong answer. The ticket was written before that measurement; its acceptance criteria now state the spelling and the reason, so the diff and the ticket agree and the claim stays falsifiable. |
| CHANGELOG said the entries still out are ".len() on a defaulted env read", but B-17719/17720/17721 do exactly that | **right.** The ones still out chain `.len()` *directly* onto the defaulted read, which has no variable to take `${#var}` on; binding it to a variable first works and is what those three entries cover. The CHANGELOG now says so. |
| "18,592 to 18,814" with "the 203 added" is inconsistent (222) | **right.** 203 in the first pass plus 19 here; the bullet now states both. |
| `env("X").<method>` was not requested by the ticket | **in scope, and now said so.** `env("X")` and `std::env::var("X")` are one read through one converter; lowering only the longer spelling would leave the DSL's own spelling broken. The acceptance criteria name both. |

No code changed in response to round 1: the three confirmed findings were claims in the ticket and the CHANGELOG, not defects in the diff, and they are corrected where they were wrong.

## Jidoka

| defect | owner | whys |
|---|---|---|
| a release's own corpus candidates found three transpiler defects | corpus | the corpus is generated from what people actually write → what people write reaches methods the lowering never covered → the failures were read as "bad candidates" in earlier cycles → v7.4.0 read them as findings instead → three fixes and 19 entries |

verdict: PASS — the four defects named in the ticket are fixed, each pinned by a falsification test, and the three confirmed review findings are corrected in the text that carried them.

IMPL-PMAT-265-RECEIPT-END
