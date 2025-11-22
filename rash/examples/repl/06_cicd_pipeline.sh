#!/bin/bash
# REPL Example 06: CI/CD Pipeline Development
# Demonstrates using the REPL for CI/CD script development
#
# This example shows:
# - Building deployment pipelines interactively
# - Testing commands before committing
# - Ensuring idempotency for CI/CD
# - Variable management for environments
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 06: CI/CD Pipeline Development
=================================================================

This example demonstrates how to use the REPL to develop and test
CI/CD pipeline scripts interactively.

SCENARIO: Building a Docker deployment pipeline
------------------------------------------------

STEP 1: Set up environment variables
-------------------------------------
$ bashrs repl

# Production environment
bashrs [normal]> env=production
✓ Variable set: env = production

bashrs [normal]> region=us-east-1
✓ Variable set: region = us-east-1

bashrs [normal]> registry=123456789.dkr.ecr.us-east-1.amazonaws.com
✓ Variable set: registry = 123456789.dkr.ecr.us-east-1.amazonaws.com

bashrs [normal]> app_name=myapp
✓ Variable set: app_name = myapp

bashrs [normal]> version=v2.1.0
✓ Variable set: version = v2.1.0

bashrs [normal]> :vars
Session Variables (5 variables):
  app_name = myapp
  env = production
  region = us-east-1
  registry = 123456789.dkr.ecr.us-east-1.amazonaws.com
  version = v2.1.0

STEP 2: Test build commands in lint mode
-----------------------------------------
bashrs [normal]> :mode lint
Switched to lint mode

# Test Docker build
bashrs [lint]> docker build -t $app_name:$version .
Found 2 issue(s):
  ⚠ 2 warning(s)

[1] ⚠ SC2086 - Unquoted variable: app_name
[2] ⚠ SC2086 - Unquoted variable: version

Fix: docker build -t "$app_name:$version" .

# Test Docker tag
bashrs [lint]> docker tag $app_name:$version $registry/$app_name:$version
Found 4 issue(s):
  ⚠ 4 warning(s)
All variables need quoting.

# Test Docker push
bashrs [lint]> docker push $registry/$app_name:$version
Found 3 issue(s):
  ⚠ 3 warning(s)

STEP 3: Get purified versions
------------------------------
bashrs [lint]> :mode purify
Switched to purify mode

bashrs [purify]> docker build -t $app_name:$version .
✓ Purified:
docker build -t "$app_name:$version" .

bashrs [purify]> docker tag $app_name:$version $registry/$app_name:$version
✓ Purified:
docker tag "$app_name:$version" "$registry/$app_name:$version"

bashrs [purify]> docker push $registry/$app_name:$version
✓ Purified:
docker push "$registry/$app_name:$version"

STEP 4: Build complete pipeline interactively
----------------------------------------------
bashrs [purify]> :mode normal
Switched to normal mode

# Build pipeline step by step, testing each

# Step 1: Login to registry
bashrs [normal]> aws ecr get-login-password --region $region | docker login --username AWS --password-stdin $registry
# (Would execute if AWS configured)

# Step 2: Build image
bashrs [normal]> docker build -t "$app_name:$version" .
# (Would execute)

# Step 3: Tag image
bashrs [normal]> docker tag "$app_name:$version" "$registry/$app_name:$version"
# (Would execute)

# Step 4: Run tests
bashrs [normal]> docker run --rm "$app_name:$version" npm test
# (Would execute)

# Step 5: Push to registry
bashrs [normal]> docker push "$registry/$app_name:$version"
# (Would execute)

# Step 6: Deploy to Kubernetes
bashrs [normal]> kubectl set image deployment/$app_name $app_name="$registry/$app_name:$version" --namespace=$env
# (Would execute)

# Step 7: Wait for rollout
bashrs [normal]> kubectl rollout status deployment/$app_name --namespace=$env
# (Would execute)

STEP 5: Review and save history
--------------------------------
bashrs [normal]> :history
Command History:
  1 env=production
  2 region=us-east-1
  3 registry=123456789.dkr.ecr.us-east-1.amazonaws.com
  ...
  15 docker build -t "$app_name:$version" .
  16 docker tag "$app_name:$version" "$registry/$app_name:$version"
  17 docker push "$registry/$app_name:$version"
  18 kubectl set image deployment/$app_name ...
  19 :history

# Now copy the working commands to your CI/CD script!

=================================================================
Complete CI/CD Pipeline Script (from REPL session):
=================================================================

After testing in REPL, create this script:

$ cat > ci/deploy.sh << 'SCRIPT'
#!/bin/bash
set -euo pipefail

# Environment variables (from CI/CD system)
: "${ENV:?Error: ENV not set}"
: "${REGION:?Error: REGION not set}"
: "${REGISTRY:?Error: REGISTRY not set}"
: "${APP_NAME:?Error: APP_NAME not set}"
: "${VERSION:?Error: VERSION not set}"

# Login to ECR
echo "Logging in to ECR..."
aws ecr get-login-password --region "$REGION" | \
  docker login --username AWS --password-stdin "$REGISTRY"

# Build image
echo "Building Docker image..."
docker build -t "$APP_NAME:$VERSION" .

# Tag image
echo "Tagging image..."
docker tag "$APP_NAME:$VERSION" "$REGISTRY/$APP_NAME:$VERSION"

# Run tests
echo "Running tests..."
docker run --rm "$APP_NAME:$VERSION" npm test

# Push to registry
echo "Pushing to registry..."
docker push "$REGISTRY/$APP_NAME:$VERSION"

# Deploy to Kubernetes
echo "Deploying to Kubernetes..."
kubectl set image deployment/"$APP_NAME" \
  "$APP_NAME"="$REGISTRY/$APP_NAME:$VERSION" \
  --namespace="$ENV"

# Wait for rollout
echo "Waiting for rollout..."
kubectl rollout status deployment/"$APP_NAME" --namespace="$ENV"

echo "Deployment complete!"
SCRIPT

=================================================================
Multi-Environment Pipeline:
=================================================================

Use REPL to test different environments:

# Staging environment
bashrs [normal]> env=staging
bashrs [normal]> region=us-west-2
bashrs [normal]> version=v2.1.0-rc1

bashrs [normal]> :mode purify
bashrs [purify]> # Test all commands...

# Production environment
bashrs [purify]> :mode normal
bashrs [normal]> env=production
bashrs [normal]> region=us-east-1
bashrs [normal]> version=v2.1.0

bashrs [normal]> :mode purify
bashrs [purify]> # Test all commands again...

=================================================================
Idempotency for CI/CD:
=================================================================

CI/CD scripts must be idempotent (safe to re-run):

# Non-idempotent (fails on retry)
bashrs [normal]> kubectl create deployment $app_name --image=$registry/$app_name:$version

# Idempotent (safe to retry)
bashrs [purify]> kubectl apply -f deployment.yaml

# Idempotent with conditional
bashrs [purify]> kubectl get deployment $app_name >/dev/null 2>&1 || \
                 kubectl create deployment $app_name --image=$registry/$app_name:$version

=================================================================
Testing Rollback Scenarios:
=================================================================

Use REPL to develop rollback logic:

bashrs [normal]> previous_version=v2.0.0
bashrs [normal]> current_version=v2.1.0

# Test rollback command
bashrs [normal]> :mode purify
bashrs [purify]> kubectl set image deployment/$app_name $app_name=$registry/$app_name:$previous_version

# Test rollback verification
bashrs [purify]> kubectl rollout status deployment/$app_name
bashrs [purify]> kubectl get pods -l app=$app_name

=================================================================
Debugging Failed Deployments:
=================================================================

Use REPL to investigate deployment issues:

# Check deployment status
bashrs [normal]> kubectl get deployment $app_name -o yaml

# Check pod status
bashrs [normal]> kubectl get pods -l app=$app_name

# View logs
bashrs [normal]> kubectl logs -l app=$app_name --tail=50

# Describe pods
bashrs [normal]> kubectl describe pods -l app=$app_name

=================================================================
Real-World GitLab CI Example:
=================================================================

Developing .gitlab-ci.yml scripts interactively:

# Set GitLab CI variables
bashrs [normal]> CI_COMMIT_SHA=abc123
bashrs [normal]> CI_COMMIT_REF_NAME=main
bashrs [normal]> CI_PROJECT_NAME=myapp

# Test build stage
bashrs [normal]> :mode purify
bashrs [purify]> docker build -t $CI_PROJECT_NAME:$CI_COMMIT_SHA .

# Test deploy stage
bashrs [purify]> kubectl set image deployment/$CI_PROJECT_NAME \
                 $CI_PROJECT_NAME=$REGISTRY/$CI_PROJECT_NAME:$CI_COMMIT_SHA

# Save to .gitlab-ci.yml
bashrs [purify]> :history
# Copy commands to GitLab CI config

=================================================================
Kubernetes Deployment Patterns:
=================================================================

Test these patterns in REPL:

1. Blue-Green Deployment
   bashrs [purify]> kubectl apply -f deployment-blue.yaml
   bashrs [purify]> # Test new version
   bashrs [purify]> kubectl patch service $app_name -p '{"spec":{"selector":{"version":"green"}}}'

2. Canary Deployment
   bashrs [purify]> kubectl apply -f deployment-canary.yaml
   bashrs [purify]> # Monitor metrics
   bashrs [purify]> kubectl scale deployment $app_name-canary --replicas=0

3. Rolling Update
   bashrs [purify]> kubectl set image deployment/$app_name $app_name=$new_image
   bashrs [purify]> kubectl rollout status deployment/$app_name

=================================================================
CI/CD Best Practices (Validated in REPL):
=================================================================

✅ Quote all variables
✅ Use set -euo pipefail for safety
✅ Validate required variables with ${VAR:?Error}
✅ Make operations idempotent
✅ Add proper error handling
✅ Log each step clearly
✅ Test rollback procedures
✅ Use health checks before promoting

=================================================================
Key Takeaways:
=================================================================

1. Use REPL to test CI/CD commands before committing
2. Set up environment variables to simulate CI/CD context
3. Use purify mode to ensure idempotent operations
4. Test rollback scenarios interactively
5. Save working commands from :history to your CI/CD scripts
6. Validate multi-environment deployments
7. Test failure scenarios and recovery

Next Steps:
-----------
Try example 07_configuration_management.sh to learn about
managing shell configuration files!
EOF
