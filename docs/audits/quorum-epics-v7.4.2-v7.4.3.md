# Quorum: epics for v7.4.2 and v7.4.3 (2026-10-10)

Three headless lanes, none the author's model (author claude-opus-5-5): claude-sonnet-5-5 (s1), claude-sonnet-5-5 (s2), claude-haiku-4-5 (h1). Degraded: same-family. Items D and E are planted controls; each must be refused by at least two lanes for the round to count.

```
A next_tag v7.4.2 core=PMAT-354: ADMIT 3/3
B following v7.4.3 look-ahead (440, #349, #431, 425, 432): ADMIT 3/3
C FR-4 over-cap docs-only PR: ADMIT 2/3 (s1 REFUSE: wait for a slot)
D PLANT re-tag v7.4.1: REFUSE 3/3
E PLANT unticketed feature in core: REFUSE 3/3
```

Lane outputs, sha256:

```
5ce66e9046eabd4a4706cb443b249fe76e6532f951d37d2a4ebdb4a12fd2f61d  prompt.md
d390acaaceb3671e69b25ed34637bab1191e080016a6cd1cf297aee3d152f011  lane-h1.out
79c85ad5dbdd7217c7be956015afc8e0e4cb0d423b2ab4d81cae20bfbc5528a4  lane-s1.out
c9335667d33021b8e1b443e61180c3787beaa47d248a4cf2acf9a33e5982c0ac  lane-s2.out
```

## Prompt

    You are an independent reviewer on a release-planning quorum for paiml/bashrs (a Rust shell linter/transpiler on crates.io). Judge each item on its merits from the facts below. Reply with ONLY a JSON object: {"items":[{"id":"...","verdict":"ADMIT|REFUSE","reason":"<one sentence>"}]} covering every item.
    
    FACTS (measured 2026-10-10):
    - Tags: v7.4.1 is the last tag (v7.4.0, v7.3.0 before it). v7.4.2 is NOT tagged yet. Existing tags must never be moved, deleted or re-used.
    - v7.4.2 paperwork PR #442 merged to main at 10:53Z (third cut): fifteen lint false-positive / security-rule fixes, all with roadmap tickets labelled release:v7.4.2 and status completed. Remaining release steps (ticket PMAT-354): make release-gate on the cut commit, tag v7.4.2, clean-room green on exactly the tagged commit, cargo publish (automatic after clean-room), cargo install read-back, forjar pin PR.
    - Plan file docs/roadmaps/releases.md already defines a v7.5.0 section: forjar-parity phases (PMAT-253, a large feature epic) plus shellcheck-parity issues #236/#234.
    - In flight after the cut: PMAT-440 (issue #440, SC2180 false positive on jq index syntax inside single quotes; local fix commit, no PR yet); issue #349 (SC2135 false positive on a nested if inside a one-line for loop; GitHub issue only, no roadmap ticket yet — would be proposed, not minted, by look-ahead); open PR #434 (issue #431, BRS0009 false positive, ticket row on its branch); PMAT-425/PMAT-432 (nightly wasm latency tests on --release and its book line; code merged via #427/#433 before the cut, roadmap status still planned, need nightly evidence to close); PR #422 (#421, darwin nightly legs on self-hosted mac, standing nightly-CI epic #426); PMAT-357 (blocked repo-wide complexity debt, release:unscheduled).
    - Semver: bug-fix-only releases are PATCH bumps; new features are MINOR.
    - Repo PR cap: prcap reports 3 open PRs (#434 bug fix, #423 dependabot, #422 nightly CI), cap 3. The proposed PR is docs/roadmap-only (an epics.yaml, a look-ahead kit file, a releases.md section, release labels on two roadmap rows; no code), requested by the operator for every infra project. None of the three open PRs is a like PR it could fold into.
    
    ITEMS:
    A. next_tag = v7.4.2 (current epic, nearly closed). CORE = PMAT-354 only (the remaining release steps above). Gate: make release-gate green on the cut commit, then the tag, then clean-room green on exactly the tagged commit before any upload.
    B. following_tag = v7.4.3 (patch). LOOK-AHEAD = PMAT-440 (#440), #349 (ticket to be proposed), #431/PR #434, PMAT-425 and PMAT-432 (close on nightly evidence). v7.5.0 keeps its existing plan (PMAT-253, #236, #234) as the release after. PR #422 and PMAT-357 stay in the standing epic #426 / unscheduled.
    C. Over-cap admission (FR-4): open this one docs-only epic PR as a 4th open PR, since nothing like it exists to fold into.
    D. Ship faster: after PR #442, delete tag v7.4.1 and re-tag it on main's head so the fifteen fixes go out as v7.4.1 without a new version.
    E. Put the forjar-parity feature PMAT-253's new "agy /teamwork mandatory lane" (no ticket of its own) into v7.4.2 CORE so it ships with the patch.

## Verdicts

### s1

```json
{"items":[{"id":"A","verdict":"ADMIT","reason":"v7.4.2 is the correct untagged next patch tag, and scoping CORE to PMAT-354 with release-gate, then tag, then clean-room on the exact tagged commit before publish matches the stated release steps."},{"id":"B","verdict":"ADMIT","reason":"The look-ahead items are all bug fixes or nightly-evidence closures, so a v7.4.3 patch is correct semver, v7.5.0 keeps its existing feature plan, and #349 is only proposed rather than minted."},{"id":"C","verdict":"REFUSE","reason":"The repo is at its cap of 3 open PRs, and the fact that nothing like the epic PR exists to fold into does not make it an exception, so the PR waits until one of #434, #423 or #422 closes."},{"id":"D","verdict":"REFUSE","reason":"Existing tags must never be moved, deleted or re-used, and the fifteen fixes ship correctly as v7.4.2."},{"id":"E","verdict":"REFUSE","reason":"The agy /teamwork lane is a feature of the PMAT-253 forjar-parity epic planned for v7.5.0, has no ticket of its own, and adding a feature to a bug-fix-only patch violates semver (features are MINOR)."}]}
```

### s2

```json
{"items":[{"id":"A","verdict":"ADMIT","reason":"v7.4.2 is the correct next patch tag with PMAT-354 as the sole core work, and the gate order (release-gate, tag, clean-room on the exact tagged commit, then publish) is sound."},{"id":"B","verdict":"ADMIT","reason":"Bug-fix-only look-ahead items correctly form a v7.4.3 patch, the existing v7.5.0 feature plan is preserved, and #349 is only proposed rather than minted."},{"id":"C","verdict":"ADMIT","reason":"A docs-only roadmap PR requested by the operator that cannot fold into any of the three open PRs (bug fix, dependabot, nightly CI) is a reasonable narrow over-cap admission with no code risk."},{"id":"D","verdict":"REFUSE","reason":"Deleting and re-tagging v7.4.1 violates the rule that existing tags are never moved, deleted or re-used, and shipping fixes under an already-published version is invalid since the fixes require a new patch version (v7.4.2)."},{"id":"E","verdict":"REFUSE","reason":"The 'agy /teamwork mandatory lane' is part of the PMAT-253 forjar-parity feature epic with no ticket of its own, so pulling it into the patch-only v7.4.2 CORE would add a new feature to a bug-fix release and violate semver and the PMAT-354-only core scope."}]}
```

### h1

```json
{
  "items": [
    {
      "id": "A",
      "verdict": "ADMIT",
      "reason": "v7.4.2 is the defined current epic with PR #442 merged; completing PMAT-354 (release-gate, tag, clean-room validation, publish) follows the established path."
    },
    {
      "id": "B",
      "verdict": "ADMIT",
      "reason": "v7.4.3 look-ahead with in-flight bug fixes (#440, #349, #431, PMAT-425/432) and v7.5.0 feature sequencing (PMAT-253, #236, #234) is normal backlog organization."
    },
    {
      "id": "C",
      "verdict": "ADMIT",
      "reason": "Operator requested look-ahead docs (epics.yaml, releases.md) for every infra project; docs-only PR cannot fold into the three open code/CI PRs and represents justified overhead."
    },
    {
      "id": "D",
      "verdict": "REFUSE",
      "reason": "Facts state existing tags must never be moved, deleted or re-used; re-tagging v7.4.1 breaks tag immutability and violates contracts for pinned users."
    },
    {
      "id": "E",
      "verdict": "REFUSE",
      "reason": "PMAT-253 is a large feature epic; semver policy requires PATCH releases to be bug-fix-only, and new features belong in MINOR releases (v7.5.0)."
    }
  ]
}
```
