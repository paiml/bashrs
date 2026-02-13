# Installation

There are several ways to install Rash depending on your platform and preferences.

## Using cargo (Recommended)

The easiest way to install Rash is from crates.io:

```bash
cargo install bashrs
```

This will install both the `bashrs` and `rash` commands (they are aliases).

### Verify Installation

```bash
bashrs --version
```

You should see output like:

```text
bashrs 6.63.0
```

## From Source

If you want the latest development version:

```bash
git clone https://github.com/paiml/bashrs.git
cd bashrs
cargo build --release
sudo cp target/release/bashrs /usr/local/bin/
```

## Platform-Specific Installation

### macOS

Using Homebrew (coming soon):

```bash
# brew install bashrs  # Not yet available
```

For now, use `cargo install bashrs`.

### Linux

#### Debian/Ubuntu (coming soon)

```bash
# wget https://github.com/paiml/bashrs/releases/download/v6.63.0/bashrs_6.61.0_amd64.deb
# sudo dpkg -i bashrs_6.61.0_amd64.deb
```

#### Arch Linux (coming soon)

```bash
# yay -S bashrs
```

For now, use `cargo install bashrs`.

### Windows

Using WSL (Windows Subsystem for Linux):

```bash
# Inside WSL
cargo install bashrs
```

## Requirements

- Rust 1.70+ (if building from source)
- Linux, macOS, or WSL on Windows

## Optional Dependencies

For full functionality, consider installing:

- **shellcheck**: For POSIX compliance verification
  ```bash
  # macOS
  brew install shellcheck

  # Ubuntu/Debian
  sudo apt-get install shellcheck

  # Arch
  sudo pacman -S shellcheck
  ```

## Next Steps

Now that you have Rash installed, let's explore the [Quick Start](./quick-start.md) guide!
