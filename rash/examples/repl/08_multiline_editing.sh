#!/bin/bash
# REPL Example 08: Multi-line Editing Mastery
# Demonstrates advanced multi-line input techniques
#
# This example shows:
# - Writing functions interactively
# - Creating loops and conditionals
# - Cancelling multi-line input
# - Complex script development
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 08: Multi-line Editing Mastery
=================================================================

This example demonstrates how to use multi-line input effectively
in the REPL for complex script development.

STEP 1: Understanding multi-line mode
--------------------------------------
$ bashrs repl

# When you start a construct that requires continuation,
# the REPL automatically switches to multi-line mode.

bashrs [normal]> function greet() {
... >   echo "Hello, $1"
... >   echo "Welcome to bashrs!"
... > }
✓ Function 'greet' defined

# The ... > prompt indicates continuation mode

STEP 2: Creating functions
---------------------------

# Simple function
bashrs [normal]> function log_info() {
... >   echo "[INFO] $(date +%Y-%m-%d_%H:%M:%S) $1"
... > }
✓ Function 'log_info' defined

# Function with multiple statements
bashrs [normal]> function deploy_app() {
... >   local app_name="$1"
... >   local version="$2"
... >
... >   echo "Deploying $app_name version $version..."
... >   docker build -t "$app_name:$version" .
... >   docker push "$app_name:$version"
... >   kubectl set image deployment/"$app_name" "$app_name"="$app_name:$version"
... >   echo "Deployment complete!"
... > }
✓ Function 'deploy_app' defined

# Function with error handling
bashrs [normal]> function safe_deploy() {
... >   set -euo pipefail
... >
... >   if [ $# -lt 2 ]; then
... >     echo "Usage: safe_deploy <app> <version>" >&2
... >     return 1
... >   fi
... >
... >   local app="$1"
... >   local version="$2"
... >
... >   echo "Building $app:$version..."
... >   docker build -t "$app:$version" . || {
... >     echo "Build failed!" >&2
... >     return 1
... >   }
... >
... >   echo "Pushing to registry..."
... >   docker push "$app:$version" || {
... >     echo "Push failed!" >&2
... >     return 1
... >   }
... >
... >   echo "Deployment complete!"
... > }
✓ Function 'safe_deploy' defined

STEP 3: Creating loops
-----------------------

# For loop
bashrs [normal]> for file in *.txt; do
... >   echo "Processing: $file"
... >   wc -l "$file"
... > done
Processing: file1.txt
42 file1.txt
Processing: file2.txt
108 file2.txt

# For loop with conditional
bashrs [normal]> for i in {1..10}; do
... >   if [ $((i % 2)) -eq 0 ]; then
... >     echo "$i is even"
... >   else
... >     echo "$i is odd"
... >   fi
... > done
1 is odd
2 is even
3 is odd
...

# While loop
bashrs [normal]> counter=0
✓ Variable set: counter = 0

bashrs [normal]> while [ $counter -lt 5 ]; do
... >   echo "Count: $counter"
... >   counter=$((counter + 1))
... > done
Count: 0
Count: 1
Count: 2
Count: 3
Count: 4

# While loop reading file
bashrs [normal]> while read -r line; do
... >   echo "Line: $line"
... > done < /etc/hostname
Line: myhost

STEP 4: Creating conditionals
------------------------------

# If statement
bashrs [normal]> if [ -f /etc/passwd ]; then
... >   echo "Password file exists"
... >   wc -l /etc/passwd
... > fi
Password file exists
42 /etc/passwd

# If-elif-else statement
bashrs [normal]> if [ -f config.yaml ]; then
... >   echo "Using YAML config"
... > elif [ -f config.json ]; then
... >   echo "Using JSON config"
... > elif [ -f config.toml ]; then
... >   echo "Using TOML config"
... > else
... >   echo "No config file found"
... > fi
No config file found

# Nested conditionals
bashrs [normal]> if [ -d /var/log ]; then
... >   if [ -w /var/log ]; then
... >     echo "Can write to /var/log"
... >   else
... >     echo "Cannot write to /var/log"
... >   fi
... > else
... >   echo "/var/log does not exist"
... > fi
Cannot write to /var/log

STEP 5: Creating case statements
---------------------------------

bashrs [normal]> case "$1" in
... >   start)
... >     echo "Starting service..."
... >     systemctl start myapp
... >     ;;
... >   stop)
... >     echo "Stopping service..."
... >     systemctl stop myapp
... >     ;;
... >   restart)
... >     echo "Restarting service..."
... >     systemctl restart myapp
... >     ;;
... >   status)
... >     systemctl status myapp
... >     ;;
... >   *)
... >     echo "Usage: $0 {start|stop|restart|status}"
... >     exit 1
... >     ;;
... > esac

STEP 6: Cancelling multi-line input
------------------------------------

# If you make a mistake, press Ctrl-C to cancel:

bashrs [normal]> for file in *.txt; do
... >   echo "This is wrong..."
... >   # Oops, made a mistake!
... > ^C (multi-line input cancelled)
bashrs [normal]>

# Now you can start over

STEP 7: Complex nested structures
----------------------------------

# Nested loops with conditionals
bashrs [normal]> for dir in /var/log /tmp /home; do
... >   echo "Checking $dir..."
... >   if [ -d "$dir" ]; then
... >     for file in "$dir"/*; do
... >       if [ -f "$file" ]; then
... >         echo "  File: $file"
... >       elif [ -d "$file" ]; then
... >         echo "  Dir: $file"
... >       fi
... >     done
... >   else
... >     echo "  $dir does not exist"
... >   fi
... > done

# Function with nested structures
bashrs [normal]> function process_logs() {
... >   local log_dir="${1:-/var/log}"
... >
... >   if [ ! -d "$log_dir" ]; then
... >     echo "Error: $log_dir is not a directory" >&2
... >     return 1
... >   fi
... >
... >   echo "Processing logs in $log_dir..."
... >
... >   for log_file in "$log_dir"/*.log; do
... >     if [ -f "$log_file" ]; then
... >       local line_count=$(wc -l < "$log_file")
... >
... >       if [ "$line_count" -gt 10000 ]; then
... >         echo "  $log_file: $line_count lines (large)"
... >       else
... >         echo "  $log_file: $line_count lines"
... >       fi
... >     fi
... >   done
... >
... >   echo "Processing complete!"
... > }
✓ Function 'process_logs' defined

STEP 8: Here documents in REPL
-------------------------------

# Create multi-line strings with heredoc
bashrs [normal]> cat << 'EOF'
... > This is a multi-line string
... > It can contain variables: $HOME
... > And special characters: !@#$%
... > EOF
This is a multi-line string
It can contain variables: $HOME
And special characters: !@#$%

# Heredoc in function
bashrs [normal]> function print_help() {
... >   cat << 'HELP'
... > Usage: myapp [OPTIONS] COMMAND
... >
... > Commands:
... >   start       Start the application
... >   stop        Stop the application
... >   restart     Restart the application
... >   status      Show application status
... >
... > Options:
... >   -h, --help     Show this help message
... >   -v, --verbose  Enable verbose output
... > HELP
... > }
✓ Function 'print_help' defined

=================================================================
Multi-line Editing Tips:
=================================================================

1. Natural continuation
   The REPL automatically detects incomplete input:
   - Unclosed quotes
   - Unclosed braces/brackets
   - Bash keywords (if, for, while, function, case)
   - Line ending with backslash (\)

2. Indentation
   You can indent for readability:
   function deploy() {
     if [ -f config.yaml ]; then
       kubectl apply -f config.yaml
     fi
   }

3. Cancel with Ctrl-C
   Press Ctrl-C to abandon multi-line input

4. Paste code blocks
   You can paste entire functions/loops at once
   The REPL will handle the multi-line input automatically

5. Review before executing
   Once you complete the input (close all braces, etc.),
   the REPL will execute it

=================================================================
Real-World Example: Deployment Function
=================================================================

bashrs [normal]> function deploy_to_k8s() {
... >   # Validate arguments
... >   if [ $# -lt 3 ]; then
... >     echo "Usage: deploy_to_k8s <namespace> <app> <version>" >&2
... >     return 1
... >   fi
... >
... >   local namespace="$1"
... >   local app="$2"
... >   local version="$3"
... >   local image="registry.example.com/$app:$version"
... >
... >   echo "Deploying $app:$version to $namespace..."
... >
... >   # Check if namespace exists
... >   if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
... >     echo "Creating namespace $namespace..."
... >     kubectl create namespace "$namespace"
... >   fi
... >
... >   # Update deployment
... >   kubectl set image "deployment/$app" \
... >     "$app=$image" \
... >     --namespace="$namespace" \
... >     --record
... >
... >   # Wait for rollout
... >   echo "Waiting for rollout to complete..."
... >   if kubectl rollout status "deployment/$app" \
... >        --namespace="$namespace" \
... >        --timeout=5m; then
... >     echo "Deployment successful!"
... >
... >     # Show pod status
... >     kubectl get pods \
... >       --namespace="$namespace" \
... >       --selector="app=$app"
... >   else
... >     echo "Deployment failed!" >&2
... >     echo "Rolling back..." >&2
... >     kubectl rollout undo "deployment/$app" --namespace="$namespace"
... >     return 1
... >   fi
... > }
✓ Function 'deploy_to_k8s' defined

# Test the function
bashrs [normal]> deploy_to_k8s production myapp v2.1.0

=================================================================
Testing Multi-line Code in REPL:
=================================================================

Workflow for developing complex functions:

1. Write function in REPL with multi-line input
2. Test the function immediately
3. If it works, save it to a file
4. If it doesn't work, edit and try again

Example:

# Step 1: Write function
bashrs [normal]> function backup_db() {
... >   # function body
... > }

# Step 2: Test it
bashrs [normal]> backup_db

# Step 3a: If it works, save to file
bashrs [normal]> :history
# Copy the function definition

# Step 3b: If it doesn't work, rewrite
bashrs [normal]> function backup_db() {
... >   # fixed function body
... > }

=================================================================
Key Takeaways:
=================================================================

1. REPL automatically detects incomplete input
2. Multi-line mode uses ... > prompt
3. Press Ctrl-C to cancel multi-line input
4. Indentation improves readability
5. You can paste entire code blocks
6. Test complex functions interactively
7. Save working functions from :history

Next Steps:
-----------
Try example 09_tab_completion.sh to master tab completion
and speed up your workflow!
EOF
