#!/usr/bin/env bash
# adversarial-crux-pr — the gate a bashrs lint fix passes before its PR exists.
#
# Two halves, and the first one outranks everything else here:
#
#   ADVERSARIAL  does the fix still catch the defect the rule exists for?
#   CRUX         does the fix match how the field solves it, or diverge on purpose?
#
# A false positive is noise. A false negative is a defect shipped under a green
# check. Trading the first for the second is a net loss even when the count
# falls, and nothing in an ordinary review catches it — the diff looks like an
# improvement and the test suite goes green.
#
# EXIT CODES, and the third is the one people get wrong:
#
#   0  the gate RAN and the fix survived. Open the PR.
#   1  the fix FAILED a half. Do not open the PR.
#   2  the GATE could not run. NOTHING was measured. This is not a pass.
#
# An absent shellcheck, an unreadable corpus or a failed build are all exit 2.
# A gate that cannot measure must never resolve to "fine" — this fleet has been
# bitten repeatedly by a checker that reported a result it had not measured.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
FIXTURES="$HERE/fixtures"

RULE=""
CORPUS=""
SELFTEST=0

while [ $# -gt 0 ]; do
    case "$1" in
        --rule)      RULE="${2:-}"; shift 2 ;;
        --corpus)    CORPUS="${2:-}"; shift 2 ;;
        --self-test) SELFTEST=1; shift ;;
        -h|--help)
            printf 'usage: gate.sh [--rule CODE] [--corpus DIR] [--self-test]\n'
            exit 0 ;;
        *) printf 'gate: unknown argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

# ── The instrument must be present before it may report anything ─────────────
#
# Each of these is a NO-GO rather than a skip. "shellcheck was not installed so
# we did not differential-test" is exactly the shape that lets a divergence
# ship: the check is absent, the output is green, and nobody can tell those
# apart afterwards.
need() {
    command -v "$1" >/dev/null 2>&1 && return 0
    printf '  NO-GO: %s is not installed, so %s\n' "$1" "$2"
    printf '  This is not a pass.\n'
    exit 2
}
need bashrs     "the fix under test cannot be exercised."
need shellcheck "the SCxxxx namespace has no reference to differ against."
need bash       "nothing can confirm the corpus is valid shell."

printf '== adversarial-crux-pr ==\n'
printf '   bashrs     %s\n' "$(bashrs --version 2>/dev/null | head -1)"
printf '   shellcheck %s\n' "$(shellcheck --version 2>/dev/null | awk '/version:/{print $2}')"

# ── HALF ONE: the must-still-fire corpus ─────────────────────────────────────
#
# Every fixture here is GENUINELY BROKEN shell. Each names the rule that must
# flag it. If a fix quiets one of these, it has bought silence with blindness
# and the gate refuses the PR.
#
# These are deliberately NOT the false-positive cases. A fix is judged on what
# it still catches, not on what it stopped complaining about.
fired=0
missed=0
checked=0

must_fire() {
    _mf_file="$1"
    _mf_rule="$2"
    _mf_why="$3"
    checked=$((checked + 1))
    if [ ! -f "$_mf_file" ]; then
        printf '  NO-GO: fixture missing: %s\n' "$_mf_file"
        printf '  The gate cannot judge a fix against a corpus it does not have.\n'
        exit 2
    fi
    # NOT `bashrs ... | grep -q`: grep -q closes the pipe on its first match,
    # SIGPIPEs bashrs, and under `set -o pipefail` the pipeline returns THAT —
    # so a PASSING check reads as a failure. Capture first, then match.
    _mf_out=$(bashrs lint "$_mf_file" 2>/dev/null)
    if printf '%s' "$_mf_out" | grep -q "$_mf_rule"; then
        printf '  ok    %-8s still fires — %s\n' "$_mf_rule" "$_mf_why"
        fired=$((fired + 1))
    else
        printf '  LOST  %-8s NO LONGER FIRES on %s\n' "$_mf_rule" "$_mf_file"
        printf '        %s\n' "$_mf_why"
        printf '        A false positive was traded for a FALSE NEGATIVE.\n'
        missed=$((missed + 1))
    fi
}

printf '\n-- half one: ADVERSARIAL — what does the fix no longer catch? --\n'

must_fire "$FIXTURES/unterminated-quote.sh"  SC1078 "a quote that genuinely runs to EOF"
must_fire "$FIXTURES/late-shebang.sh"        SC1128 "a real shebang that is genuinely not on line 1"
must_fire "$FIXTURES/bare-redirect.sh"       SC2188 "a real redirection with no command"
must_fire "$FIXTURES/unassigned-var.sh"      SC2154 "a variable genuinely never assigned"

printf '\nmust-still-fire: %s checked, %s fired, %s LOST\n' "$checked" "$fired" "$missed"

if [ "$checked" -eq 0 ]; then
    printf 'NO-GO: no fixture was exercised. A gate that inspected nothing is not a pass.\n'
    exit 2
fi

# ── The self-test proves the gate itself can fail ────────────────────────────
#
# A green never shown capable of red is not evidence. --self-test asserts the
# instrument fires on a fixture built to trip it; if that comes back clean, the
# gate is broken and must not be used to clear a PR.
if [ "$SELFTEST" -eq 1 ]; then
    printf '\n-- self-test: the gate must be able to FAIL --\n'
    # Same SIGPIPE trap as above: capture, then match.
    _st_out=$(bashrs lint "$FIXTURES/unterminated-quote.sh" 2>/dev/null)
    if printf '%s' "$_st_out" | grep -q SC1078; then
        printf '  ok    the instrument fires on a known-broken fixture\n'
    else
        printf '  NO-GO: the instrument did not fire on a fixture built to trip it.\n'
        printf '  Refusing to clear any PR with a checker that cannot go red.\n'
        exit 2
    fi
fi

# ── HALF TWO: the differential, over a real corpus ───────────────────────────
#
# The SCxxxx namespace is owned by shellcheck, and `bash -n` is the only
# definition that actually runs. Where bashrs reports an error both of them
# accept, that is a false-positive CANDIDATE — not proof, but the place to look.
#
# (This paragraph is deliberately NOT written as "# shellcheck owns …": a
# comment beginning `# shellcheck ` is parsed as a DIRECTIVE, and shellcheck
# rejected the prose with SC1073 "Couldn't parse this shellcheck directive".
# Found by dogfooding this gate against shellcheck.)
fp_candidates=0
scanned=0
sec_kept=0

if [ -n "$CORPUS" ]; then
    printf '\n-- half two: DIFFERENTIAL over %s --\n' "$CORPUS"
    if [ ! -d "$CORPUS" ]; then
        printf '  NO-GO: corpus is not a directory: %s\n' "$CORPUS"
        exit 2
    fi
    for f in "$CORPUS"/*.sh; do
        [ -e "$f" ] || continue
        scanned=$((scanned + 1))
        b=$(bashrs lint "$f" 2>/dev/null | grep -c '\[error\]')
        s=$(shellcheck -S error "$f" 2>/dev/null | grep -c '^' )
        bash -n "$f" 2>/dev/null; n=$?
        # SEC/DET are the families that justify running the tool at all. Count
        # them separately so a falling error total can never hide them going out.
        k=$(bashrs lint "$f" 2>/dev/null | grep -cE '(SEC|DET|IDEM)[0-9]+')
        sec_kept=$((sec_kept + k))
        if [ "${b:-0}" -gt 0 ] && [ "${s:-0}" -eq 0 ] && [ "$n" -eq 0 ]; then
            fp_candidates=$((fp_candidates + b))
        fi
    done
    # THE DENOMINATOR. "0 false positives" over 0 scripts is the oldest way to
    # look clean, and this fleet's most-repeated defect.
    printf '\ndifferential: %s script(s) scanned\n' "$scanned"
    printf '              %s bashrs error(s) that BOTH shellcheck and bash -n accept\n' "$fp_candidates"
    printf '              %s SEC/DET/IDEM finding(s) still reported\n' "$sec_kept"
    if [ "$scanned" -eq 0 ]; then
        printf 'NO-GO: no script was scanned — is the corpus path right?\n'
        exit 2
    fi
    if [ "$sec_kept" -eq 0 ]; then
        printf '\nNO-GO: not one SEC/DET/IDEM finding in the whole corpus.\n'
        printf 'Those families are the reason to run this tool. Either the corpus is\n'
        printf 'wrong or a fix has silenced them; the gate will not guess which.\n'
        exit 2
    fi
fi

# ── Verdict ──────────────────────────────────────────────────────────────────
printf '\n'
if [ "$missed" -gt 0 ]; then
    printf 'FAIL: %s rule(s) stopped firing on a genuine defect.\n' "$missed"
    printf 'A quieter linter that misses real bugs is worse than a noisy one.\n'
    printf 'Do not open the PR.\n'
    exit 1
fi

printf 'OK: every touched rule still fires on the defect it exists for.\n'
if [ -n "$CORPUS" ] && [ "$fp_candidates" -gt 0 ]; then
    printf 'NOTE: %s false-positive candidate(s) remain in this corpus — findings,\n' "$fp_candidates"
    # This line used to read `... not this PR%s problem.\n' "'s"`, written that
    # way to route an apostrophe around a known bashrs bug. bashrs 6.67.0
    # reported SC1078 on it anyway — "did you forget to close this
    # double-quoted string?" — and was wrong; the string was closed. The T4
    # false positive this gate exists to retire, fired on the gate itself.
    #
    # It is now plain prose with no apostrophe, so the specimen is GONE from
    # this file. That is a workaround, not a fix, and it is recorded here rather
    # than left silent: the bug is T4, tracked in bashrs-tickets.md, and the
    # honest home for a live specimen is a test fixture that must go green when
    # T4 lands — not a comment claiming something the code no longer does.
    printf 'not a gate failure. They are the next ticket, not this one.\n'
fi
printf '\nStill required in the PR body, and NOT checkable here:\n'
printf '  - the CRUX comparison: how shellcheck / mvdan/sh / tree-sitter-bash\n'
printf '    solve this, and where this fix diverges on purpose\n'
printf '  - a written justification for any SCxxxx divergence from shellcheck\n'
