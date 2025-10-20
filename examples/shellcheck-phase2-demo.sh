#!/bin/bash
# examples/shellcheck-phase2-demo.sh
# Demonstration of Sprint 86 ShellCheck rules (Phase 2)

##############################################################################
# Quoting & Escaping Rules (SC2001, SC2027, SC2028, SC2050, SC2081)
##############################################################################

# SC2001: Use parameter expansion instead of sed
echo "$input" | sed 's/old/new/'  # Bad: useless use of sed
# Fix: ${input//old/new}

# SC2027: Wrong quoting in printf format strings
printf "$var\n"  # Bad: variable in format string
# Fix: printf '%s\n' "$var"

# SC2028: Echo with escape sequences
echo "Line 1\nLine 2"  # Bad: \n won't expand without -e
# Fix: printf "Line 1\nLine 2\n"
# Or: echo -e "Line 1\nLine 2"

# SC2050: Constant expression (missing $)
if [ "var" = "value" ]; then  # Bad: comparing string literals
    echo "Always false"
fi
# Fix: if [ "$var" = "value" ]; then

# SC2081: Variables in single quotes don't expand
echo 'Value: $var'  # Bad: $var won't expand in single quotes
# Fix: echo "Value: $var"

##############################################################################
# Command Substitution Rules (SC2002, SC2162, SC2164, SC2181, SC2196)
##############################################################################

# SC2002: Useless use of cat
cat file.txt | grep "pattern"  # Bad: unnecessary cat
# Fix: grep "pattern" file.txt

# SC2162: read without -r mangles backslashes
while read line; do  # Bad: read without -r
    echo "$line"
done < file.txt
# Fix: while read -r line; do

# SC2164: cd without error handling
cd /some/directory  # Bad: if cd fails, wrong directory
./script.sh
# Fix: cd /some/directory || exit

# SC2181: Check exit code directly
command
if [ $? -eq 0 ]; then  # Bad: indirect exit code check
    echo "Success"
fi
# Fix: if command; then echo "Success"; fi

# SC2196: egrep/fgrep deprecated
egrep "pattern" file.txt  # Bad: egrep is deprecated
fgrep "literal" file.txt  # Bad: fgrep is deprecated
# Fix: grep -E "pattern" file.txt
# Fix: grep -F "literal" file.txt

##############################################################################
# Array Operation Rules (SC2128, SC2145, SC2178, SC2190, SC2191)
##############################################################################

# SC2128: Array without index
files=(*.txt)
echo "$files"  # Bad: only prints first element
# Fix: echo "${files[@]}"

# SC2145: Array syntax without braces
args=(one two three)
echo "Arguments: $args[@]"  # Bad: missing braces
# Fix: echo "Arguments: ${args[@]}"

# SC2178: String assigned to array variable
array=(a b c)
array="single value"  # Bad: converts array to string
echo "${array[@]}"    # Only prints "single value"
# Fix: array[0]="single value"  # Update specific element
# Or: array=("single value")    # Keep as array

# SC2190: Associative array without keys
declare -A map
map=(value1 value2)  # Bad: associative arrays need keys
# Fix: map=([key1]=value1 [key2]=value2)

# SC2191: Space between = and (
items= (x y z)  # Bad: space causes literal =
# Fix: items=(x y z)

##############################################################################
# Real-World Example: Deploy Script with All Issues
##############################################################################

deploy_bad() {
    # SC2050: Missing $
    if [ "debug" = "true" ]; then
        echo "Debug mode"
    fi

    # SC2164: cd without error handling
    cd /app/deploy

    # SC2002: Useless cat
    cat config.txt | grep "version"

    # SC2162: read without -r
    while read line; do
        # SC2081: Variable in single quotes
        echo 'Processing: $line'
    done < manifest.txt

    # SC2128: Array without index
    files=(*.sh)
    echo "Deploying: $files"

    # SC2181: Indirect exit code
    ./deploy_step1.sh
    if [ $? -eq 0 ]; then
        echo "Step 1 complete"
    fi

    # SC2196: Deprecated egrep
    egrep "ERROR" logs/*.log
}

##############################################################################
# Fixed Version: All Issues Resolved
##############################################################################

deploy_good() {
    # Fixed: Use $debug
    if [ "$debug" = "true" ]; then
        echo "Debug mode"
    fi

    # Fixed: cd with error handling
    cd /app/deploy || exit 1

    # Fixed: Direct grep
    grep "version" config.txt

    # Fixed: read -r
    while read -r line; do
        # Fixed: Double quotes for expansion
        echo "Processing: $line"
    done < manifest.txt

    # Fixed: Array with [@]
    files=(*.sh)
    echo "Deploying: ${files[@]}"

    # Fixed: Direct exit code check
    if ./deploy_step1.sh; then
        echo "Step 1 complete"
    fi

    # Fixed: grep -E
    grep -E "ERROR" logs/*.log
}

##############################################################################
# Demonstration Output
##############################################################################

echo "=== Rash ShellCheck Phase 2 Demo ==="
echo ""
echo "This script demonstrates all 15 new ShellCheck rules from Sprint 86:"
echo "  - 5 Quoting & Escaping rules (SC2001, SC2027, SC2028, SC2050, SC2081)"
echo "  - 5 Command Substitution rules (SC2002, SC2162, SC2164, SC2181, SC2196)"
echo "  - 5 Array Operation rules (SC2128, SC2145, SC2178, SC2190, SC2191)"
echo ""
echo "Run with: rash lint examples/shellcheck-phase2-demo.sh"
echo "Expected: 15+ diagnostics (1 per rule demonstrated)"
echo ""
echo "Compare with: shellcheck examples/shellcheck-phase2-demo.sh"
echo "Rash aims for ShellCheck parity + additional safety rules"
