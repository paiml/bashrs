#!/usr/bin/env bash
# GENUINELY BROKEN: a second, real shebang below line 1 — not inside any
# heredoc. SC1128 must fire here.
set -euo pipefail
echo hello
#!/bin/sh
