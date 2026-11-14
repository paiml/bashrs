#!/bin/sh
# test-no-pull-request-target-guard.sh -- the guard must go red on each bad form and green on clean ones.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
guard="$here/no-pull-request-target-guard.sh"
tmp=$(mktemp -d)
[ -n "$tmp" ] && [ -d "$tmp" ] || exit 2
trap 'rm -rf "${tmp:?}"' EXIT
fail=0

# expect <name> <want-rc> <dir> [allowlist]
expect() {
  name=$1 want=$2 d=$3 al=${4:-$tmp/none}
  rc=0
  sh "$guard" "$d" "$al" >/dev/null 2>&1 || rc=$?
  if [ "$rc" -ne "$want" ]; then
    echo "FAIL $name: want rc=$want got rc=$rc"
    fail=$((fail + 1))
  else
    echo "ok   $name"
  fi
}

mk() { mkdir -p "$tmp/$1"; }

mk clean;   printf 'on:\n  pull_request:\n    types: [opened]\n' > "$tmp/clean/a.yml"
mk map;     printf 'on:\n  pull_request_target:\n    types: [opened]\n' > "$tmp/map/a.yml"
mk inline;  printf 'on: [push, pull_request_target]\n' > "$tmp/inline/a.yml"
mk expr;    printf "concurrency:\n  x: \${{ github.event_name == 'pull_request_target' }}\n" > "$tmp/expr/a.yaml"
mk comment; printf '# pull_request_target is banned\non:\n  pull_request:\n' > "$tmp/comment/a.yml"
mk empty
mk allowed; printf 'on:\n  pull_request_target:\n' > "$tmp/allowed/closer.yml"
mk allowco; printf 'on:\n  pull_request_target:\njobs:\n  j:\n    steps:\n      - uses: actions/checkout@v4\n' > "$tmp/allowco/closer.yml"
printf 'closer.yml  # metadata only\n' > "$tmp/allow.txt"

expect clean-passes            0 "$tmp/clean"
expect map-form-fails          1 "$tmp/map"
expect inline-form-fails       1 "$tmp/inline"
expect expression-form-fails   1 "$tmp/expr"
expect comment-only-passes     0 "$tmp/comment"
expect zero-files-not-a-pass   2 "$tmp/empty"
expect missing-dir-is-usage    2 "$tmp/nope"
expect allowlisted-passes      0 "$tmp/allowed" "$tmp/allow.txt"
expect allowlisted-checkout    1 "$tmp/allowco" "$tmp/allow.txt"
expect not-allowlisted-fails   1 "$tmp/allowed"

# PR #491 blocker 2: a `#` inside a value must not hide the trigger, and a
# double-quoted YAML escape or escaped line break must not spell it past the match.
# Fixtures are written with quoted heredocs so every backslash reaches the file as-is.
mkf() { mkdir -p "$tmp/$1"; cat > "$tmp/$1/$2"; }
mkf hashflow  a.yml <<'EOF'
on: { push: { branches: [feat#1] }, pull_request_target: {} }
EOF
mkf hashquote a.yml <<'EOF'
on: { push: { branches: ["v1#"] }, pull_request_target: {} }
EOF
mkf hashname  a.yml <<'EOF'
{ name: "PR #1", on: pull_request_target }
EOF
mkf esc-x     a.yml <<'EOF'
on:
  "pull_request_\x74arget":
EOF
mkf esc-xup   a.yml <<'EOF'
on:
  "pu\x6Cl_request_target":
EOF
mkf esc-u     a.yml <<'EOF'
on:
  "pull_request_\u0074arget":
EOF
mkf esc-U     a.yml <<'EOF'
on:
  "pull_request_\U00000074arget":
EOF
mkf esc-nl    a.yml <<'EOF'
on:
  ? "pull_request_\
    target"
  : {}
EOF
mkf trailcom  a.yml <<'EOF'
on:
  pull_request: # never pull_request_target
EOF
expect hash-in-flow-value-fails    1 "$tmp/hashflow"
expect hash-in-quoted-value-fails  1 "$tmp/hashquote"
expect hash-in-name-fails          1 "$tmp/hashname"
expect escape-x-fails              1 "$tmp/esc-x"
expect escape-x-upper-hex-fails    1 "$tmp/esc-xup"
expect escape-u-fails              1 "$tmp/esc-u"
expect escape-U-fails              1 "$tmp/esc-U"
expect escaped-line-break-fails    1 "$tmp/esc-nl"
expect trailing-comment-fails-closed 1 "$tmp/trailcom"

# the same truncation and spellings on the allowlisted-checkout test
mkf alhash closer.yml <<'EOF'
on: pull_request_target
jobs: { j: { steps: [ { name: "a#b", uses: actions/checkout@v4 } ] } }
EOF
mkf alesc  closer.yml <<'EOF'
on: pull_request_target
jobs:
  j:
    steps:
      - uses: "actions/\x63heckout@v4"
EOF
expect allowlisted-hash-checkout-fails    1 "$tmp/alhash" "$tmp/allow.txt"
expect allowlisted-escaped-checkout-fails 1 "$tmp/alesc"  "$tmp/allow.txt"
# `.` in an allowlist line is a literal, not a regex wildcard
printf 'closerXyml  # not closer.yml\n' > "$tmp/allow-dot.txt"
expect allowlist-dot-is-literal 1 "$tmp/allowed" "$tmp/allow-dot.txt"

# PR #491 blocker 3: an entry the guard cannot read is not scanned-and-clean.
mkf unread b.yml <<'EOF'
on: push
EOF
mkdir -p "$tmp/unread/a.yml"
mkf dangling b.yml <<'EOF'
on: push
EOF
ln -s "$tmp/does-not-exist.yml" "$tmp/dangling/a.yml"
expect unreadable-entry-is-error 2 "$tmp/unread"
expect dangling-link-is-error    2 "$tmp/dangling"

# PR #491 blocker 1: every workflow that runs the guard must fire on a push to an
# open PR (synchronize) and on a push to main, or a later commit is never scanned.
callers=0
for wf in "$here/../../.github/workflows"/*.yml "$here/../../.github/workflows"/*.yaml; do
  [ -f "$wf" ] || continue
  grep -q 'no-pull-request-target-guard\.sh' "$wf" || continue
  callers=$((callers + 1))
  # the top-level `on:` block, up to the next top-level key
  trig=$(sed -n '/^on:/,/^[a-z]/p' "$wf" | sed '$d')
  name=$(basename "$wf")
  if printf '%s\n' "$trig" | grep -q '^  pull_request:' &&
     { ! printf '%s\n' "$trig" | grep -q 'types:' || printf '%s\n' "$trig" | grep -q 'synchronize'; } &&
     printf '%s\n' "$trig" | grep -q '^  push:' &&
     printf '%s\n' "$trig" | sed -n '/^  push:/,/^  [a-z]/p' | grep -q 'main'; then
    echo "ok   trigger-$name"
  else
    echo "FAIL trigger-$name: guard does not run on synchronize and on push to main"
    fail=$((fail + 1))
  fi
done
if [ "$callers" -eq 0 ]; then echo "FAIL trigger: no workflow runs the guard"; fail=$((fail + 1)); fi

# the repo's own workflows must be clean
repo=$(cd "$here/../.." && pwd)
if [ -d "$repo/.github/workflows" ]; then
  rc=0
  sh "$guard" "$repo/.github/workflows" "$repo/.github/pull-request-target-allowlist.txt" >/dev/null 2>&1 || rc=$?
  if [ "$rc" -ne 0 ]; then echo "FAIL repo-workflows: rc=$rc"; fail=$((fail + 1)); else echo "ok   repo-workflows"; fi
fi

[ "$fail" -eq 0 ] || { echo "$fail failure(s)"; exit 1; }
echo "all passed"
