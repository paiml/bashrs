Continue implementing the spec using `pmat work` for task management.

## Workflow

1. **Check `pmat work status`** to find in-progress and planned tasks
2. **Pick the highest-priority unblocked task** and start it with `pmat work start <ID>`
3. **Implement** the task using the spec in `docs/specifications/shell-safety-inference.md`
4. **Push frequently** — commit and push after each meaningful change
5. **Fix every issue** (deep or trivial) using **Five Whys** root cause analysis:
   - Ask "why?" 5 times to find the root cause
   - Fix the root cause, not the symptom
   - Document the five-whys chain in the commit message
6. **Complete tasks** with `pmat work complete <ID>` when acceptance criteria are met
7. **Monitor training** on GB10 (`ssh gx10`) if Run 11d is active — check heartbeat, JSONL, GPU exclusivity
8. **Update docs** — spec version history, roadmap, training config after significant changes
9. **Repeat** — pick the next task and continue

## Principles

- **Jidoka**: Stop the line on any defect. No workarounds.
- **Kaizen**: Continuous improvement. Every commit makes the system better.
- **Genchi Genbutsu**: Go and see. Read the code before changing it. Check the actual training logs.
- **Zero waste**: Don't add features nobody asked for. Don't over-engineer.
- **Parallel work**: Launch agents for independent tasks. Push while waiting.

## Current Context

- **Training**: Run 11d on GB10, 1 epoch (5,543 steps), delta checkpoints, ~8 days ETA
- **Eval pipeline**: `bashrs corpus batch-eval` + `eval-benchmark` ready
- **Spec**: v12.30, Section 17 has the next steps
- **Repos**: bashrs (this), entrenar (training), trueno (GPU kernels)
