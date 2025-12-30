# TDD-First Installer Framework

The bashrs installer framework provides a declarative, TDD-first approach to building reliable installers. Every installer starts with tests, ensuring your deployment is verifiable from day one.

## Philosophy

The installer framework applies:

- **Karl Popper's Falsificationism**: Every claim about your installer can be empirically tested
- **Toyota Production System**: Stop-the-line quality, with checkpointing and rollback
- **EXTREME TDD**: Tests exist before implementation

## Key Features

| Feature | Description |
|---------|-------------|
| **TDD Scaffolding** | Generated test harness with falsification tests |
| **Checkpointing** | Resume from any failure point |
| **Dry-Run Mode** | Preview changes before executing |
| **Dependency Graph** | Visualize step dependencies |
| **Container Testing** | Test across multiple platforms |
| **Hermetic Builds** | Reproducible, locked dependencies |
| **Signature Verification** | Ed25519 artifact signing |

## Quick Start

```bash
# Initialize a new installer project
bashrs installer init my-app-installer

# Validate the specification
bashrs installer validate my-app-installer

# Preview changes (dry-run)
bashrs installer run my-app-installer --dry-run --diff

# Execute the installer
bashrs installer run my-app-installer
```

## Project Structure

When you run `bashrs installer init`, the following structure is created:

```text
my-app-installer/
├── installer.toml          # Declarative specification
├── tests/
│   ├── mod.rs              # Test module
│   └── falsification.rs    # Popper-style tests
└── templates/              # Template files
```

## installer.toml Format

The `installer.toml` file defines your installer declaratively:

```toml
[installer]
name = "my-app"
version = "1.0.0"
description = "My application installer"

[installer.requirements]
os = ["ubuntu >= 20.04", "debian >= 11"]
arch = ["x86_64", "aarch64"]
privileges = "user"

[[step]]
id = "install-deps"
name = "Install Dependencies"
action = "script"

[step.script]
interpreter = "sh"
content = '''
apt-get update
apt-get install -y curl git
'''

[[step]]
id = "download-app"
name = "Download Application"
action = "script"
depends_on = ["install-deps"]

[step.script]
content = '''
curl -L https://example.com/app.tar.gz -o app.tar.gz
'''
```

## Commands Reference

| Command | Description |
|---------|-------------|
| `init <name>` | Initialize new installer project |
| `validate <path>` | Validate installer.toml |
| `run <path>` | Execute installer |
| `test <path>` | Run test suite |
| `graph <path>` | Generate dependency graph |
| `lock <path>` | Generate lockfile |
| `keyring` | Manage signing keys |

## Next Steps

- [Getting Started](./getting-started.md) - Create your first installer
- [Step Types](./step-types.md) - Available action types
- [Checkpointing](./checkpointing.md) - Resume from failures
- [Testing](./testing.md) - Container-based testing
