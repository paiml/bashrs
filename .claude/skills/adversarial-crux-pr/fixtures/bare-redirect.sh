#!/usr/bin/env bash
# GENUINELY BROKEN: a redirection with no command in front of it.
# SC2188 must fire here.
set -euo pipefail
> out.txt
