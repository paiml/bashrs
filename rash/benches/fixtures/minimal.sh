#!/bin/bash
# Minimal bash script for benchmarking
# Uses only basic constructs known to be supported by parser

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

while [ "$y" -gt "0" ]; do
    y=$((y - 1))
done

echo "done"
