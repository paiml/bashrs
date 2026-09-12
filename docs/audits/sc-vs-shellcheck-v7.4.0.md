# bashrs 7.4.0 against shellcheck 0.8.0, on the sample issue #236 measured

Re-measurement for the v7.4.0 close-out (PMAT-264). Issue #236 measured bashrs
6.67.0 against shellcheck 0.8.0 over the tracked `*.sh` files of
`paiml/paiml-mcp-agent-toolkit` and reported `SC2046` at 210:1. This is the same
comparison with the released 7.4.0 binary, run 2026-09-12.

## Method

```
bashrs 7.4.0 (release build of 4aa24ca271)   shellcheck 0.8.0
for f in $(git ls-files '*.sh'); do
  bashrs lint --format json "$f" | jq '.diagnostics[]?'      # 97 files
  shellcheck -f json1 "$f"      | jq '.comments[]?'
done
```

**The file set is not the issue's.** The issue counted 40 tracked `*.sh`; the
repository now tracks 97, so the absolute counts are not comparable with the
issue's table and only the per-code shape is. shellcheck reports 130
diagnostics in total over these 97 files.

## Per code, the codes the issue named

| code | bashrs 6.67.0 (40 files) | bashrs 7.4.0 (97 files) | shellcheck 0.8.0 (97 files) |
|---|---|---|---|
| SC2046 | 210 | **72** | 5 |
| SC2086 | 97 | **316** | 7 |
| SC1004 | 75 | **92** | 0 |
| SC1078 | 40 | **0** | 0 |
| SC2183 | 27 | **0** | 0 |
| SC2047 | 23 | **20** | 0 |
| SC1009 | 21 | **0** | 0 |
| SC2164 | 21 | **26** | 0 |
| SC1028 | 19 | **0** | 0 |
| SC2161 | 18 | **19** | 0 |
| SC2036 | 10 | **0** | 0 |
| SC2101 | 10 | **14** | 0 |
| SC2154 | 10 | **108** | 0 |

Five of the thirteen codes the issue named are now **zero**: SC1078, SC1009,
SC1028, SC2036 and SC2183. Those are the lexer-context class fixed across
v7.1.0–v7.3.0 (PMAT-248, PMAT-250, PMAT-255, PMAT-257).

## What is left, largest first, with a measured example each

1. **SC2086 inside a heredoc body** — 316 reports. A heredoc body is data, not
   code; `$TIMESTAMP` inside one is not a word the shell splits at that point.
   Reproducer (bashrs reports SC2086 at line 4, shellcheck reports nothing):
   ```sh
   #!/bin/sh
   TIMESTAMP=x
   cat <<EOF
   **Time**: $TIMESTAMP
   EOF
   ```
   This is GH-242's class (a heredoc body reaching a shell rule) for a rule
   that was not in that ticket's list.

2. **SC2154 on a variable a sourced file defines** — 108 reports.
   `lsb_dist="$(. /etc/os-release && echo "$ID")"` draws SC2154 ("referenced
   but not assigned") where shellcheck reports SC1091 ("not following") and
   nothing else: it knows a sourced file may define the name.
   (`get-docker.sh:263`, and 4 more in that file.)

3. **SC1004 across a multi-line quoted argument** — 92 reports, 75 of them in
   one file (`scripts/create-doc-validate-issues.sh`), where a `gh issue
   create` call carries a multi-line `--body` containing escaped backticks.
   Reduced reproducers of the individual constructs (an escaped backtick in a
   double-quoted string, a backslash-continued pipeline, a multi-line quoted
   argument) each draw **no** SC1004, so the trigger is a combination and the
   reduction is the first job of the ticket that fixes it.

## Verdict

#236's headline claim is no longer true as written: SC2046 is 14:1, not 210:1.
The issue stays open because its subject — "bashrs reuses shellcheck's numbering
so users expect comparable semantics" — is still unsatisfied for SC2086,
SC2154 and SC1004, which are now the three largest gaps. The issue is retagged
`release:v7.5.0` with this measurement, and the three classes above are its
work items.
