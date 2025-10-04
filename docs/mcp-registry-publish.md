# MCP Registry Publishing Guide

This document describes the automated publishing process for the Rash MCP server to the Model Context Protocol (MCP) registry.

## Overview

The Rash MCP server is published to two locations:
1. **crates.io** - Rust package registry
2. **MCP Registry** - Official Model Context Protocol server registry

Publishing is fully automated via GitHub Actions when you push a tag with the `mcp-v*` pattern.

## Prerequisites

### Required Secrets

Configure these in your GitHub repository settings (Settings → Secrets → Actions):

1. **CARGO_TOKEN** (Required for crates.io publishing)
   - Create at: https://crates.io/settings/tokens
   - Add to: https://github.com/paiml/bashrs/settings/secrets/actions
   - Scope: `publish-update` permission

### Repository Permissions

The GitHub Actions workflow requires:
- `contents: read` - Read repository contents
- `id-token: write` - Generate OIDC tokens for MCP registry authentication

These are already configured in `.github/workflows/publish-mcp.yml`.

## Publishing Process

### 1. Prepare for Release

Ensure all changes are committed and pushed to `main`:

```bash
# Make your changes
git add .
git commit -m "feat: Add new MCP functionality"
git push origin main
```

### 2. Create and Push Version Tag

Tag the release with the `mcp-v*` pattern:

```bash
# Create tag (replace with your version)
git tag mcp-v0.2.0

# Push tag to trigger workflow
git push origin mcp-v0.2.0
```

### 3. Monitor Workflow

The GitHub Actions workflow will automatically:

1. ✅ Run tests on `rash-mcp`
2. ✅ Build release binary
3. ✅ Publish to crates.io (if CARGO_TOKEN is configured)
4. ✅ Build MCP publisher CLI from source
5. ✅ Authenticate via GitHub OIDC
6. ✅ Publish to MCP registry

Monitor progress at: https://github.com/paiml/bashrs/actions/workflows/publish-mcp.yml

### 4. Verify Publication

Check that the server appears in the registry:

```bash
curl -s "https://registry.modelcontextprotocol.io/v0/servers?search=rash" | jq
```

Or visit: https://registry.modelcontextprotocol.io/v0/servers?search=rash

## Server Metadata

Server metadata is defined in `rash-mcp/server.json`:

```json
{
  "$schema": "https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json",
  "name": "io.github.paiml/rash",
  "displayName": "Rash MCP Server",
  "description": "Transpile Rust code to POSIX-compliant shell scripts...",
  "version": "0.1.0",
  "deployment": {
    "type": "package",
    "package": {
      "type": "cargo",
      "name": "rash-mcp",
      "binaryName": "rash-mcp"
    }
  }
}
```

### Key Fields

- **name**: `io.github.paiml/rash` - Namespace format must match GitHub org
- **description**: Max 100 characters
- **version**: Must match `Cargo.toml` version
- **deployment.package.type**: `cargo` for Rust crates

## Troubleshooting

### Build Failures

**Problem**: Tests fail during workflow

**Solution**: Run tests locally first:
```bash
cargo test -p rash-mcp
```

### Authentication Errors

**Problem**: `403 Forbidden` when publishing to MCP registry

**Solution**: Verify namespace format in `server.json`:
- ✅ Correct: `io.github.paiml/rash`
- ❌ Wrong: `io.github.paiml.rash`

### Crates.io Publishing Fails

**Problem**: `error: crate rash-mcp@X.Y.Z already exists`

**Solution**: This is normal if the version was previously published. The workflow uses `continue-on-error: true` for this step.

### Description Too Long

**Problem**: `validation failed: expected length <= 100`

**Solution**: Edit `server.json` to shorten the description to 100 characters or less.

## Workflow Configuration

The publishing workflow is defined in `.github/workflows/publish-mcp.yml`:

```yaml
name: Publish MCP Server

on:
  push:
    tags:
      - 'mcp-v*.*.*'
  workflow_dispatch:  # Manual trigger option

permissions:
  contents: read
  id-token: write  # Required for GitHub OIDC
```

### Manual Triggering

You can also manually trigger the workflow:

1. Go to: https://github.com/paiml/bashrs/actions/workflows/publish-mcp.yml
2. Click "Run workflow"
3. Select branch and run

## Version Management

### Semantic Versioning

Follow semantic versioning for MCP server releases:
- **Major** (1.0.0): Breaking changes to API
- **Minor** (0.2.0): New features, backward compatible
- **Patch** (0.1.1): Bug fixes, backward compatible

### Updating Version

Update version in two places:

1. **rash-mcp/Cargo.toml**:
```toml
[package]
version = "0.2.0"
```

2. **rash-mcp/server.json**:
```json
{
  "version": "0.2.0"
}
```

## Dependencies

### Cargo Dependencies

The MCP server depends on:
- `bashrs` - Main transpiler library
- `pforge-runtime` - MCP framework runtime (published to crates.io)

Ensure `pforge-runtime` is published before publishing `rash-mcp`.

### Build Dependencies

The workflow requires:
- **Rust**: Latest stable (via `dtolnay/rust-toolchain`)
- **Go 1.21+**: To build MCP publisher CLI
- **Git**: To clone MCP registry repository

## Registry Schema

The MCP registry uses JSON Schema for validation. Current schema version:
- **URL**: https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json
- **Format**: camelCase field names
- **Validation**: Enforced by registry API

### Schema Updates

If the registry schema changes:
1. Update `$schema` URL in `server.json`
2. Follow migration guide: https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/CHANGELOG.md

## Resources

- **MCP Registry**: https://github.com/modelcontextprotocol/registry
- **Publishing Guide**: https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/
- **Schema Reference**: https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/
- **Workflow Runs**: https://github.com/paiml/bashrs/actions/workflows/publish-mcp.yml

## Support

For issues with:
- **MCP Registry**: https://github.com/modelcontextprotocol/registry/issues
- **Rash Publishing**: https://github.com/paiml/bashrs/issues
- **GitHub Actions**: Check workflow logs at https://github.com/paiml/bashrs/actions
