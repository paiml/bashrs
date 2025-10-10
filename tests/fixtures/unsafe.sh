#!/bin/bash
# This script has multiple safety issues

FILES=$1
DIR=$2

# SC2086: Unquoted variable expansions
ls $FILES
rm -rf $DIR

# SC2046: Unquoted command substitution
result=$(find . -name '*.txt')
echo $result

# SC2116: Useless echo
message=$(echo $result)
