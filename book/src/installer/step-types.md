# Step Types

The installer framework supports various step types for different operations.

## Script Action

The most flexible action type. Runs arbitrary shell scripts:

```toml
[[step]]
id = "run-script"
name = "Run Custom Script"
action = "script"

[step.script]
interpreter = "sh"  # Options: sh, bash, dash
content = '''
echo "Running installation..."
mkdir -p ~/.local/bin
'''
```

### Script Options

| Option | Description | Default |
|--------|-------------|---------|
| `interpreter` | Shell interpreter | `sh` |
| `content` | Script content | required |

## Package Installation (apt)

Install packages via apt package manager:

```toml
[[step]]
id = "install-deps"
name = "Install Dependencies"
action = "apt-install"
packages = ["curl", "git", "build-essential"]
```

## Package Removal (apt)

Remove packages:

```toml
[[step]]
id = "cleanup"
name = "Remove Build Dependencies"
action = "apt-remove"
packages = ["build-essential"]
```

## File Write

Write content to a file:

```toml
[[step]]
id = "create-config"
name = "Create Configuration File"
action = "file-write"
path = "~/.config/my-app/config.toml"
content = '''
[settings]
log_level = "info"
'''
```

## User/Group Management

Add users to groups:

```toml
[[step]]
id = "add-docker-group"
name = "Add User to Docker Group"
action = "user-group"
user = "$USER"
group = "docker"
```

## Verification

Run verification commands without side effects:

```toml
[[step]]
id = "verify-install"
name = "Verify Installation"
action = "verify"

[step.verification]
commands = [
    { cmd = "my-app --version", expect = "1.0" },
    { cmd = "test -f ~/.config/my-app/config.toml" }
]
```

## Step Dependencies

Steps can depend on other steps:

```toml
[[step]]
id = "step-a"
name = "First Step"
action = "script"
[step.script]
content = "echo 'Step A'"

[[step]]
id = "step-b"
name = "Second Step"
action = "script"
depends_on = ["step-a"]  # Runs after step-a
[step.script]
content = "echo 'Step B'"

[[step]]
id = "step-c"
name = "Third Step"
action = "script"
depends_on = ["step-a", "step-b"]  # Runs after both
[step.script]
content = "echo 'Step C'"
```

## Parallel Execution

Steps without dependencies can run in parallel:

```toml
# These run in parallel (wave 1)
[[step]]
id = "download-a"
name = "Download Component A"
action = "script"
[step.script]
content = "curl -O https://example.com/a.tar.gz"

[[step]]
id = "download-b"
name = "Download Component B"
action = "script"
[step.script]
content = "curl -O https://example.com/b.tar.gz"

# This waits for both (wave 2)
[[step]]
id = "install"
name = "Install Components"
action = "script"
depends_on = ["download-a", "download-b"]
[step.script]
content = "tar xzf a.tar.gz && tar xzf b.tar.gz"
```

## Preconditions and Postconditions

Validate state before and after steps:

```toml
[[step]]
id = "install"
name = "Install Application"
action = "script"

# Preconditions (must be true before step runs)
[step.preconditions]
file_exists = "/usr/bin/curl"
command_succeeds = "which curl"

# Postconditions (must be true after step completes)
[step.postconditions]
file_exists = "~/.local/bin/my-app"
file_mode = "755"
command_succeeds = "~/.local/bin/my-app --version"

[step.script]
content = '''
curl -fsSL https://example.com/my-app -o ~/.local/bin/my-app
chmod +x ~/.local/bin/my-app
'''
```

## Timeouts and Retries

Configure step timing:

```toml
[[step]]
id = "download-large"
name = "Download Large File"
action = "script"

[step.timing]
timeout = "10m"  # 10 minute timeout

[step.timing.retry]
count = 3        # Retry up to 3 times
delay = "30s"    # Wait 30 seconds between retries
backoff = "exponential"  # exponential or linear

[step.script]
content = "curl -fsSL https://example.com/large-file.tar.gz -o file.tar.gz"
```

## Failure Handling

Configure what happens on failure:

```toml
[[step]]
id = "optional-step"
name = "Optional Enhancement"
action = "script"

[step.on_failure]
action = "continue"  # continue, stop, abort, retry
message = "Enhancement failed, continuing without it"
preserve_state = true

[step.script]
content = "install-optional-feature || true"
```

## Privilege Escalation

Some steps may need elevated privileges:

```toml
[[step]]
id = "install-system"
name = "Install System Package"
action = "script"
privileges = "root"  # Requires sudo

[step.script]
content = '''
apt-get update
apt-get install -y my-package
'''
```

## Next Steps

- [Checkpointing](./checkpointing.md) - Resume from failures
- Artifacts - Download and verify files (coming soon)
- [Testing](./testing.md) - Container-based testing
