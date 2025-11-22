#!/bin/bash
# REPL Example 03: Purification Workflow
# Demonstrates how to make scripts idempotent and deterministic
#
# This example shows:
# - Idempotency transformations
# - Determinism enforcement
# - Common purification patterns
# - Purify mode usage
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 03: Purification Workflow
=================================================================

This example demonstrates how to use the REPL to make bash scripts
idempotent (safe to re-run) and deterministic (predictable results).

STEP 1: Start in purify mode
-----------------------------
$ bashrs repl
bashrs [normal]> :mode purify
Switched to purify mode

STEP 2: Purify directory operations
------------------------------------
# Non-idempotent: fails if directory exists
bashrs [purify]> mkdir /var/lib/myapp
✓ Purified:
mkdir -p "/var/lib/myapp"

# Non-idempotent: fails if directory doesn't exist
bashrs [purify]> rmdir /tmp/old
✓ Purified:
rm -rf "/tmp/old"

STEP 3: Purify file operations
-------------------------------
# Non-idempotent: fails if file doesn't exist
bashrs [purify]> rm /tmp/tempfile.txt
✓ Purified:
rm -f "/tmp/tempfile.txt"

# Non-idempotent: fails if file exists
bashrs [purify]> cp source.txt dest.txt
✓ Purified:
cp -f "source.txt" "dest.txt"

STEP 4: Purify symbolic links
------------------------------
# Non-idempotent: fails if link exists
bashrs [purify]> ln -s /opt/app/v2.0 /usr/local/bin/app
✓ Purified:
ln -sf "/opt/app/v2.0" "/usr/local/bin/app"

STEP 5: Purify non-deterministic values
----------------------------------------
# Non-deterministic: $RANDOM gives different value each time
bashrs [purify]> SESSION_ID=$RANDOM
✓ Purified:
SESSION_ID="$(date +%s)-$$"

# Non-deterministic: timestamp changes
bashrs [purify]> RELEASE_NAME="release-$(date +%s)"
✓ Purified:
RELEASE_NAME="release-${VERSION}"
# Note: Use version variable instead of timestamp

# Non-deterministic: process ID
bashrs [purify]> PID=$$
✓ Purified:
PID="$(cat /var/run/myapp.pid)"
# Note: Read from pidfile for consistency

STEP 6: Purify destructive operations
--------------------------------------
# Unsafe: no confirmation, no backup
bashrs [purify]> rm -rf /var/lib/app
✓ Purified:
if [ -d "/var/lib/app" ]; then
  mv "/var/lib/app" "/var/lib/app.bak.$(date +%Y%m%d-%H%M%S)"
fi
rm -rf "/var/lib/app"

STEP 7: Purify network operations
----------------------------------
# Non-idempotent: doesn't check if already downloaded
bashrs [purify]> wget https://example.com/file.tar.gz
✓ Purified:
if [ ! -f "file.tar.gz" ]; then
  wget -c https://example.com/file.tar.gz
fi

# Non-idempotent: always overwrites
bashrs [purify]> curl -o app.tar.gz https://example.com/app.tar.gz
✓ Purified:
curl -z app.tar.gz -o app.tar.gz https://example.com/app.tar.gz
# Note: -z only downloads if newer than local file

STEP 8: Purify service operations
----------------------------------
# Non-idempotent: fails if already running
bashrs [purify]> systemctl start nginx
✓ Purified:
systemctl start nginx || systemctl restart nginx

# Non-idempotent: fails if not running
bashrs [purify]> systemctl stop nginx
✓ Purified:
systemctl stop nginx || true

=================================================================
Purification Patterns:
=================================================================

1. Directory Creation
   Before: mkdir /path/to/dir
   After:  mkdir -p "/path/to/dir"

2. File Deletion
   Before: rm file.txt
   After:  rm -f "file.txt"

3. Symbolic Links
   Before: ln -s target link
   After:  ln -sf "target" "link"

4. File Copy
   Before: cp source dest
   After:  cp -f "source" "dest"

5. Non-deterministic Values
   Before: ID=$RANDOM
   After:  ID="$(uuidgen)" or ID="${VERSION}-${BUILD_NUMBER}"

6. Timestamps
   Before: BACKUP=backup-$(date +%s)
   After:  BACKUP="backup-${VERSION}"

7. Conditional Operations
   Before: wget file.tar.gz
   After:  [ -f file.tar.gz ] || wget file.tar.gz

8. Service Management
   Before: systemctl start app
   After:  systemctl is-active app || systemctl start app

=================================================================
Real-World Example: Deployment Script Purification
=================================================================

BEFORE (non-idempotent):

  #!/bin/bash
  mkdir /var/lib/myapp
  mkdir /var/log/myapp
  cp config.ini /etc/myapp/config.ini
  ln -s /opt/myapp/v2.0 /usr/local/bin/myapp
  systemctl start myapp
  echo "Deployed at $(date +%s)" > /var/lib/myapp/deploy.log

AFTER (idempotent):

  #!/bin/bash
  mkdir -p "/var/lib/myapp"
  mkdir -p "/var/log/myapp"
  mkdir -p "/etc/myapp"
  cp -f "config.ini" "/etc/myapp/config.ini"
  ln -sf "/opt/myapp/v2.0" "/usr/local/bin/myapp"
  systemctl is-active myapp || systemctl start myapp
  echo "Deployed version ${VERSION}" > /var/lib/myapp/deploy.log

How to purify it in REPL:

  bashrs [purify]> :load deploy.sh
  bashrs [purify]> # Each command automatically purified
  bashrs [purify]> :history
  # Copy purified commands back to deploy.sh

=================================================================
Idempotency vs Determinism:
=================================================================

Idempotency: Safe to run multiple times
  - mkdir -p instead of mkdir
  - rm -f instead of rm
  - ln -sf instead of ln -s
  - Operations don't fail if already in desired state

Determinism: Same input = same output
  - Replace $RANDOM with version-based IDs
  - Replace timestamps with build numbers
  - Replace process IDs with stable identifiers
  - Results are reproducible

=================================================================
Testing Idempotency:
=================================================================

To test if a script is idempotent:

1. Run the script once
2. Run it again immediately
3. Both runs should succeed
4. Both runs should produce the same result

In the REPL:

  bashrs [normal]> :load deploy.sh
  bashrs [normal]> # Run all commands
  bashrs [normal]> :reload
  bashrs [normal]> # Run again - should succeed

=================================================================
Key Takeaways:
=================================================================

1. Purify mode automatically fixes idempotency issues
2. Add -p, -f, -s flags to make operations safe to re-run
3. Replace non-deterministic values with stable identifiers
4. Test idempotency by running scripts twice
5. Idempotent scripts are safer in production

Next Steps:
-----------
Try example 04_explain_mode.sh to learn bash constructs
interactively!
EOF
