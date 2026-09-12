# PMAT-262 receipt — the v7.3.0 close-out

## What

Ticket statuses for PMAT-258, 259, 260 and 261; the verified release record appended to the PMAT-258 receipt; the v7.3.0 sections dated in the plan and the notes; PMAT-253 and PMAT-256 moved to v7.4.0, because `release-lint` R3 refuses open work that names a release already cut; and the v7.4.0 sections opened in both documents.

No code changes. `release-lint` reports, with this ticket now completed against v7.3.0 rather than planned on v7.4.0:

```
release-lint: open=2 releases=v7.4.0(2) highest_tag=v7.3.0
```

**That green depends on a local patch.** The upstream `release-lint.sh` skips only `completed`, so it counts the three long-cancelled entries (PMAT-168, PMAT-169, PMAT-240) as open work that must each name a release, and exits non-zero on them. The count it prints therefore differs from the line above by those three; the exact figure is not quoted here because this machine cannot run the unpatched script to measure it. A blind quorum decided on 2026-09-11 that cancelled is closed, and the one-line change is filed upstream as paiml/paiml-mcp-agent-toolkit#1326; the skill bundle has been upgraded twice since and discarded the patch each time. A quorum lane measured exactly this difference from an unpatched environment, which is why it is written down here rather than left as an unqualified "green".


## Why it is its own ticket

Three quorum rounds refused close-out and follow-up diffs in this release because each was judged against the release ticket whose code had already merged. A lane compares the diff with its ticket's stated intent, and a documentation close-out does not implement a release. The rule held every time it was tested, so a close-out gets a ticket of its own from now on, opened at the start of the cycle rather than discovered at the end.

## Release verification, recorded here rather than claimed

The measurements behind the record are in `docs/audits/impl-PMAT-258-receipt.md` under "Released": the sparse index, the GitHub release, the install, and three behaviours checked against the installed 7.3.0 binary — SC2009 firing where SC2106 used to, SC2106 firing on a subshell break, and `s.len()` emitting `${#s}` and printing 5.

## Landed in

This ticket spans two pull requests, and a reviewer judging either one alone should know which:

| pull request | what it carried |
|---|---|
| #327 | the close-out itself: statuses for PMAT-258, 259, 260 and 261; the verified release record appended to `docs/audits/impl-PMAT-258-receipt.md`; the v7.3.0 sections dated in the plan and the notes; PMAT-253 and PMAT-256 moved to v7.4.0; the v7.4.0 sections opened |
| #328 | the corrections: PMAT-262 moved from v7.4.0 to v7.3.0 and marked completed, since a close-out ships with the release it closes; the v7.4.0 plan and notes returned to the two entries actually open there; the PMAT-262 row placed in the v7.3.0 table (an earlier commit on this branch had put it in v7.2.0 by matching the wrong anchor, so the diff against main shows only the net move out of v7.4.0 and into v7.3.0); and PMAT-253 and PMAT-256 removed from the v7.3.0 table, because a released section lists what shipped in it and those two were carried forward |

Lanes refused #328 three times, each time correctly: the claims above read as if that diff had performed the whole close-out, the PMAT-262 row sat in the wrong release's table, and the quoted lint line was the one from before the correction. Each refusal named a file and a line, and each was verified before being fixed. It is the same lesson as the three refusals earlier in the release: a reviewer reads a diff against what the ticket says, so the ticket has to say which diff did what, and say it accurately.
