# PMAT-254 triage policy (bashrs 7.0.4 → v7.1.0)
## Verdict vocabulary (the ledger row's `verdict`)
- A  fix in v7.1.0 -> label `release:7.1.0`, milestone v7.1.0
- B  fix in v7.2.0 -> label `release:7.2.0`
- C  backlog -> label `release:backlog`, milestone Backlog
- D  close with evidence: the ask is satisfied by something MEASURED (name it)
- E  already fixed on 7.0.4: the issue's own reproducer is clean on the 7.0.4 binary
Every row carries the measurement that decided it. Closes go through mutate.sh close with a quorum artifact only.
