# AGENTS.md

This file provides guidance to AI agents such as Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**edgectl** is a Rust CLI tool for managing [Nimbra Edge](https://netinsight.net/nimbra-edge/) installations. It provides commands for controlling edge resources: inputs, outputs, appliances, nodes, regions, groups, tunnels, and settings.

## Development Commands

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Cross-compile for Linux (x86_64-unknown-linux-musl) using cargo-zigbuild
make target/x86_64-unknown-linux-musl/release/edgectl

# Cross-compile for macOS (aarch64-apple-darwin) using cargo-zigbuild
make target/aarch64-apple-darwin/release/edgectl
```

### Testing

```bash
# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run tests in a specific module
cargo test module_name::
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy
```

**IMPORTANT: Code Quality Standards**

All code must meet these strict requirements:

1. **Zero Warnings Policy** - Treat all compiler warnings as errors. Code must compile with no warnings.

2. **No Dead Code** - Never leave unused structs, functions, methods, or imports. If code is not used, remove it immediately. Do not use `#[allow(dead_code)]` or similar attributes.

3. **Minimal Comments** - Only include comments when absolutely necessary to explain non-obvious logic or business rules. Do not add:
   - Self-explanatory comments (e.g., "// Find the list by name" before `client.find_lists(name)`)
   - Obvious descriptions of what the code does
   - TODO comments without immediate implementation
   - Commented-out code

4. **Clean Build Output** - Every build should be clean with no warnings or errors.

5. **Minimal Output** - Follow the output conventions:
   - Create/Add/Remove operations: Silent on success (no output)
   - Delete operations: Simple text like `Deleted <resource> '<name>'`
   - No emoji or color indicators (`green!("✓")`, `red!("✗")`, etc.)
   - Plain error messages without decorative symbols

### Building Debian Package

```bash
# Build .deb package (automatically installs cargo-zigbuild via Docker)
make deb

# Output: build/edgectl-v<version>.deb
```

### Release Process

```bash
# 1. Create and push a git tag
git tag -a v0.0.0-rc0
git push --tags

# 2. Build and publish release artifacts (requires gh CLI)
make release
```

This builds Linux (x86_64-unknown-linux-musl), macOS (aarch64-apple-darwin) binaries and a Debian package, then creates a GitHub release.

### Configuration for Development

```bash
export EDGE_URL="https://your-edge-api-endpoint"
export EDGE_PASSWORD="your-admin-password"
# EDGE_USER is optional, defaults to "admin"
```

## Architecture

### High-Level Structure

```
src/
├── main.rs           # Entry point, dispatches to subcommands
├── cli.rs            # Clap command tree builder
├── edge.rs           # Core API client, all data models, HTTP logic
└── <resource>.rs     # Per-resource modules (input, output, appliance, etc.)
```

### Key Architectural Patterns

#### 1. Centralized API Client (edge.rs)

All HTTP communication, data models, and serialization logic lives in `edge.rs` (~2,000+ lines). This is the single source of truth for:
- API client (`EdgeClient` struct with blocking reqwest client)
- All data structures (Input, Output, Appliance, Node, Region, etc.)
- Custom serde deserializers for API type conversions (u8 ↔ enums)
- Error types and HTTP status handling

**Do not** create API-related code outside edge.rs. All new resources, fields, or API methods must be added here.

#### 2. Module-per-Resource Pattern

Each resource domain (input, output, appliance, etc.) follows this structure:

```rust
// output_list.rs example
pub fn subcommand() -> Command {
    // Clap command definition with all subcommands
    Command::new("output-list")
        .subcommand(Command::new("list").about("List output lists"))
        .subcommand(Command::new("show").arg(/* ... */))
        // ...
}

pub fn run(args: &ArgMatches) {
    // Dispatch based on subcommand name
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("show", sub_args)) => show(sub_args),
        Some(("create", sub_args)) => create(sub_args),
        Some(("delete", sub_args)) => delete(sub_args),
        // ...
    }
}

fn list() {
    let client = edge::new_client();  // Creates authenticated client
    let items = client.list_output_recipient_lists();
    // Format and print output
}
```

**Key points:**
- Each module exports `subcommand()` (builds Clap command tree) and `run()` (dispatcher)
- Handler functions use simple names: `list()`, `show()`, `create()`, `delete()`, `add_*()`, `remove_*()`
- Handlers call `edge::new_client()` to get an authenticated EdgeClient
- No async - uses `reqwest::blocking::Client`
- Handlers exit process on error (via expect/unwrap or explicit process::exit)

#### 3. Error Handling Strategy

The codebase uses **fast-fail** error handling:
- Configuration errors (missing env vars) → immediate exit with message
- Authentication failures → immediate exit with message
- API errors → unwrap/expect causes panic and exit
- No graceful error recovery or retries

When adding API methods to `edge.rs`:
- Return `Result<T, reqwest::Error>` for HTTP operations
- Let callers handle errors (usually via unwrap)
- API errors propagate as HTTP response body text

#### 4. CLI Construction (cli.rs)

The command tree is built in `cli.rs` by calling each module's `subcommand()` function:

```rust
pub(crate) fn build() -> Command {
    Command::new("edgectl")
        .about("Nimbra Edge CLI")
        .subcommand_required(true)
        .subcommand(input::subcommand())
        .subcommand(output::subcommand())
        // ...
}
```

Dispatching happens in `main.rs` using pattern matching on subcommand names.

#### 5. Output Formatting

**Table Output:** Uses the `tabled` crate with empty styling and title case headers:

```rust
use tabled::{builder::Builder, settings::Style};

let mut builder = Builder::default();
builder.push_record(["ID", "Name", "Description"]);  // Title case headers

for item in items {
    builder.push_record([&item.id, &item.name, &item.description]);
}

let mut table = builder.build();
table.with(Style::empty());
println!("{}", table);
```

**Output Conventions:**
- **Silent on success**: Create, add, and remove operations produce no output on success
- **Simple messages**: Delete operations output plain text: `Deleted <resource> '<name>'`
- **No emoji or color**: Avoid using `green!("✓")`, `red!("✗")` or similar status indicators
- **Error messages**: Plain text without emoji, e.g., `eprintln!("Resource '{}' not found", name);`

#### 6. Authentication & Session Management

Authentication is eager and cookie-based:
1. `edge::new_client()` creates client with cookie jar
2. Immediately calls `client.login(username, password)`
3. API returns session cookie, stored in cookie jar
4. All subsequent requests include cookie automatically
5. No explicit session refresh - relies on cookie validity

## Module Responsibilities

- **edge.rs**: API client, all data models, HTTP/JSON handling, custom serializers
- **main.rs**: Entry point, top-level command dispatching to modules
- **cli.rs**: Clap command tree construction (delegates to module subcommands)
- **input.rs, output.rs, output_list.rs, group_list.rs, etc.**: Resource-specific command handlers
- **colors.rs**: Terminal color macros (exists for legacy code, avoid using in new code)
- **completions.rs**: Shell completion generation (bash, zsh)
- **health.rs**: Health check commands
- **buildinfo.rs**: Display build metadata
- **kubernetes.rs**: Kubernetes-related utilities (if applicable)
- **settings.rs**: Settings management commands
- **tunnels.rs**: Network tunnel management

## Adding a New Command

To add a new command to an existing resource (e.g., `edgectl output-list add-output`):

1. **Add API method to `edge.rs`**:
   ```rust
   impl EdgeClient {
       pub fn add_output_to_list(
           &self,
           list_id: &str,
           list_name: &str,
           output_ids: Vec<String>,
       ) -> Result<(), EdgeError> {
           let url = format!("{}/api/outputRecipientList/{}", self.url, list_id);
           // ... implementation
       }
   }
   ```

2. **Add subcommand in resource module** (e.g., `output_list.rs`):
   ```rust
   pub fn subcommand() -> Command {
       Command::new("output-list")
           // ... existing subcommands
           .subcommand(
               Command::new("add-output")
                   .about("Add an output to an output list")
                   .arg(Arg::new("list").required(true).help("The name of the output list"))
                   .arg(Arg::new("output").required(true).num_args(1..).help("The name of the outputs to add"))
           )
   }
   ```

3. **Add handler in dispatcher**:
   ```rust
   pub fn run(args: &ArgMatches) {
       match args.subcommand() {
           // ... existing matches
           Some(("add-output", sub_args)) => add_output(sub_args),
           // ...
       }
   }

   fn add_output(args: &ArgMatches) {
       let list_name = args.get_one::<String>("list").unwrap();
       let output_names = args.get_many::<String>("output").unwrap();
       let client = new_client();

       let lists = client.find_output_recipient_lists(list_name)
           .expect("Failed to find output list");
       if lists.is_empty() {
           eprintln!("Output list '{}' not found", list_name);
           std::process::exit(1);
       }

       let output_ids = get_output_ids_by_names(&client, output_names);
       client.add_output_to_list(&lists[0].id, &lists[0].name, output_ids)
           .expect("Failed to add output to list");
       // Silent on success - no output
   }
   ```

## Adding a New Resource Module

To add a completely new resource (e.g., `stream`):

1. **Create `src/stream.rs`** following the module-per-resource pattern
2. **Add data structures to `edge.rs`** (Stream, ListStreamsResponse, etc.)
3. **Add API methods to `EdgeClient` in `edge.rs`**
4. **Add `mod stream;` to `main.rs`**
5. **Register in `cli.rs`**: `.subcommand(stream::subcommand())`
6. **Add dispatcher case in `main.rs`**: `Some(("stream", subcmd)) => stream::run(subcmd),`

## Build System Notes

### Cross-Compilation

The project uses `cargo-zigbuild` via Docker for cross-platform builds:
- Runs in container: `ghcr.io/rust-cross/cargo-zigbuild:0.19.7`
- Targets: `x86_64-unknown-linux-musl` (Linux), `aarch64-apple-darwin` (macOS)
- OpenSSL is vendored (via `features = ["vendored"]`) to enable static linking

### Debian Package Structure

Generated .deb includes:
- Binary: `/usr/bin/edgectl`
- Bash completions: `/usr/share/bash-completion/completions/edgectl`
- Zsh completions: `/usr/local/share/zsh/site-functions/edgectl`

Completions are generated by running `edgectl completion <shell>`.

## Important Conventions

### Data Serialization

API responses often use numeric enums. Convert them using custom Deserialize implementations in edge.rs:

```rust
#[derive(Debug)]
pub enum InputType {
    Srt,
    RtmpPush,
    // ...
}

impl<'de> Deserialize<'de> for InputType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Srt),
            1 => Ok(Self::RtmpPush),
            _ => Err(D::Error::unknown_variant(&value.to_string(), &["0", "1"])),
        }
    }
}
```

For serialization back to API, implement `Serialize` trait similarly.

### Environment Variables

- `EDGE_URL`: Base API URL (required)
- `EDGE_PASSWORD`: Admin password (required)
- `EDGE_USER`: Username (optional, defaults to "admin")

### Table Output

Always use empty styling with title case headers:

```rust
use tabled::{builder::Builder, settings::Style};

let mut builder = Builder::default();
builder.push_record(["ID", "Name", "Description"]);  // Title case

for item in items {
    builder.push_record([&item.id, &item.name, &item.description]);
}

let mut table = builder.build();
table.with(Style::empty());
println!("{}", table);
```

### Output Style

**Do not use emoji or color indicators in output:**
- ❌ `println!("{} Created successfully", green!("✓"));`
- ✅ Silent on success (no output)
- ❌ `eprintln!("{} Resource not found", red!("✗"));`
- ✅ `eprintln!("Resource '{}' not found", name);`

**Output conventions:**
- Create/Add/Remove operations: Silent on success
- Delete operations: Plain text like `Deleted <resource> '<name>'`
- List operations: Table output only
- Show operations: Plain text fields, then table if applicable

## CI/CD

GitHub Actions workflow (`.github/workflows/rust.yml`):
- Runs on every push and PR
- Executes `cargo build` and `cargo test`
- Uses Ubuntu runner
