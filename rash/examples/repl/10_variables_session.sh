#!/bin/bash
# REPL Example 10: Variables and Session Management
# Demonstrates working with variables and managing REPL sessions
#
# This example shows:
# - Variable assignment and expansion
# - Session state management
# - Persistent variables across modes
# - Using :vars command
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 10: Variables and Session Management
=================================================================

This example demonstrates how to effectively manage variables and
session state in the REPL for productive interactive development.

=================================================================
STEP 1: Basic Variable Assignment
=================================================================

$ bashrs repl

# Simple variable assignment
bashrs [normal]> app_name="myapp"
✓ Variable set: app_name = myapp

bashrs [normal]> version=1.0.0
✓ Variable set: version = 1.0.0

bashrs [normal]> env=production
✓ Variable set: env = production

# View all variables
bashrs [normal]> :vars
Session Variables (3 variables):
  app_name = myapp
  env = production
  version = 1.0.0

=================================================================
STEP 2: Variable Expansion
=================================================================

# Simple expansion
bashrs [normal]> echo $app_name
myapp

bashrs [normal]> echo $version
1.0.0

# Braced expansion
bashrs [normal]> echo ${app_name}
myapp

# Multiple variables
bashrs [normal]> echo "Deploying $app_name version $version to $env"
Deploying myapp version 1.0.0 to production

# Variables in commands
bashrs [normal]> docker build -t "$app_name:$version" .
# Would execute: docker build -t "myapp:1.0.0" .

=================================================================
STEP 3: Variable Types
=================================================================

# String variables
bashrs [normal]> name="Alice"
✓ Variable set: name = Alice

bashrs [normal]> greeting="Hello, $name!"
✓ Variable set: greeting = Hello, Alice!

# Numeric variables
bashrs [normal]> port=8080
✓ Variable set: port = 8080

bashrs [normal]> timeout=300
✓ Variable set: timeout = 300

# Path variables
bashrs [normal]> base_dir=/var/lib/myapp
✓ Variable set: base_dir = /var/lib/myapp

bashrs [normal]> config_dir=$base_dir/config
✓ Variable set: config_dir = /var/lib/myapp/config

bashrs [normal]> data_dir=$base_dir/data
✓ Variable set: data_dir = /var/lib/myapp/data

# List variables (space-separated)
bashrs [normal]> environments="dev staging production"
✓ Variable set: environments = dev staging production

=================================================================
STEP 4: Variable Quotes
=================================================================

# Double quotes (allows expansion)
bashrs [normal]> message="Hello, $USER!"
✓ Variable set: message = Hello, root!

# Single quotes (literal string)
bashrs [normal]> literal='Hello, $USER!'
✓ Variable set: literal = Hello, $USER!

bashrs [normal]> echo $message
Hello, root!

bashrs [normal]> echo $literal
Hello, $USER!

# No quotes (word splitting may occur)
bashrs [normal]> path=/usr/local/bin
✓ Variable set: path = /usr/local/bin

# With spaces, quotes are required
bashrs [normal]> name_with_spaces="Alice Johnson"
✓ Variable set: name_with_spaces = Alice Johnson

=================================================================
STEP 5: Variables Across Modes
=================================================================

# Set variables in normal mode
bashrs [normal]> app=myapp
✓ Variable set: app = myapp

bashrs [normal]> tag=v2.0
✓ Variable set: tag = v2.0

# Switch to purify mode - variables persist
bashrs [normal]> :mode purify
Switched to purify mode

bashrs [purify]> echo $app:$tag
myapp:v2.0

bashrs [purify]> docker build -t $app:$tag .
✓ Purified:
docker build -t "$app:$tag" .

# Switch to lint mode - variables still available
bashrs [purify]> :mode lint
Switched to lint mode

bashrs [lint]> docker push $app:$tag
Found 1 issue(s):
  ⚠ 1 warning(s)
[1] ⚠ SC2086 - Unquoted variable

# Check variables - they persist
bashrs [lint]> :vars
Session Variables (2 variables):
  app = myapp
  tag = v2.0

=================================================================
STEP 6: Variable Reassignment
=================================================================

bashrs [normal]> status=pending
✓ Variable set: status = pending

bashrs [normal]> echo $status
pending

# Reassign variable
bashrs [normal]> status=in_progress
✓ Variable set: status = in_progress

bashrs [normal]> echo $status
in_progress

# Reassign again
bashrs [normal]> status=complete
✓ Variable set: status = complete

bashrs [normal]> echo $status
complete

=================================================================
STEP 7: Environment Simulation
=================================================================

# Simulate a complete deployment environment
bashrs [normal]> PROJECT=myproject
✓ Variable set: PROJECT = myproject

bashrs [normal]> ENV=production
✓ Variable set: ENV = production

bashrs [normal]> REGION=us-east-1
✓ Variable set: REGION = us-east-1

bashrs [normal]> CLUSTER=prod-cluster
✓ Variable set: CLUSTER = prod-cluster

bashrs [normal]> NAMESPACE=myapp-prod
✓ Variable set: NAMESPACE = myapp-prod

bashrs [normal]> VERSION=v2.1.0
✓ Variable set: VERSION = v2.1.0

bashrs [normal]> REGISTRY=123456789.dkr.ecr.us-east-1.amazonaws.com
✓ Variable set: REGISTRY = 123456789.dkr.ecr.us-east-1.amazonaws.com

# View complete environment
bashrs [normal]> :vars
Session Variables (7 variables):
  CLUSTER = prod-cluster
  ENV = production
  NAMESPACE = myapp-prod
  PROJECT = myproject
  REGION = us-east-1
  REGISTRY = 123456789.dkr.ecr.us-east-1.amazonaws.com
  VERSION = v2.1.0

# Now use these variables throughout your session
bashrs [normal]> echo "Deploying $PROJECT:$VERSION to $ENV in $REGION"
Deploying myproject:v2.1.0 to production in us-east-1

=================================================================
STEP 8: Variable Patterns
=================================================================

# Configuration variables
bashrs [normal]> DB_HOST=db.example.com
bashrs [normal]> DB_PORT=5432
bashrs [normal]> DB_NAME=myapp
bashrs [normal]> DB_USER=appuser
# (Don't set DB_PASSWORD in REPL history!)

# Build connection string
bashrs [normal]> db_url="postgres://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
✓ Variable set: db_url = postgres://appuser@db.example.com:5432/myapp

# Docker variables
bashrs [normal]> registry=registry.example.com
bashrs [normal]> image_name=myapp
bashrs [normal]> image_tag=latest
bashrs [normal]> image="$registry/$image_name:$image_tag"
✓ Variable set: image = registry.example.com/myapp:latest

# Kubernetes variables
bashrs [normal]> k8s_namespace=production
bashrs [normal]> k8s_deployment=myapp
bashrs [normal]> k8s_replicas=3

=================================================================
STEP 9: Using Variables in Commands
=================================================================

# In normal mode (execution)
bashrs [normal]> echo "Building $image..."
Building registry.example.com/myapp:latest...

# In purify mode (transformation)
bashrs [normal]> :mode purify
bashrs [purify]> docker tag $image_name:$image_tag $registry/$image_name:$image_tag
✓ Purified:
docker tag "$image_name:$image_tag" "$registry/$image_name:$image_tag"

# In lint mode (validation)
bashrs [purify]> :mode lint
bashrs [lint]> kubectl scale deployment $k8s_deployment --replicas=$k8s_replicas
Found 2 issue(s):
  ⚠ 2 warning(s)
Fix: kubectl scale deployment "$k8s_deployment" --replicas="$k8s_replicas"

=================================================================
STEP 10: Session Workflow Example
=================================================================

Complete workflow using variables for CI/CD development:

# Step 1: Set up session variables
bashrs [normal]> app=payment-service
bashrs [normal]> version=v3.2.1
bashrs [normal]> env=staging
bashrs [normal]> region=eu-west-1
bashrs [normal]> registry=123.dkr.ecr.eu-west-1.amazonaws.com

bashrs [normal]> :vars
Session Variables (5 variables):
  app = payment-service
  env = staging
  region = eu-west-1
  registry = 123.dkr.ecr.eu-west-1.amazonaws.com
  version = v3.2.1

# Step 2: Test build commands
bashrs [normal]> :mode purify
bashrs [purify]> docker build -t $app:$version .
✓ Purified: docker build -t "$app:$version" .

bashrs [purify]> docker tag $app:$version $registry/$app:$version
✓ Purified: docker tag "$app:$version" "$registry/$app:$version"

# Step 3: Test deployment commands
bashrs [purify]> kubectl set image deployment/$app $app=$registry/$app:$version --namespace=$env
✓ Purified: kubectl set image deployment/"$app" "$app"="$registry/$app:$version" --namespace="$env"

# Step 4: View complete workflow
bashrs [purify]> :history
Command History:
  ... (shows all commands with expanded variables)

# Step 5: Save to script
# Copy commands from :history to deploy.sh

=================================================================
STEP 11: Unknown Variables
=================================================================

# Unknown variables expand to empty string (bash behavior)
bashrs [normal]> echo $unknown_variable
(empty output)

bashrs [normal]> echo "Before:$missing:After"
Before::After

# This matches bash behavior - no error, just empty expansion

=================================================================
Best Practices for Variable Management:
=================================================================

1. Use descriptive names
   ✅ app_name=myapp
   ❌ x=myapp

2. Use uppercase for environment variables
   ✅ ENV=production, REGION=us-east-1
   ✅ app_name=myapp (lowercase for local variables)

3. Quote when assigning (especially with spaces)
   ✅ name="Alice Johnson"
   ❌ name=Alice Johnson (word splitting)

4. Use :vars frequently
   ✅ :vars to view all variables
   ✅ Verify variables before using

5. Organize related variables
   ✅ Set all DB_* variables together
   ✅ Set all K8S_* variables together

6. Don't store secrets in REPL
   ❌ password="secret123" (saved in history!)
   ✅ Read from environment or secure store

7. Use variables to simulate environments
   ✅ Test with dev/staging/production variables
   ✅ Switch environments by reassigning variables

8. Variables persist across modes
   ✅ Set once in normal mode
   ✅ Use in all modes (purify, lint, explain)

=================================================================
Real-World Example: Multi-Environment Testing
=================================================================

# Development environment
bashrs [normal]> env=dev
bashrs [normal]> db_host=localhost
bashrs [normal]> db_port=5432
bashrs [normal]> api_url=http://localhost:3000

# Test with dev variables
bashrs [normal]> :mode purify
bashrs [purify]> curl $api_url/health
✓ Purified: curl "$api_url/health"

# Switch to staging environment
bashrs [purify]> :mode normal
bashrs [normal]> env=staging
bashrs [normal]> db_host=staging-db.internal
bashrs [normal]> db_port=5432
bashrs [normal]> api_url=https://api-staging.example.com

# Test with staging variables
bashrs [normal]> :mode purify
bashrs [purify]> curl $api_url/health
✓ Purified: curl "$api_url/health"

# Switch to production environment
bashrs [purify]> :mode normal
bashrs [normal]> env=production
bashrs [normal]> db_host=prod-db.internal
bashrs [normal]> db_port=5432
bashrs [normal]> api_url=https://api.example.com

# Test with production variables
bashrs [normal]> :mode purify
bashrs [purify]> curl $api_url/health
✓ Purified: curl "$api_url/health"

=================================================================
Key Takeaways:
=================================================================

1. Variables persist throughout your REPL session
2. Variables work across all modes (normal, purify, lint, explain)
3. Use :vars to view all session variables
4. Quote variables when assigning and using
5. Use descriptive variable names
6. Organize related variables together
7. Don't store secrets in REPL (saved in history!)
8. Use variables to simulate different environments
9. Variables enable reusable, parameterized workflows

Next Steps:
-----------
Try example 11_troubleshooting.sh for tips on debugging
and resolving common REPL issues!
EOF
