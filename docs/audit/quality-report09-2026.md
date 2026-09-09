# bashrs Quality Report — September 2026

**Commit:** `3c50327fff` (== tag `v7.0.1`, 0 commits since) · **Date:** 2026-09-09 · **pmat:** 3.39.0
**Method:** 3 agy quorum lanes (opinion) + 2 measurement subagents, every headline number re-executed by the orchestrator before publication. Claims not re-executed are marked *[unverified]*.

---

## 1. Headline answer

| Question | Answer |
|---|---|
| Open GitHub issues | **21** |
| Open pull requests | **19** (15 dependabot / 4 human) |
| Open pmat tickets | **7** — 6 `inprogress` (PMAT-238, -239, -241, -242, -243, GH-189) + 1 `planned` (PMAT-240); 111 completed, 2 cancelled |
| Known defects | **24 distinct** — 21 tracked as issues (S1=11, S2=5, S3=5) + **3 untracked, found by this audit**, one of them P0 |

**The P0 is not on the issue tracker: the corpus is empty.**

---

## 2. P0 — `bashrs corpus run` scores 0 entries

The v7.0.1 release headline is *"zero false positives on the corpus, with every SEC finding intact."* That statement is **vacuously true**: the corpus contains no entries.

```
$ /mnt/nvme-raid0/targets/bashrs/debug/bashrs corpus run     # built Aug 30, v7.0.1
$ bashrs corpus run                                          # ~/.cargo/bin, v7.0.1
  V2 Corpus Score: 0.0/100 (F)
  Entries: 0 total, 0 passed, 0 failed (0.0%)
```

Reproduced on **two independently built binaries**, from two working directories. Against a documented baseline of **17,942 entries / 99.1 A+**.

### Root cause (five whys)

1. *Why 0 entries?* — `CorpusRegistry::load_full()` (`rash/src/corpus/registry/mod_corpusentry.rs:132`) calls 355+ `load_expansion*()` loaders.
2. *Why do the loaders add nothing?* — `rash/src/corpus/registry/corpus_data_load.rs` holds **139 loader functions, 139 of which have an empty body `{}`; zero have a body.**
3. *Why are they empty?* — `a98f6a89d0` "refactor: split corpus_data.rs (1107→322) — PMAT-229" **created** the split file with stub bodies (4 files, +1664 lines, **0 deletions** — the split never applied). `ee8c6a2d04` "fix: actually apply … splits" then **deleted the real bodies**: `corpus_data.rs` 787 lines → 2.
4. *Why did nothing catch it?* — the corpus gate is not a required CI check, and a 0-entry corpus **cannot fail**: 0 failures / 0 entries reads as a clean run to every consumer of the score.
5. *Why is the blast radius wider?* — the same PMAT-229 pass (`1468c41499`, "include!() split 254 files", 447 files, 134,737+/134,575−) left **720 zero-byte `*_incl*.rs` files** in the tree, several of them still `include!`d (e.g. `adversarial_templates_adversarialtemplate.rs:486` includes a 0-byte file). Repo-wide `CorpusEntry::new` is now **76 occurrences across 18 files**.

### Impact

Every corpus-derived quality claim since 2026-04-05 is unsupported: the 99.1/100 A+ score, the V2 A–G breakdown, "zero false positives on the corpus", and the corpus half of the release checklist. It also explains the tracker: **13 open false-positive issues filed by outside use in the last 21 days** against a linter whose regression corpus stopped containing anything five months ago.

### Recommended P0 ticket

```
P0: CorpusRegistry::load_full() loads 0 entries — corpus gate cannot fail
Severity: P0 — STOP THE LINE      Category: Corpus/Registry
Regression: a98f6a89d0 + ee8c6a2d04 (PMAT-229 split), 2026-04-05
RED:   assert!(CorpusRegistry::load_full().entries().len() >= 17_000)
GREEN: restore loader bodies from ee8c6a2d04^:rash/src/corpus/registry/corpus_data.rs
GATE:  make the entry-count assertion a required CI check, so an empty corpus fails loudly
```

---

## 3. Two further untracked defects

**U-2 — `unwrap_used` is `allow`, not `deny`.** CLAUDE.md and the README both document `unwrap_used = { level = "deny", priority = 1 }` as an enforced Cloudflare-class gate. The workspace `Cargo.toml` says otherwise:

```toml
# CI uses -D warnings which promotes warns to errors. Allow until remediated.
unwrap_used  = { level = "allow", priority = 1 }
expect_used  = { level = "allow", priority = 1 }
```

Measured non-test `unwrap()` calls: **998** in non-test `.rs` (890 in `rash/src/` alone) *[unverified — subagent count]*. Four figures are on record and none agree: CLAUDE.md says 289, the `Cargo.toml` comment says 649, pmat's TDG defect scanner says 554, literal count says 890. The policy is documented as enforced and is not enforced.

**U-3 — 720 zero-byte source files.** Artifacts of the PMAT-229 include-split (see §2). Some are dead; some are live `include!` targets contributing nothing. Related to but larger than issue #254, which sampled 7 of them.

---

## 4. Issue backlog (21)

Classification *[unverified — subagent, from `gh issue list --json`]*:

| Class | Count | Issues |
|---|---|---|
| **FP** false positive | 13 | 264, 262, 261, 258, 257, 256, 255, 252, 242, 241, 238, 237, 235 |
| **FN** false negative / scope gap | 2 | 263, 249 |
| **INTEROP** | 2 | 265, 236 |
| **DEBT** | 3 | 254, 234, 233 |
| **ENH** | 1 | 232 |

Severity: **S1 (release-blocking) = 11 · S2 = 5 · S3 = 5.**

Everything on the tracker is one of two stories: **the lexer reads non-shell text as shell** (string bodies #235/#241/#252, heredoc bodies #242, `$(( ))` #237, Makefile comments #255, trailing comments #258, printf formats #261) and **codes mean different things than shellcheck's** (#236 SC2046 fires 210×, #265 suppression comments silently disable shellcheck itself).

Backlog shape: the issue queue is **fresh, not accumulating** — oldest open issue is 21 days old; 45 opened vs 25 closed in 30d. The PR queue is the opposite.

---

## 5. Pull requests (19)

| | |
|---|---|
| dependabot / human | 15 / 4 |
| Draft | 0 |
| `CONFLICTING` | **15 of 19** |
| ≥1 failing or cancelled check | **14 of 19** |
| Age of oldest (#138, crossterm) | **194 days** |
| dependabot PRs merged in last 90d | **0** (26/26 recent merges human-authored) |

The dependabot lane is **structurally stuck**, not slow: conflicts plus failing CI, with zero merges in 90 days. Four of these (#147 schemars, #145 rand, #143 rand_chacha, #142 sysinfo) are real dependency-currency debt, and `rust-project-score` docks Dependency Health to 9.5/12 for it.

*[unverified — subagent, from `gh pr list --json ...statusCheckRollup`]*

Hygiene: **21/21 open issues carry zero labels; 0 have a milestone.**
**0 of 21 issues are linked to or referenced by any open PR** — nothing in flight is aimed at the tracker.

---

## 6. Measured quality surface

| Metric | Value | Verified |
|---|---|---|
| `pmat rust-project-score` | **209.6/245 · 85.6% · A-** | ✅ re-run |
| — Formal Verification | 8.5/16 (53.1%) | ✅ |
| — Reproducibility | 8.1/15 (54.0%) | ✅ |
| — Testing Excellence | 16.5/20 (82.5%) | ✅ |
| — Known Defects | 20/20 (100%) | ✅ *(scored blind to §2–3)* |
| SATD | **17** (0 critical · 1 medium · 16 low) | ✅ re-run |
| `cargo clippy --workspace --all-targets` | **0 errors, 94 warnings** | ✅ re-run |
| `cargo test --workspace --no-run` | **0 errors — 113 executables (104 integration + 9 unit)** | ✅ re-run |
| `pmat comply check` | **NON-COMPLIANT** — 166 checks: 67 pass / 15 warn / **10 fail** / 74 skip | *[unverified]* |
| — CB-200 TDG gate | **774** functions below grade A | *[unverified]* |
| — CB-400 shell/make quality | 1,253 errors / 5,849 warnings | *[unverified]* |
| — CB-2100 | 9 error-severity rules unreachable from required CI checks | *[unverified]* |
| TDG grades (`rash/src/`) | 954 graded, avg 81.6 (B), **107 F-grade files** | *[unverified]* |
| Complexity | max cyclomatic **31**, max cognitive **111**; **191** fns over the stated limit of 10 | *[unverified]* |
| Reachability | **443 orphan `.rs` files**, 99,623 lines, **3,596 `#[test]` fns that never run** | *[unverified]* |
| Line coverage | **not obtainable** — `.pmat/coverage-cache.json` empty for this HEAD | *[unverified]* |
| Contracts | 5 YAML in `provable-contracts/contracts/bashrs/`; 99 `.pmat-work/*/contract.json`; **78/99 claim L3 on L1 evidence** | *[unverified]* |

### Two tracker claims that no longer hold

- **#233 — "37 of 145 integration test targets do not compile": FALSIFIED at HEAD.** `cargo test --workspace --no-run` completes with 0 errors and builds 113 executables against 107 declared test files. PMAT-238 appears to have closed this; the issue should be updated or closed.
- **#234 — "120 functions below grade A": understated.** `pmat comply check` CB-200 now reports **774**. The companion figure, 107 F-grade files, still matches exactly. (Caveat: `.pmat/project.toml` pins PMAT 3.11.1 against an installed 3.39.0, so the CB-200 algorithm may have changed — 774 is the honest current reading, not a proven 6.45× regression.)

---

## 7. Quorum verdicts

Three independent agy lanes (agy 1.1.28, `--sandbox`, read-only) — conversations `a1cc9b96`, `5eaa973d`, `e828564b`.

| Lane | Question | Verdict |
|---|---|---|
| 1 | "v7.0.1 has no release-blocking defect" | **FAIL** — 13 FPs, SEC011 FN (#264) and the suppression-interop bug (#265) are each release-blocking |
| 2 | "the A- grade is defensible" | **FAIL** — the corpus score is unsound; A- overstates real quality |
| 3 | "the in-progress ticket set targets the right problem" | **FAIL** — PMAT-243 chases 95% coverage while integration tests and the corpus rot |

**3/3 FAIL.** Two caveats, both from the delegate's own receipt:

1. **All three lanes ran `num_turns=1` with zero `grounding=measured` findings.** They reasoned over orchestrator-supplied context and opened no file. These are opinion lanes, not verification.
2. **Lane 3's premise was falsified on re-execution** (§6: the tests compile). Its conclusion may still stand on the corpus argument, but not on the reason it gave.

The lanes converged on "the corpus metric is overfitted to internal tests." The measurement disagrees in the lanes' favour and then some: the corpus is not overfitted, it is **empty**.

---

## 8. Verdict and ordered actions

The A- (85.6%) does not survive contact with §2. `rust-project-score` awards **Known Defects 20/20** while the project's own regression corpus has silently scored nothing for five months and 13 false-positive reports have arrived from real use in three weeks. The grade measures the presence of quality machinery, not its output.

1. **P0 — restore the corpus** and make a minimum entry count a required CI check (§2). Nothing else is measurable until this is true.
2. **Re-run the corpus against the 13 open FP issues.** They are the acceptance test for the restore: a corpus that passes while #235/#237/#241/#242/#252 reproduce is still not measuring anything.
3. **Fix the lexer-context bug class** behind 8 of the 13 FPs — string bodies, heredoc bodies and `$(( ))` are one defect wearing three codes, not eight bugs.
4. **Reconcile the unwrap policy** (§3 U-2): either restore `deny` with a documented allow-list, or amend CLAUDE.md and the README to state the real posture. The current gap is a documentation defect regardless of which way it is closed.
5. **Close or rebase the dependabot lane** — 15 conflicting PRs, 194 days, 0 merges in 90 days. Batch-rebase the 4 real dependency bumps; close the rest.
6. **Correct #233 and #234** with the numbers in §6, and label the backlog (21/21 unlabelled).
7. **Then** resume PMAT-243 (COV-95). Coverage of a tree with 443 orphan files and 3,596 tests that never run measures the wrong denominator.

---

*Generated by a quorum audit: 3 agy lanes + 2 measurement subagents, orchestrator-verified. Raw lane artifacts: `scratchpad/agy/lane-{1,2,3}.json`, `receipt-final.json`.*
