---
name: adversarial-crux-pr
description: The gate every bashrs lint-rule change passes before a PR is opened. Two halves — an ADVERSARIAL review that tries to break the fix by hunting the false negative it may have introduced, and a CRUX competitive analysis that checks the fix against how shellcheck, mvdan/sh and tree-sitter-bash solve the same problem. Invoke it when the user says "adversarial-crux", "harden this fix", "gate this PR", "crux this lint change", or before opening ANY PR that touches a lint rule, the lexer, or a diagnostic code. NOT a code formatter and NOT a general reviewer — it asks two questions and refuses to pass a fix that cannot answer them.
---

# adversarial-crux-pr — the gate before the PR

Two questions, asked of a lint fix before anyone is allowed to open a pull
request for it:

> **1. What does this fix now FAIL to catch?**
> **2. How does the field solve this, and are we diverging on purpose?**

A linter earns its place by being trusted when it is red. Every fix to a false
positive moves the rule's boundary, and a boundary can be moved too far. This
skill exists because the cheapest way to make a noisy rule quiet is to make it
wrong in the other direction, and nothing in a normal review catches that.

```bash
# from the bashrs checkout, on the branch carrying the fix
bash .claude/skills/adversarial-crux-pr/gate.sh --rule SC1078 --corpus ~/src/rmedia/scripts
bash .claude/skills/adversarial-crux-pr/gate.sh --self-test     # prove the gate can fail
```

---

## The exit code says whether the GATE worked, not whether the fix is good

```
exit 0   the gate RAN and the fix survived both halves. Open the PR.
exit 1   the fix FAILED a half. Findings below. Do not open the PR.
exit 2   the GATE could not run. Nothing was measured. This is not a pass.
```

`exit 2` is the one people get wrong. An absent shellcheck, an unreadable
corpus, a `cargo build` failure — none of those are a clean bill of health.
A gate that cannot measure must never resolve to "fine".

---

## Half one — ADVERSARIAL: hunt the false negative

**This half outranks everything else in the skill.** A false positive is noise;
a false negative is a defect shipped under a green check. Trading the first for
the second is a net loss even when the count goes down.

### The rule: every rule you touch keeps a must-still-fire case

For each rule the change affects, the PR must carry a test whose *only* job is
to prove the rule still fires on the genuine defect it exists for. Not "the
tests pass" — a specific case that goes red if the rule goes silent.

The pairs that matter for the lexer family, and why each is easy to break while
fixing its false positive:

| rule | false positive being fixed | what it must STILL catch |
|---|---|---|
| `SC1078` | a `"` legally spanning newlines (`python3 -c "…"`) | a genuinely unterminated `"` running to EOF |
| `SC1128` | a `#!` inside a heredoc writing a fixture | a real shebang on line 5 of the script itself |
| `SC2188` | `<svg …>` inside a heredoc body | a real `> out.txt` with no command |
| `SC1028` / `SC2104` | parens/brackets inside a quoted string | real unescaped parens inside a real `[ ]` |
| `SC2154` | a var assigned in a `;`-separated list | a genuinely unassigned variable |
| `SC1035` | `for` matching inside `git for-each-ref` | a real `forx` with a genuinely missing space |

Teach the lexer to *know* it is inside a quote or a heredoc. Do not teach the
rule to skip lines that look difficult.

### Construct the attack, do not imagine it

Write the broken shell. Run the built binary against it. Paste the output.
"The rule should still fire" is not evidence; a red test is.

### Read the diff, not the summary

A fix can be a suppression in disguise. Look specifically for:

- a rule disabled, or its severity lowered so the count drops
- a file-pattern or path exclusion that happens to cover the corpus
- an early `return` on a condition the corpus always satisfies
- a threshold moved to exactly the current measurement
- the corpus added to an allowlist

If the honest conclusion is that a rule cannot be made accurate, **disabling it
is a legitimate outcome** — stated explicitly, with the reason. What is not
legitimate is a rule that still claims to check something it no longer checks.

### Differential-test against the reference implementation

`shellcheck` owns the `SCxxxx` namespace. For any input where bashrs and
shellcheck now disagree on a shared code, the PR needs a written justification.
Silent divergence on a borrowed identifier is how a code stops meaning anything.

Run all three, because they disagree in informative ways:

```bash
bashrs lint "$f"          # us
shellcheck -S error "$f"  # the reference for SCxxxx
bash -n "$f"              # the only definition that actually runs
```

### The signal you are protecting

Count the findings you must NOT lose, before and after, and print both. On the
rmedia corpus that is `SEC011`, `SEC010` and `DET002` — the families that found
a real `rm -rf` hazard through a symlink where a source and its destination
resolved to one directory. If the error count falls partly because those went
quiet, the fix has destroyed the reason to run the tool.

**Print the denominator.** "0 false positives" over 0 scripts scanned is the
oldest way to look clean.

---

## Half two — CRUX: how does the field solve this?

Competitive analysis is not flattery in either direction. Its job is to stop
bashrs re-deriving a solved problem badly, and to make a deliberate divergence
legible as a decision rather than an accident.

### The comparands, and what each is good for

| tool | language / approach | ask it about |
|---|---|---|
| **shellcheck** | Haskell, parser combinators | how a heredoc body and a multi-line `"` are *represented*; it is the reference for `SCxxxx` |
| **mvdan/sh** | Go, full bash parser with a documented AST | how a real AST models command lists, quoting, and word boundaries |
| **tree-sitter-bash** | incremental GLR grammar | how editors get partial/broken input right without a full parse |
| **clippy** | Rust, non-shell | lint *namespacing* and cross-tool suppression — the model for avoiding code collisions |

### What the write-up must answer, per fix

1. **What does the field do?** Mechanism, not marketing. "shellcheck handles it"
   is not an answer; *how it represents the construct* is.
2. **What does bashrs do today?**
3. **Does this fix match the field's approach, or diverge?**
4. **Where divergence is defensible, and where it is a mistake we should not
   repeat.**

Be honest where bashrs is genuinely better, and where a competitor carries the
same bug. The point is a correct fix, not a favourable comparison.

### Namespace collisions are a CRUX finding, not a cosmetic one

When bashrs emits an `SCxxxx` code whose meaning differs from shellcheck's, three
things break at once, and none of them are style:

1. `# shellcheck disable=SCxxxx` in a shared codebase suppresses the wrong
   diagnostic in one tool or the other.
2. Any baseline, ratchet or dashboard keyed on code numbers silently conflates
   two checks.
3. A user who looks the code up reads documentation for something else.

`SEC###`, `DET###` and `IDEM###` already namespace correctly. That is the model.

**Renaming a code is a breaking change**, and its cost grows every day it ships:
each new baseline written against the wrong meaning is another thing that breaks
at reconciliation. When this half finds a collision, say so with the deadline
shape attached — volume argues for fixing noisy rules first, irreversibility
argues for fixing the namespace first, and irreversibility wins.

Migration is part of the fix, not a follow-up: accept the old code as a
deprecated alias in suppression pragmas, so an existing baseline does not
*silently* stop suppressing on upgrade.

---

## Running it as a gate

This runs **before** the PR is opened, not as a review comment afterwards. A
gate that runs after the PR is a gate that gets argued with.

```
1. reproduce      the reported false positive, from the named file and line
2. fix            the cause, not the symptom
3. ADVERSARIAL    construct the false negative; prove the rule still fires
4. differential   bashrs vs shellcheck vs bash -n
5. CRUX           how the field solves it; where we diverge and why
6. denominators   corpus count before/after, and the SEC/DET counts unchanged
7. THEN open the PR, with 3-6 in the body
```

A PR body that does not contain the must-still-fire evidence and the CRUX
comparison has not passed this gate, whatever the CI says.

---

## Why this exists

Measured on a 45-script production corpus at bashrs 6.67.0:

```
bashrs lint            150 error-severity findings
shellcheck -S error      0
bash -n                  0
```

All 150 were false positives, clustered into six lexer and namespace defects.
The cost was not the noise. The cost was that `SEC011` had found a real
destructive-`rm` hazard in that same corpus, and a consumer running the tool as
a gate had to treat every red as *"look, don't obey"* — which is the state in
which a gate stops being a gate.

This skill is the thing that keeps the next fix from buying quiet with blindness.
