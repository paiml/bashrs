#!/bin/bash
# Script with known linting issues

# SC2086: Unquoted variable expansion
echo $unquoted_var

# SC2002: Useless cat
cat file.txt | grep pattern

# SC2006: Deprecated backticks
result=`date`

# DET002: Non-deterministic timestamp
timestamp=$(date +%s)

# Missing error handling (no set -e)
# Missing idempotent flags (mkdir without -p)
mkdir /tmp/test_dir
