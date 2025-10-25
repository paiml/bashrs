#!/bin/sh
# Example 2: Variables
# Demonstrates variable assignment and expansion

# Simple variable
name="Claude"
echo "My name is $name"

# Multiple variables
greeting="Hello"
target="World"
echo "$greeting, $target!"

# Variables with numbers
version="1.0"
echo "Version: $version"

# Variable reuse
message="WASM is awesome"
echo $message
echo $message
