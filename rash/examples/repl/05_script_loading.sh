#!/bin/bash
# REPL Example 05: Script Loading and Analysis
# Demonstrates loading, analyzing, and iterating on bash scripts
#
# This example shows:
# - Loading scripts with :load
# - Function extraction
# - Reloading after changes
# - Script analysis workflow
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 05: Script Loading and Analysis
=================================================================

This example demonstrates how to load and analyze complete bash
scripts in the REPL for interactive development.

STEP 1: Create a sample script
-------------------------------
First, create a sample script to work with:

$ cat > /tmp/deploy.sh << 'SCRIPT'
#!/bin/bash
# Sample deployment script

function log_info() {
  echo "[INFO] $(date +%Y-%m-%d_%H:%M:%S) $1"
}

function log_error() {
  echo "[ERROR] $(date +%Y-%m-%d_%H:%M:%S) $1" >&2
}

function check_dependencies() {
  log_info "Checking dependencies..."
  command -v docker >/dev/null || { log_error "docker not found"; exit 1; }
  command -v kubectl >/dev/null || { log_error "kubectl not found"; exit 1; }
  log_info "All dependencies found"
}

function build_app() {
  log_info "Building application..."
  docker build -t myapp:$VERSION .
  log_info "Build complete"
}

function deploy_app() {
  log_info "Deploying to $ENVIRONMENT..."
  kubectl apply -f deployment.yaml
  kubectl set image deployment/myapp myapp=myapp:$VERSION
  log_info "Deployment complete"
}

# Main execution
check_dependencies
build_app
deploy_app
SCRIPT

STEP 2: Load the script in REPL
--------------------------------
$ bashrs repl
bashrs [normal]> :load /tmp/deploy.sh
✓ Loaded: /tmp/deploy.sh (5 functions, 35 lines)

STEP 3: View extracted functions
---------------------------------
bashrs [normal]> :functions
Available functions (5 total):
  1 log_info
  2 log_error
  3 check_dependencies
  4 build_app
  5 deploy_app

STEP 4: Analyze the script
---------------------------
# Check for linting issues
bashrs [normal]> :mode lint
Switched to lint mode

bashrs [lint]> :load /tmp/deploy.sh
Found 8 issue(s):
  ⚠ 6 warning(s)
  ℹ 2 info

Issues found:
  [1] ⚠ SC2086 - Unquoted $VERSION (multiple occurrences)
  [2] ⚠ SC2086 - Unquoted $ENVIRONMENT
  [3] ℹ INFO - Consider using mktemp for temp files
  [4] ℹ INFO - Consider adding error handling to build

STEP 5: Get purified version
-----------------------------
bashrs [lint]> :mode purify
Switched to purify mode

bashrs [purify]> :load /tmp/deploy.sh
✓ Purified version available

Purification suggestions:
  1. Quote variables: "$VERSION", "$ENVIRONMENT"
  2. Make operations idempotent:
     - docker build → docker build (check if image exists first)
     - kubectl apply → kubectl apply (already idempotent)
  3. Add error handling:
     - Check build success before deploy
     - Rollback on failure

STEP 6: Reload after editing
-----------------------------
# Edit /tmp/deploy.sh in your editor to fix issues...
# Then reload in REPL:

bashrs [purify]> :reload
Reloading: /tmp/deploy.sh
✓ Reloaded: /tmp/deploy.sh (5 functions, 40 lines)

bashrs [purify]> :mode lint
bashrs [lint]> :load /tmp/deploy.sh
✓ No issues found!

STEP 7: Iterative development workflow
---------------------------------------
# Step 7.1: Load script
bashrs [normal]> :load ~/projects/app/deploy.sh

# Step 7.2: Analyze in lint mode
bashrs [normal]> :mode lint
bashrs [lint]> :load ~/projects/app/deploy.sh
Found 3 issues...

# Step 7.3: Edit script externally
# (Open in vim/emacs/vscode)

# Step 7.4: Reload and verify
bashrs [lint]> :reload
✓ Reloaded

bashrs [lint]> :load ~/projects/app/deploy.sh
Found 1 issue...  # Progress!

# Step 7.5: Repeat until clean
bashrs [lint]> # Edit again...
bashrs [lint]> :reload
✓ No issues found!

# Step 7.6: Get purified version
bashrs [lint]> :mode purify
bashrs [purify]> :load ~/projects/app/deploy.sh
✓ Script is idempotent and deterministic!

=================================================================
Script Analysis Workflow:
=================================================================

1. Load script
   :load script.sh

2. View structure
   :functions

3. Check for issues
   :mode lint
   :load script.sh

4. Get purified version
   :mode purify
   :load script.sh

5. Edit script externally
   (in your editor)

6. Reload and verify
   :reload

7. Repeat until clean
   (iterate steps 5-6)

=================================================================
Real-World Example: CI/CD Script Development
=================================================================

Scenario: Developing a CI/CD deployment script

# Terminal 1: Editor
$ vim ci/deploy.sh

# Terminal 2: REPL
$ bashrs repl

# Initial load
bashrs [normal]> :load ci/deploy.sh
✓ Loaded: ci/deploy.sh (8 functions, 150 lines)

# Check functions
bashrs [normal]> :functions
Available functions (8 total):
  1 validate_env
  2 build_docker_image
  3 run_tests
  4 push_to_registry
  5 deploy_staging
  6 run_smoke_tests
  7 deploy_production
  8 rollback

# Analyze for issues
bashrs [normal]> :mode lint
bashrs [lint]> :load ci/deploy.sh
Found 15 issues:
  ⚠ 12 warnings
  ℹ 3 info

# View specific function
bashrs [lint]> :parse "$(grep -A 10 'function validate_env' ci/deploy.sh)"

# Fix issues in editor (Terminal 1)
# Then reload:

bashrs [lint]> :reload
Reloading: ci/deploy.sh
Found 8 issues...  # Better!

# Continue iterating...

bashrs [lint]> :reload
Found 2 issues...

bashrs [lint]> :reload
✓ No issues found!

# Verify idempotency
bashrs [lint]> :mode purify
bashrs [purify]> :load ci/deploy.sh
✓ Script is idempotent and safe to re-run!

=================================================================
Advanced: Multi-Script Projects
=================================================================

For projects with multiple scripts:

# Load main script
bashrs [normal]> :load lib/utils.sh
✓ Loaded: lib/utils.sh (10 functions)

bashrs [normal]> :functions
Available functions (10 total):
  1 log_info
  2 log_error
  ...

# Load another script
bashrs [normal]> :load lib/deploy.sh
✓ Loaded: lib/deploy.sh (5 functions)

bashrs [normal]> :functions
Available functions (15 total):
  1 log_info
  2 log_error
  ...
  11 deploy_app
  12 rollback
  ...

# Note: Functions accumulate in the session
# Use :mode normal and reassign to clear

=================================================================
Debugging Complex Scripts:
=================================================================

For complex scripts with issues:

1. Load the script
   bashrs [normal]> :load complex.sh

2. View structure
   bashrs [normal]> :functions

3. Check specific sections
   bashrs [normal]> :parse "$(sed -n '10,20p' complex.sh)"

4. Lint specific functions
   bashrs [normal]> :mode lint
   bashrs [lint]> "$(grep -A 20 'function problematic' complex.sh)"

5. Explain confusing constructs
   bashrs [lint]> :mode explain
   bashrs [explain]> ${VERSION:?Error: VERSION not set}

6. Test in normal mode
   bashrs [explain]> :mode normal
   bashrs [normal]> VERSION=1.0
   bashrs [normal]> echo ${VERSION:?Error}

=================================================================
Tips for Effective Script Analysis:
=================================================================

1. Load early and often
   Load your script at the start of each session

2. Use :functions to navigate
   Quickly see script structure

3. Combine modes for complete analysis
   lint → purify → explain → normal

4. Reload after every change
   :reload is your friend

5. Use history to track changes
   :history shows your workflow

6. Test functions individually
   Extract and test complex functions separately

=================================================================
Key Takeaways:
=================================================================

1. :load parses scripts and extracts functions
2. :functions shows all extracted functions
3. :reload reloads the most recent script
4. Combine with lint/purify modes for analysis
5. Iterative edit-reload-verify workflow is powerful
6. REPL complements your text editor

Next Steps:
-----------
Try example 06_cicd_pipeline.sh to see how to use the REPL
for CI/CD pipeline development!
EOF
