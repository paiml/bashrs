# bashrs — VS Code Extension

Real-time shell script linting, auto-fix, and hover documentation powered by the
[bashrs](https://github.com/paiml/bashrs) language server.

## Features

- **Diagnostics**: Real-time lint warnings and errors on save/change (487+ rules)
- **Quick Fix**: One-click auto-fix for safe corrections
- **Hover**: Rule documentation, severity, and fix suggestions on hover
- **Multi-language**: Bash/Shell, Makefile, and Dockerfile support
- **Zero config**: Works out of the box once `bashrs` is installed

## Requirements

Install the `bashrs` CLI:

```bash
cargo install bashrs
```

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `bashrs.serverPath` | `"bashrs"` | Path to the bashrs binary |
| `bashrs.lintOnSave` | `true` | Lint on file save |
| `bashrs.lintOnChange` | `true` | Real-time diagnostics on change |
| `bashrs.trace.server` | `"off"` | LSP trace level (`off`, `messages`, `verbose`) |

## Building from Source

```bash
cd editors/vscode
npm install
npm run compile
npm run package   # produces bashrs-0.1.0.vsix
```

Install the `.vsix`:

```bash
code --install-extension bashrs-0.1.0.vsix
```

## License

MIT
