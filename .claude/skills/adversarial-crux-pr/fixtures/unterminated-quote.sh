#!/usr/bin/env bash
# GENUINELY BROKEN: this quote is never closed and runs to EOF.
# SC1078 must fire here. If a lexer fix quiets it, that fix bought silence
# with blindness.
echo "this string never closes
