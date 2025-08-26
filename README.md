# edgectl

**Nimbra Edge CLI** - A command-line tool for managing and monitoring [Nimbra Edge](https://netinsight.net/nimbra-edge/) installations. Built in Rust, it provides a unified interface for controlling various edge resources including inputs, outputs, appliances, nodes, regions, and network tunnels.

## Installation

Download the latest release from [GitHub Releases](https://github.com/netinsight/edge-cli/releases/latest)

Shell completions are automatically included in the Debian package. For other installations, configure completions by adding this to your `~/.bashrc`:

```bash
if command -v edgectl &> /dev/null; then
    source <(edgectl completion bash)
fi
```

## Configuration

`edgectl` requires environment variables for API access:

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
