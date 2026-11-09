#!/bin/sh
# Fails once any "expires YYYY-MM-DD" waiver in the advisory ignore lists is past.
# cargo-audit and cargo-deny never read these dates, so this is the enforcement.
# WAIVER_TODAY=YYYY-MM-DD overrides the clock (used by the self-test).
set -eu

today="${WAIVER_TODAY:-$(date -u +%Y-%m-%d)}"
case "$today" in
    [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;;
    *) echo "waiver-expiry: bad date '$today'" >&2; exit 2 ;;
esac
[ "$#" -gt 0 ] || set -- .cargo/audit.toml deny.toml

checked=0
expired=0
for f in "$@"; do
    [ -f "$f" ] || { echo "waiver-expiry: missing file $f" >&2; exit 2; }
    for d in $(sed -n 's/.*expires \([0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]\).*/\1/p' "$f"); do
        checked=$((checked + 1))
        # ISO dates compare correctly as strings; expiry day itself is still valid.
        if [ "$today" \> "$d" ]; then
            echo "waiver-expiry: EXPIRED $d (today $today) in $f" >&2
            expired=$((expired + 1))
        fi
    done
done
echo "waiver-expiry: $checked dated waiver(s) checked, $expired expired (today $today)"
[ "$expired" -eq 0 ]
