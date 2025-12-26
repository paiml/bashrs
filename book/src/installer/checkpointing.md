# Checkpointing and Recovery

The installer framework provides robust checkpointing to ensure installations can resume from any failure point.

## How Checkpointing Works

Each step can optionally create a checkpoint. When a step completes:

1. Step state is saved to SQLite database
2. System state is captured (file checksums, etc.)
3. Rollback commands are recorded

If a later step fails, you can:
- Resume from the last checkpoint
- Rollback to a previous state
- Restart from a specific step

## Enabling Checkpoints

Enable checkpointing for a step:

```toml
[[step]]
id = "install-app"
name = "Install Application"
action = "script"

[step.checkpoint]
enabled = true
rollback = "rm -rf ~/.local/bin/my-app"
state_files = ["~/.local/bin/my-app", "~/.config/my-app"]

[step.script]
content = '''
curl -fsSL https://example.com/my-app -o ~/.local/bin/my-app
chmod +x ~/.local/bin/my-app
'''
```

## Checkpoint Options

| Option | Description |
|--------|-------------|
| `enabled` | Enable checkpointing for this step |
| `rollback` | Command to undo this step |
| `state_files` | Files to track for state changes |

## Resuming from Failure

If an installation fails:

```bash
# Resume from last successful checkpoint
bashrs installer resume my-installer

# Resume from a specific step
bashrs installer resume my-installer --from download-app

# List available checkpoints
bashrs installer resume my-installer --list
```

## Checkpoint Storage

Checkpoints are stored in the checkpoint directory:

```
~/.local/share/bashrs/checkpoints/
└── my-installer/
    ├── checkpoint.db        # SQLite database
    ├── state/               # State snapshots
    │   ├── step-1.json
    │   └── step-2.json
    └── rollback/            # Rollback scripts
        ├── step-1.sh
        └── step-2.sh
```

## Custom Checkpoint Directory

Specify a custom checkpoint directory:

```bash
bashrs installer run my-installer --checkpoint-dir /tmp/my-checkpoints
```

## Rollback

Rollback to a previous state:

```bash
# Rollback last step
bashrs installer rollback my-installer

# Rollback to specific step
bashrs installer rollback my-installer --to prepare-env

# Full rollback (undo entire installation)
bashrs installer rollback my-installer --full
```

## Atomic Steps

For critical operations, make steps atomic:

```toml
[[step]]
id = "atomic-install"
name = "Atomic Installation"
action = "script"

[step.checkpoint]
enabled = true
atomic = true  # All-or-nothing execution
rollback = '''
# Comprehensive rollback
rm -rf ~/.local/bin/my-app
rm -rf ~/.config/my-app
rm -rf ~/.local/share/my-app
'''

[step.script]
content = '''
# If any command fails, entire step fails and rollback runs
set -e
mkdir -p ~/.local/bin
curl -fsSL https://example.com/my-app -o ~/.local/bin/my-app
chmod +x ~/.local/bin/my-app
mkdir -p ~/.config/my-app
echo "[settings]" > ~/.config/my-app/config.toml
'''
```

## Best Practices

1. **Enable checkpoints for long-running steps**: Network downloads, compilations
2. **Always provide rollback commands**: Make steps reversible
3. **Track state files**: Know what files were modified
4. **Use atomic mode for critical operations**: Ensure consistency
5. **Test rollback**: Verify rollback commands actually work

## Example: Robust Installer

```toml
[installer]
name = "robust-app"
version = "1.0.0"

[[step]]
id = "backup-existing"
name = "Backup Existing Installation"
action = "script"

[step.checkpoint]
enabled = true
rollback = "rm -rf ~/.my-app.backup"

[step.script]
content = '''
if [ -d ~/.my-app ]; then
    cp -r ~/.my-app ~/.my-app.backup
fi
'''

[[step]]
id = "install"
name = "Install Application"
action = "script"
depends_on = ["backup-existing"]

[step.checkpoint]
enabled = true
atomic = true
rollback = '''
rm -rf ~/.my-app
if [ -d ~/.my-app.backup ]; then
    mv ~/.my-app.backup ~/.my-app
fi
'''
state_files = ["~/.my-app"]

[step.script]
content = '''
set -e
mkdir -p ~/.my-app
curl -fsSL https://example.com/app.tar.gz | tar xz -C ~/.my-app
'''

[[step]]
id = "verify"
name = "Verify Installation"
action = "verify"
depends_on = ["install"]

[step.verification]
commands = [
    { cmd = "~/.my-app/bin/app --version", expect = "1.0" }
]
```

## Next Steps

- [Artifacts](./artifacts.md) - Download and verify files
- [Testing](./testing.md) - Container-based testing
- [Hermetic Builds](./hermetic.md) - Reproducible installations
