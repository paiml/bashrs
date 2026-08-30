# The `SCxxxx` namespace belongs to ShellCheck

**Status**: census complete; 36 collisions resolved, 2 deferred with reasons.
**Measured**: 2026-08-30, bashrs 6.68.0 vs shellcheck 0.10.0-era registry read
from shellcheck 0.8.0.

## Why this is not cosmetic

bashrs emits 225 distinct `SCxxxx` codes. `docs/SHELLCHECK-PARITY.md` claimed
three. Nobody had compared the other 222 against ShellCheck's registry, and 36
of them turned out to carry a different check in each tool.

`SC2032` is the sharpest case. It is the single most frequent code bashrs
emits — 461 findings across 45 scripts in the rmedia corpus, 9253 across 1200
scripts on the fleet:

| | |
|---|---|
| ShellCheck `SC2032` | Use own script or `sh -c '..'` to run this from sudo. |
| bashrs `SC2032` | Variable `'NAS_RAW'` assigned in script with shebang. To affect caller, source this script or remove shebang. |

Three things break at once:

1. `# shellcheck disable=SC2032` in a shared codebase suppresses the wrong
   diagnostic in one tool or the other — silent interop breakage.
2. A dashboard, baseline file or ratchet keyed on the number conflates two
   unrelated checks.
3. A user who looks up `SC2032` reads documentation for something else.

## How the census was measured

The oracle is `rash/tests/data/shellcheck-registry.tsv`: ShellCheck's own
message for 190 codes, produced two ways.

- **Corpus harvest** — shellcheck run over 4000 real scripts under `~/src`,
  once per dialect (`sh bash dash ksh busybox`) so shell-gated checks fire,
  with `--enable=all -S style`. For each code the recorded message is the mode.
- **Targeted probes** — a minimal snippet per code a real corpus does not
  contain (`sudo somefunction` for `SC2032`, and so on). Only codes ShellCheck
  actually emitted were recorded.

Two measurement hazards worth writing down, because both silently produced
wrong answers on the first pass:

- **Parallel stdout interleaves.** 16 shellcheck children sharing one stdout
  corrupted the oracle: `SC2248` came back as `acters.` Every child now writes
  to its own file.
- **Absence is not evidence.** A code missing from the registry means "not
  measured", never "ShellCheck leaves this number free". `SC2148` was missing
  purely because every harvest run passed `-s <shell>`, which suppresses it.

The bashrs side is read the same way — from what the tool actually emits, not
from `rule_registry_data*.rs`, whose `name:` field turned out to be a third,
independent claim. `SC2223`'s registry name is "This default case is
unreachable"; the message it emits is "Use 'function name' or 'name()' but not
both"; ShellCheck's is "This default assignment may cause DoS due to globbing."
Three different statements about one number.

## The collisions

Resolved by moving the bashrs-original check into the `BRS####` namespace —
the model `SEC###` / `DET###` / `IDEM###` already follow. Rule logic is
unchanged; only the label moves.

| Code | ShellCheck's check | bashrs' check | Now |
|---|---|---|---|
| `SC1009` | The mentioned syntax error was in this simple command. | Comment here is not a command. Use a no-op `:` if the body is empty | `BRS0001` |
| `SC2036` | If you wanted to assign the output of the pipeline, use a=$(b \| c) . | Quotes in backticks need escaping. Use $( ) instead | `BRS0002` |
| `SC2066` | Since you double quoted this, it will not word split, and the loop will only run once. | Quote variable in [[ ... ]] to prevent globbing and word splitting | `BRS0003` |
| `SC2069` | To redirect stdout+stderr, 2>&1 must be last (or use '{ cmd > file; } 2>&1' to clarify). | To redirect stdout to stderr, use >&2, not 2>&1 (which redirects stderr to stdout) | `BRS0004` |
| `SC2077` | You need spaces around the comparison operator. | Regex variable pattern may word split. Quote for literal match or ensure no spaces | `BRS0005` |
| `SC2081` | [ .. ] can't match globs. Use a case statement. | Expressions don't expand in single quotes, use double quotes for that | `BRS0006` |
| `SC2087` | Quote 'REMOTE_PREFLIGHT' to make here document expansions happen on the server side rather than on the client. | Quote variables in sh -c / bash -c with single quotes or escape with \\$ | `BRS0007` |
| `SC2095` | ssh may swallow stdin, preventing this loop from working properly. | Redirections only apply to the condition command, not the if block. Move redirection after 'fi' to redirect entire block | `BRS0008` |
| `SC2096` | On most OS, shebangs can only specify a single parameter. | Multiple stdout redirections specified. Only the last one will be used | `BRS0009` |
| `SC2104` | In functions, use return instead of break. | Missing space before ] | `BRS0010` |
| `SC2114` | Warning: deletes a system directory. | CRITICAL: rm -rf on root or root-like path is extremely dangerous and likely a bug | `BRS0011` |
| `SC2117` | To run commands as another user, use su -c or sudo. | Unreachable code after '{}' on line {} | `BRS0012` |
| `SC2141` | This backslash is literal. Did you mean `IFS=$'\n'`? | '{cmd}' doesn't read stdin. Consider restructuring the command | `BRS0013` |
| `SC2165` | This nested loop overrides the index variable of its parent. | Subshells don't inherit traps. Use { } or set trap inside subshell | `BRS0014` |
| `SC2183` | This format string has 4 variables, but is passed 1 arguments. | Variable used as command name - potential injection risk | `BRS0015` |
| `SC2223` | This default assignment may cause DoS due to globbing. Quote it. | Use 'function name' or 'name()' but not both for POSIX compatibility | `BRS0016` |
| `SC2224` | This mv has no destination. Check the arguments. | Function '{}' was already defined on line {} | `BRS0017` |
| `SC2227` | Redirection applies to the find command itself. Rewrite to work per action (or move to end). | Redirection before pipe applies to first command only. Reorder if this is unexpected | `BRS0018` |
| `SC2231` | Quote expansions in this for loop glob to prevent wordsplitting, e.g. "$dir"/*.txt . | Quote variables in case expressions to prevent glob expansion | `BRS0019` |
| `SC2233` | Remove superfluous (..) around condition to avoid subshell overhead. | Spaces around operators are fine in arithmetic but unusual. Consider removing for consistency | `BRS0020` |
| `SC2266` | Use \|\| for logical OR. Single \| will pipe. | Prefer [[ ]] over [ ] for regex/glob matching | `BRS0021` |
| `SC2268` | Avoid x-prefix in comparisons as it no longer serves a purpose. | Avoid unnecessary subshells for simple assignments | `BRS0022` |
| `SC2269` | This variable is assigned to itself, so the assignment does nothing. | Use 'read -r' to prevent backslash interpretation | `BRS0023` |
| `SC2282` | Variable names can't start with numbers, so this is interpreted as a command. | Use ${var:?} to fail if variable is unset, rather than defaulting to empty | `BRS0024` |
| `SC2286` | This empty string is interpreted as a command name. Double check syntax (or use 'true' as a no-op). | Consider using mapfile/readarray for reading files into arrays | `BRS0025` |
| `SC2311` | Bash implicitly disabled set -e for this function invocation because it's inside a command substitution. Add set -e; before it or enable inherit_errexit. | Use single quotes for literal strings that don't contain expansions | `BRS0026` |
| `SC2061` | Quote the parameter to -name so the shell won't interpret it. | Quote the tr parameter '{}' to prevent glob expansion: tr '{}' ... | `BRS0027` |
| `SC2235` | Use { ..; } instead of (..) to avoid subshell overhead. | Quote arguments to unalias to prevent word splitting and glob expansion | `BRS0028` |
| `SC2248` | Prefer double quoting even when variables don't contain special characters. | Use [[ ]] instead of [ ] for regex matching with =~ | `BRS0029` |
| `SC2267` | GNU xargs -i is deprecated in favor of -I{} | Use ${var//old/new} instead of sed for simple substitutions | `BRS0030` |
| `SC2283` | Remove spaces around = to assign (or use [ ] to compare, or quote '=' if literal). | Remove extra spaces after ! in test expressions | `BRS0031` |
| `SC2287` | This is interpreted as a command name ending with '/'. Double check syntax. | Use [[ -v var ]] to check if variable is set (cleaner syntax) | `BRS0032` |
| `SC2289` | This is interpreted as a command name containing a linefeed. Double check syntax. | Use ${#var} instead of expr length for string length | `BRS0033` |
| `SC2291` | Quote repeated spaces to avoid them collapsing into one. | Use [[ ! -v var ]] to check if variable is unset (cleaner syntax) | `BRS0034` |
| `SC2292` | Prefer [[ ]] over [ ] for tests in Bash/Ksh. | Use ${var:pos:1} instead of expr substr for extracting single characters | `BRS0035` |
| `SC2294` | eval negates the benefit of arrays. Drop eval to preserve whitespace/symbols (or eval as string). | Use ((...)) instead of let for simple arithmetic assignments | `BRS0036` |

Ten of those — `SC2061`, `SC2235`, `SC2248`, `SC2267`, `SC2283`, `SC2287`,
`SC2289`, `SC2291`, `SC2292`, `SC2294` — were found by the guard, not by the
corpus. They fire rarely enough that 45 real scripts never triggered them,
which is precisely why the enumeration is done by reading the rules directory
rather than by eyeballing a corpus.

## `SC2032`: retired, not renamed

A rename is the right answer for a check that is accurate but misfiled. It is
the wrong answer for a check that reports a defect it has not found — moving
the noise to a new number keeps burying the real findings, which is the whole
cost being paid here.

bashrs' `SC2032` fires on every plain `VAR=value` in every script carrying a
shebang. Its premise — "variables set in an executed script don't affect the
calling shell" — is true, and true of every correct script. It is a property of
shell, not a defect at the flagged line.

It also cannot discriminate. The only case where the premise would matter is a
file meant to be *sourced*; the rule keys on the shebang alone, so it fires on
executed scripts (where there is no defect) and stays silent on shebang-less
files (which is exactly what a sourced file looks like). It is loudest where it
is most certainly wrong.

Measured precision: **0 true positives in 461 firings** on the rmedia corpus,
**0 in 9253** across 1200 fleet scripts under `~/src`.

`SC2032` is therefore removed from dispatch and recorded in
`code_namespace::RETIRED` with that reason. The number is now free for
ShellCheck's actual check, should someone implement it.

## Deferred, with reasons

| Code | Divergence | Why not now |
|---|---|---|
| `SC1035` | ShellCheck: missing space after `!`. bashrs generalised it to any keyword (`for`, `fi`). | Divergent in scope, not subject. The rule is being reworked by the false-positive lane that owns it; renaming underneath that work would collide. |
| `SC2111` | ShellCheck: `function` keyword and `()` together in ksh. bashrs: `function` keyword in sh (ShellCheck files that as `SC2112`/`SC3048`). | Same. |

Both are recorded in `code_namespace::PARITY` with an inline comment saying they
are *deferred*, not *equivalent*. That is a knowing overstatement of the parity
list's meaning and the only one in it; it should be revisited once those lanes
land.

Several codes are **conflations rather than collisions** — bashrs files a real
ShellCheck check under a neighbouring ShellCheck number. They are left alone
because the number still means roughly the right thing, but they are worth
knowing about:

- bashrs `SC2005` emits ShellCheck's `SC2116` message; bashrs `SC2116` emits it too.
- bashrs `SC2038` emits ShellCheck's `SC2044` message (for-loop over `find`).
- bashrs `SC2104` emitted ShellCheck's `SC1020` check ("Missing space before `]`") —
  a duplicate of bashrs' own `SC1020` rule, under a squatted number. Now `BRS0010`;
  merging it into `SC1020` is the follow-up.

## Migration: what breaks, and what does not

Renaming a code is a breaking change for anyone with a suppression pragma or a
baseline keyed on it. Handled deliberately:

**Kept working — `# bashrs disable=` and `.bashrsignore`.** These are bashrs'
own artefacts; a pragma naming `SC2081` unambiguously meant the bashrs check.
Both accept the legacy code as a deprecated alias, so existing baselines keep
suppressing. The `.bashrsignore` half was *not* in the original design — the
before/after corpus differential caught it, showing 19 findings re-appearing in
`depyler`, whose `.bashrsignore` lists `SC2081`.

**Deliberately NOT kept — `# shellcheck disable=`.** That pragma names a
ShellCheck check. Expanding aliases there would re-create the interop bug one
level down: `# shellcheck disable=SC2081` would go on silencing a bashrs check
that has nothing to do with ShellCheck's `SC2081`. A test asserts this stays
broken-on-purpose.

Both directions are covered by
`rash/tests/code_namespace_collisions.rs`.

## The guard

`no_bashrs_check_squats_on_a_shellcheck_code` enumerates every `scNNNN.rs` rule
module **by reading the directory** — deliberately not a hand-maintained list,
because a hand-maintained list is the mechanism that let 225 SC codes drift from
a doc claiming three. A module with no `Diagnostic::new` is skipped: it emits
nothing, so it cannot collide, and the guard fires the day someone implements it.

A rule keeps its `SCxxxx` code only if it is in `PARITY` — a claim that someone
read both messages. A second test stops `PARITY` becoming a place to park
collisions, by failing if a code is both migrated and declared at parity.

All of it was shown capable of failing before being trusted:

| Injected defect | Result |
|---|---|
| Restore `SC2311` to the SC namespace | `no_bashrs_check_squats_on_a_shellcheck_code` RED |
| Park `SC2311` in `PARITY` to quiet the guard | `migration_..._disjoint` RED |
| Let `# shellcheck disable=` expand aliases | `a_shellcheck_pragma_does_not_suppress...` RED |

## Evidence the rename lost nothing

The rename is a bijection, not a filter. Dumping every finding as
`file|line|col|severity|code|message` before and after, then applying
`canonical()` to the "before" set:

```
rmedia (45 scripts)   before 2335 findings -> after 1874;  identical after mapping
                      minus exactly the 461 retired SC2032 findings
SEC 61 -> 61   DET 31 -> 31   IDEM 17 -> 17     all unchanged
errors 36 -> 36                                  unchanged
```

Every span, severity and message is byte-identical; only the label moves.

```
fleet (1200 scripts)  before 44642 findings -> after 35389;  identical after mapping
                      minus exactly the 9253 retired SC2032 findings
SEC 1999 -> 1999   DET 492 -> 492   IDEM 333 -> 333   all unchanged
```

## Reproducing the census

```sh
# ShellCheck's side of the registry (per-process output files — see the note
# in tests/data/shellcheck-registry.tsv about interleaving).
for s in sh bash dash ksh busybox; do
  xargs -a scripts.txt -P 16 -n 20 sh -c \
    "shellcheck -s $s -f gcc -S style --enable=all -e SC1091 \"\$@\" > out/$s.\$\$.txt" _
done

# bashrs' side
bashrs lint --format json <file>

# The guard
cargo test -p bashrs --test code_namespace_collisions
```
