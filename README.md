# edgectl

**Nimbra Edge CLI** - A command-line tool for managing and monitoring [Nimbra Edge](https://netinsight.net/nimbra-edge/) installations. Built in Rust, it provides a unified interface for controlling various edge resources including inputs, outputs, appliances, nodes, regions, and network tunnels.

## Installation

Download the latest release from [GitHub Releases](https://github.com/netinsight/edgectl/releases/latest)

Shell completions are automatically included in the Debian package. For other installations, generate and install completions:

```bash
if command -v edgectl &> /dev/null; then
    source <(COMPLETE=bash edgectl completion)
fi
```

## Configuration

The easiest way to get started is to use the `login` command:

```bash
edgectl login
```

This will prompt for your Edge URL and credentials, then save them as a context for future use.

The configuration is stored under `$XDG_CONFIG_HOME/edgectl` or
`$HOME/.config/edgectl` on linux, `$HOME/Library/Application Support/edgectl`
on macOS and `{FOLDERID_RoamingAppData}` on windows.

### Multiple Contexts

`edgectl` supports multiple contexts (similar to kubectl) for managing different Edge installations:

```bash
# List contexts
edgectl context list

# Switch between contexts
edgectl context use production
```

### Environment Variables

You can use environment variables to override settings:

```bash
export EDGE_URL="https://your-edge-api-endpoint"
export EDGE_PASSWORD="your-admin-password"
```

## Development

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Make a release
git tag -a v0.0.0-rc0
git push
make release
```
