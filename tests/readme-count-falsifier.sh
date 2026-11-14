#!/bin/sh
# readme-count-falsifier — the two contract counts README.md prints are each
# true of the source it names (bashrs#518 review, blocker 1).
#
# README.md prints two numbers. The CONTRACT_COUNT block is pv-readme's walked
# count: regular *.yaml files under contracts/, symlinks not followed. The
# frontmatter `contract_count:` is contracts/census.json `n_files`, which also
# counts the symlinked contracts. `pv-readme --check` grades the numbers, not
# the sentence that says where they come from; this script grades both, with
# no pv-readme needed.
#
# It grades the real README.md (must pass), then plants three defects in a
# copy and requires each to be refused.
#
# Run from the repository root:  sh tests/readme-count-falsifier.sh
# Exit: 0 every case behaved; 1 a case did not; 2 it could not measure.
set -u

[ -f README.md ] && [ -f contracts/census.json ] || {
    echo "NO-GO readme-count-falsifier: run from the repository root" >&2
    exit 2
}

# The walk pv-readme --write renders (aprender readme_cmd.rs is_contract_file):
# regular *.yaml files, not dot-files, not binding/external-corpora/ontology,
# not under legacy/ pipelines/ publish-manifests/ quarantine/.
walked=$(find contracts -type f -name '*.yaml' ! -name '.*' \
    ! -name binding.yaml ! -name external-corpora.yaml ! -name ontology.yaml \
    ! -path '*/legacy/*' ! -path '*/pipelines/*' \
    ! -path '*/publish-manifests/*' ! -path '*/quarantine/*' | wc -l | tr -d ' ')
linked=$(find contracts -type l -name '*.yaml' | wc -l | tr -d ' ')
census=$(sed -n 's/^[[:space:]]*"n_files":[[:space:]]*\([0-9][0-9]*\),*$/\1/p' contracts/census.json)
for n in "$walked" "$linked" "$census"; do
    case $n in
        '' | *[!0-9]*)
            echo "NO-GO readme-count-falsifier: walked='$walked' linked='$linked' census='$census'" >&2
            exit 2
            ;;
    esac
done
failed=0
cases=0

# grade <file>: print the first defect, or nothing.
grade() {
    g=$1
    block=$(sed -n 's/.*<!-- CONTRACT_COUNT_START -->\([^<]*\)<!-- CONTRACT_COUNT_END -->.*/\1/p' "$g" | head -n 1)
    front=$(sed -n '2,/^---$/s/^contract_count: //p' "$g" | head -n 1)
    section=$(sed -n '/^## Contracts$/,/^## /p' "$g" | tr '\n' ' ' | tr -s ' ')
    if [ "$block" != "$walked" ]; then
        echo "block states '$block', the walk is $walked"
    elif [ "$front" != "$census" ]; then
        echo "frontmatter contract_count is '$front', census n_files is $census"
    else
        case $section in
            *"rendered from \`contracts/census.json\`"* | *"count is rendered from contracts/census.json"*)
                if [ "$block" != "$census" ]; then
                    echo "the block is attributed to contracts/census.json, which says $census, not $block"
                    return
                fi
                ;;
        esac
        case $section in
            *"$linked symlinked"*) ;;
            *) echo "the Contracts section does not say the $linked symlinked contracts are left out of the walk" ;;
        esac
    fi
}

# expect <want: pass|refuse> <label> <file>
expect() {
    cases=$((cases + 1))
    why=$(grade "$3")
    if [ "$1" = pass ] && [ -n "$why" ]; then
        echo "FAIL  $2: $why"
        failed=$((failed + 1))
    elif [ "$1" = refuse ] && [ -z "$why" ]; then
        echo "FAIL  $2: not refused"
        failed=$((failed + 1))
    else
        echo "ok    $2${why:+ ($why)}"
    fi
}

work=$(mktemp -d) || exit 2
[ -n "$work" ] && [ -d "$work" ] && [ "$work" != "/" ] || exit 2
trap '[ -n "$work" ] && [ "$work" != "/" ] && rm -rf "$work"' EXIT
trap 'exit 2' INT TERM

# plant <name> <sed-expr>: a copy of README.md with one change, or NO-GO.
plant() {
    sed "$2" README.md >"$work/$1.md"
    if cmp -s README.md "$work/$1.md"; then
        echo "NO-GO readme-count-falsifier: the $1 plant changed nothing" >&2
        exit 2
    fi
}

# 0. The committed README passes. Without this, the refusals prove nothing.
expect pass "the committed README.md states both counts and their sources" README.md

# 1. The block set to a count the walk does not produce.
plant block "s|<!-- CONTRACT_COUNT_START -->[^<]*<!-- CONTRACT_COUNT_END -->|<!-- CONTRACT_COUNT_START -->$((walked + 1))<!-- CONTRACT_COUNT_END -->|"
expect refuse "a block that is not the walk is refused" "$work/block.md"

# 2. The frontmatter set to a count the census does not hold.
plant front "s|^contract_count: .*$|contract_count: $((census + 1))|"
expect refuse "a frontmatter count that is not the census is refused" "$work/front.md"

# 3. The sentence that shipped in 3e1d2668: the block attributed to census.json.
# It is false only while the walk and the census differ.
plant source 's|^## Contracts$|## Contracts\
\
The count is rendered from `contracts/census.json` by `pv-readme --write`.|'
if [ "$walked" -ne "$census" ]; then
    expect refuse "the block attributed to census.json, which differs, is refused" "$work/source.md"
else
    expect pass "the block attributed to census.json, which agrees, passes" "$work/source.md"
fi

echo "readme-count-falsifier: $cases case(s) run, $failed failed (walk $walked, symlinked $linked, census $census)"
[ "$cases" -eq 4 ] || {
    echo "NO-GO readme-count-falsifier: ran $cases case(s), want 4" >&2
    exit 2
}
[ "$failed" -eq 0 ]
