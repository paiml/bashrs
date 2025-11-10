#!/bin/bash
# Minimal bash script for benchmarking
# Uses only basic constructs known to be supported by parser
# Note: Arithmetic expansion $((expr)) NOW SUPPORTED as of v6.33.0

echo "test"
x=1
y=2
z=$x

if [ "$x" = "1" ]; then
    echo "yes"
fi

for i in 1 2 3; do
    echo "$i"
done

# Simple while loop without arithmetic expansion
count=3
while [ "$count" != "0" ]; do
    echo "Iteration $count"
    # Note: Decrement requires arithmetic expansion - using break instead
    count=0
done

echo "done"
