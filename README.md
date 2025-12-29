# cargo-jam

[![CI](https://github.com/abutlabs/cargo-jam/actions/workflows/ci.yml/badge.svg)](https://github.com/abutlabs/cargo-jam/actions/workflows/ci.yml)

cargo, make me a JAM service

A Rust cargo subcommand for generating and building JAM (Join-Accumulate Machine) services for Polkadot. Follows the [cargo-generate](https://github.com/cargo-generate/cargo-generate) project architecture while providing JAM-specific tooling.

## Installation

### Install From crates.io

```bash
cargo install cargo-jam
```

### Install From source

```bash
git clone https://github.com/abutlabs/cargo-jam
cd cargo-jam
cargo install --path .
```

To reinstall after making changes:

```bash
cargo install --path . --force
```

## Prerequisites

Before using cargo-jam, you need:

1. **Rust toolchain** with nightly support
2. **jam-pvm-build** - PVM bytecode compiler

```bash
cargo install jam-pvm-build
```

## Quick Start

```bash
# 1. Install the JAM toolchain (downloads polkajam binaries)
cargo jam setup

# 2. Create a new JAM service
cargo jam new my-service

# 3. Build the service
cd my-service
cargo jam build

# 4. Start local testnet
cargo jam up

# 5. Deploy to testnet (in another terminal)
cargo jam deploy my-service.jam

# 6. Stop testnet when done
cargo jam down
```

## Commands

### `cargo jam setup`

Downloads and installs the JAM/polkajam toolchain from [polkajam-releases](https://github.com/paritytech/polkajam-releases).

```bash
# Install latest nightly
cargo jam setup

# List available versions
cargo jam setup --list

# Show installed toolchain info
cargo jam setup --info

# Install specific version
cargo jam setup --version nightly-2025-12-29

# Force reinstall
cargo jam setup --force

# Update to latest
cargo jam setup --update
```

**Installed binaries** (in `~/.cargo-jam/toolchain/polkajam-nightly/`):
- `polkajam` - JAM node
- `jamt` - JAM CLI tool for deployment
- `polkajam-testnet` - Local testnet runner
- `polkajam-repl` - Interactive REPL
- `corevm-builder` - CoreVM builder
- And more...

### `cargo jam new`

Creates a new JAM service project from a template.

```bash
# Interactive mode
cargo jam new my-service

# Skip prompts, use defaults
cargo jam new my-service --defaults

# Use custom git template
cargo jam new my-service --git https://github.com/user/template

# Specify template values
cargo jam new my-service -d author="Your Name" -d license=MIT
```

**Options:**
- `-t, --template <name>` - Template name (default: basic-service)
- `--git <url>` - Git repository URL for custom template
- `--branch <branch>` - Git branch (requires --git)
- `--path <path>` - Subdirectory in git repo (requires --git)
- `-o, --output <dir>` - Output directory
- `--defaults` - Skip prompts, use defaults
- `-d, --define <key=value>` - Set template variable
- `--no-git` - Don't initialize git repository

### `cargo jam build`

Builds a JAM service into a `.jam` blob using `jam-pvm-build`.

```bash
# Build in release mode (default)
cargo jam build

# Build specific project
cargo jam build --path /path/to/service

# Custom output path
cargo jam build --output my-service.jam

# Verbose output
cargo jam build --verbose
```

**Options:**
- `-p, --path <dir>` - Project path (default: current directory)
- `--release` - Build in release mode (default: true)
- `-o, --output <path>` - Output path for .jam blob
- `-v, --verbose` - Verbose output

### `cargo jam up`

Starts the local JAM testnet.

```bash
# Start in background (default)
cargo jam up

# Start in foreground (see logs)
cargo jam up --foreground

# Custom RPC endpoint
cargo jam up --rpc ws://localhost:9944
```

**Options:**
- `--foreground` - Run in foreground (see logs, Ctrl+C to stop)
- `--rpc <url>` - RPC endpoint (default: ws://localhost:19800)
- `-v, --verbose` - Verbose output

### `cargo jam down`

Stops the local JAM testnet.

```bash
# Stop the testnet gracefully
cargo jam down

# Force kill
cargo jam down --force
```

**Options:**
- `--force` - Force kill with SIGKILL instead of SIGTERM
- `-v, --verbose` - Verbose output

### `cargo jam deploy`

Deploys a JAM service to the network.

```bash
# Deploy a service
cargo jam deploy my-service.jam

# Deploy with initial balance
cargo jam deploy my-service.jam --amount 1000

# Deploy with memo
cargo jam deploy my-service.jam --memo "my memo data"

# Deploy to custom RPC endpoint
cargo jam deploy my-service.jam --rpc ws://localhost:9944

# Register service with a name
cargo jam deploy my-service.jam --register my_service
```

**Options:**
- `--amount <value>` - Initial balance for the service (default: 0)
- `--memo <data>` - Memo data to include
- `-G, --min-item-gas <value>` - Minimum gas per work item (default: 1000000)
- `-g, --min-memo-gas <value>` - Minimum gas for memo (default: 1000000)
- `-r, --register <name>` - Register service with a name
- `--rpc <url>` - RPC endpoint (default: ws://localhost:19800)
- `-v, --verbose` - Verbose output

### `cargo jam monitor`

Monitor the testnet with an interactive TUI (jamtop).

```bash
# Start the monitor
cargo jam monitor

# Monitor with custom RPC endpoint
cargo jam monitor --rpc ws://localhost:9944
```

**Options:**
- `--rpc <url>` - RPC endpoint (default: ws://localhost:19800)
- `-v, --verbose` - Verbose output

### `cargo jam test`

Run comprehensive end-to-end tests that verify the entire workflow.

```bash
# Run full test suite
cargo jam test

# Run tests with verbose output
cargo jam test --verbose

# Keep testnet running after tests (for debugging)
cargo jam test --keep-running

# Skip testnet startup (use already running testnet)
cargo jam test --skip-testnet

# Use custom test directory
cargo jam test --dir /tmp/my-test
```

**Options:**
- `--keep-running` - Keep testnet running after tests complete
- `--skip-testnet` - Skip testnet startup (assume already running)
- `--dir <path>` - Test directory (default: temp directory)
- `-v, --verbose` - Verbose output with command details

**Tests performed:**
1. Create new JAM service (`cargo jam new`)
2. Build JAM service to `.jam` blob (`cargo jam build`)
3. Start local testnet (`cargo jam up`)
4. Deploy service to testnet (`cargo jam deploy`)
5. Stop testnet (`cargo jam down`)

## Local Development

### Running from source

```bash
# Run commands directly
cargo run -- setup --info
cargo run -- new test-service --defaults
cargo run -- build

# Or build and install locally
cargo build && cargo install --path . --force
```

### Development workflow

```bash
# Make changes, rebuild, and test
cargo build && cargo install --path . --force && cargo jam setup --info
```

### Uninstall

```bash
cargo uninstall cargo-jam
rm -rf ~/.cargo-jam  # Remove toolchain and config
```

## Testing with Local Testnet

### Full end-to-end test

```bash
# Terminal 1: Start the testnet
cargo jam up --foreground

# Terminal 2: Create and deploy a service
cargo jam new test-service --defaults
cd test-service
cargo jam build
cargo jam deploy test-service.jam

# Stop the testnet when done
cargo jam down
```

### Background mode workflow

```bash
# Start testnet in background
cargo jam up

# Create, build, and deploy
cargo jam new test-service --defaults
cd test-service
cargo jam build
cargo jam deploy test-service.jam

# Stop testnet
cargo jam down
```

### Additional testnet tools

```bash
# Monitor with jamtop
cargo jam monitor

# Interactive REPL
~/.cargo-jam/toolchain/polkajam-nightly/polkajam-repl
```

## Project Structure

Generated JAM service structure:

```
my-service/
├── Cargo.toml          # Dependencies: jam-pvm-common, polkavm-derive
├── src/
│   └── lib.rs          # Service implementation (refine, accumulate)
└── .gitignore
```

### Service Implementation

```rust
#![no_std]
#![no_main]

extern crate alloc;

use jam_pvm_common::{declare_service, Service, accumulate::*, jam_types::*};

declare_service!(MyService);

struct MyService;

impl Service for MyService {
    fn refine(
        _core_index: CoreIndex,
        _item_index: usize,
        _service_id: ServiceId,
        payload: WorkPayload,
        _package_hash: WorkPackageHash,
    ) -> WorkOutput {
        // Stateless computation (up to 6 seconds)
        payload.take().into()
    }

    fn accumulate(
        _slot: Slot,
        _service_id: ServiceId,
        item_count: usize,
    ) -> Option<Hash> {
        // Stateful integration (~10ms)
        None
    }
}
```

## Configuration

Configuration is stored in `~/.cargo-jam/`:

```
~/.cargo-jam/
├── config.toml              # Toolchain configuration
└── toolchain/
    └── polkajam-nightly/    # Installed binaries
```

**config.toml:**
```toml
installed_version = "nightly-2025-12-29"
toolchain_path = "/Users/you/.cargo-jam/toolchain"
installed_at = "1767015039"
```

## Running Tests

### End-to-end tests (recommended)

The easiest way to test the entire cargo-jam workflow:

```bash
# Run full end-to-end test suite
cargo jam test

# Run with verbose output
cargo jam test --verbose
```

This automatically tests: new → build → up → deploy → down

### Unit and integration tests

```bash
# Run all Rust tests (excluding ignored tests)
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### Full integration tests (require jam-pvm-build)

```bash
# Run ignored tests that require toolchain
cargo test --test integration_tests -- --ignored
```

### Manual testnet deployment tests

These tests require a running local testnet:

```bash
# Terminal 1: Start the testnet
cargo jam up --foreground

# Terminal 2: Run testnet tests
cargo test --test testnet_tests -- --ignored --nocapture
```

The testnet tests will:
1. Create a new JAM service
2. Build it to a `.jam` blob
3. Deploy it to the running testnet using `cargo jam deploy`

## Publishing to crates.io

```bash
# Login to crates.io
cargo login <your-token>

# Dry run to verify
cargo publish --dry-run

# Publish
cargo publish
```

## Reference JAM Service implementations

- [zk-jam-service](https://github.com/abutlabs/zk-jam-service) - Zero-knowledge proof verification service

## Resources

- [JAM Gray Paper](https://graypaper.com/) - Official JAM specification
- [PolkaVM](https://github.com/paritytech/polkavm) - The virtual machine powering JAM
- [polkajam-releases](https://github.com/paritytech/polkajam-releases) - JAM toolchain releases
- [Building JAM Services in Rust](https://forum.polkadot.network/t/building-jam-services-in-rust/10161) - Forum discussion

## License

MIT OR Apache-2.0
