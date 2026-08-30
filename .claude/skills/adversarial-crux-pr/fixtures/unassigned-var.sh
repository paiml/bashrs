#!/usr/bin/env bash
# GENUINELY BROKEN: never_set_anywhere is referenced and never assigned.
# SC2154 must fire here.
set -euo pipefail
echo "$never_set_anywhere"
