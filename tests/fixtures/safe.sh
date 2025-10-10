#!/bin/sh
# This script follows all safety best practices

FILES="$1"
DIR="$2"

# All variables properly quoted
ls "$FILES"
rm -rf "$DIR"

# Command substitution properly quoted
result="$(find . -name '*.txt')"
echo "$result"

# No useless echo
message="$result"
