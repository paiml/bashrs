# PMAT-262 receipt — the v7.3.0 close-out

## What

Ticket statuses for PMAT-258, 259, 260 and 261; the verified release record appended to the PMAT-258 receipt; the v7.3.0 sections dated in the plan and the notes; PMAT-253 and PMAT-256 moved to v7.4.0, because `release-lint` R3 refuses open work that names a release already cut; and the v7.4.0 sections opened in both documents.

No code changes. `release-lint` reports, with this ticket itself counted, which is what makes the number three rather than two:

```
release-lint: open=3 releases=v7.4.0(3) highest_tag=v7.3.0
```

**That green depends on a local patch.** The upstream `release-lint.sh` skips only `completed`, so it counts the three long-cancelled entries (PMAT-168, PMAT-169, PMAT-240) as open work that must each name a release, exits 1, and reports `open=6`. A blind quorum decided on 2026-09-11 that cancelled is closed, and the one-line change is filed upstream as paiml/paiml-mcp-agent-toolkit#1326; the skill bundle has been upgraded twice since and discarded the patch each time. A quorum lane measured exactly this difference from an unpatched environment, which is why it is written down here rather than left as an unqualified "green".


## Why it is its own ticket

Three quorum rounds refused close-out and follow-up diffs in this release because each was judged against the release ticket whose code had already merged. A lane compares the diff with its ticket's stated intent, and a documentation close-out does not implement a release. The rule held every time it was tested, so a close-out gets a ticket of its own from now on, opened at the start of the cycle rather than discovered at the end.

## Release verification, recorded here rather than claimed

The measurements behind the record are in `docs/audits/impl-PMAT-258-receipt.md` under "Released": the sparse index, the GitHub release, the install, and three behaviours checked against the installed 7.3.0 binary — SC2009 firing where SC2106 used to, SC2106 firing on a subshell break, and `s.len()` emitting `${#s}` and printing 5.

## Landed in

This ticket spans two pull requests, and a reviewer judging either one alone should know which:

| pull request | what it carried |
|---|---|
| #327 | the close-out itself: statuses for PMAT-258, 259, 260 and 261; the verified release record appended to `docs/audits/impl-PMAT-258-receipt.md`; the v7.3.0 sections dated in the plan and the notes; PMAT-253 and PMAT-256 moved to v7.4.0; the v7.4.0 sections opened |
| #328 | the correction: PMAT-262 itself moved from v7.4.0 to v7.3.0 and marked completed, since a close-out ships with the release it closes, and the v7.4.0 plan and notes returned to the two entries actually open there |

Three lanes refused #328 on the first round because the claims above read as if this diff performed all of them. They did not; #327 did. The table is the fix, and it is the same lesson as the three refusals earlier in the release: a reviewer reads a diff against what the ticket says, so the ticket has to say which diff.
