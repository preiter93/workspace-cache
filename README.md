# workspace-cache

Like [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) but focused on **Rust workspaces with multiple binaries** (microservices). Generates optimized Dockerfiles with proper layer caching.

## Installation

```sh
cargo install workspace-cache
```

Or install from git for the latest development version:

```sh
cargo install --git https://github.com/preiter93/workspace-cache workspace-cache
```

## Quick Start

Generate a Dockerfile for your service:

```sh
workspace-cache dockerfile --bin api -o Dockerfile
```

This produces an optimized multi-stage Dockerfile:

```dockerfile
# Stage 1: Install workspace-cache tool
FROM rust:1.94-bookworm AS base
WORKDIR /app
RUN cargo install workspace-cache@0.1.0

# Stage 2: Generate minimal workspace with stub sources
FROM base AS planner
COPY . .
RUN workspace-cache deps --bin api

# Stage 3: Build dependencies only (cached until Cargo.toml/Cargo.lock change)
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Stage 4: Build the actual binary with real source code
FROM deps AS builder
RUN rm -rf crates/api/src crates/common/src
COPY crates/api crates/api
COPY crates/common crates/common
RUN cargo clean --release -p api -p common
RUN cargo build --release --bin api

# Stage 5: Minimal runtime image
FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/api /usr/local/bin/api
ENTRYPOINT ["/usr/local/bin/api"]
```

## Build & Run

```sh
docker build -f Dockerfile -t api .
docker run --rm api
```

## How It Works

1. **planner** - Creates a minimal workspace with stub sources for dependency resolution
2. **deps** - Builds dependencies (cached until Cargo.toml/Cargo.lock changes)
3. **builder** - Copies real source and builds binary
4. **runtime** - Minimal image with just the binary

When source files change but dependencies don't, Docker skips the `deps` stage entirely.

## Commands

### Generate Dockerfile

```sh
workspace-cache dockerfile --bin <binary> [OPTIONS]
```

Options:
- `--bin <binary>` - Binary to build (required)
- `--profile <profile>` - Build profile: `release` or `debug` (default: `release`)
- `-o, --output <path>` - Output path (default: stdout)
- `--base-image <image>` - Base image for build stages (default: `rust:1.94-bookworm`)
- `--runtime-image <image>` - Runtime image (default: `debian:bookworm-slim`)
- `--tool-version <version>` - Version of workspace-cache to install (default: current version)
- `--from-git` - Install workspace-cache from git instead of crates.io
- `--fast` - Fast mode: skip dependency resolution for faster generation

Examples:
```sh
# Generate release Dockerfile (default)
workspace-cache dockerfile --bin api -o Dockerfile

# Generate debug Dockerfile
workspace-cache dockerfile --bin api --profile debug -o Dockerfile.debug

# Use a specific version
workspace-cache dockerfile --bin api --tool-version 0.1.0 -o Dockerfile

# Install from git (latest dev version)
workspace-cache dockerfile --bin api --from-git -o Dockerfile

# Custom base image
workspace-cache dockerfile --bin api --base-image rust:1.80-alpine -o Dockerfile
```

### Generate Minimal Workspace

```sh
workspace-cache deps [OPTIONS]
```

Options:
- `--bin <binary>` - Only include dependencies for specific binary/binaries
- `-o, --output <dir>` - Output directory (default: `.workspace-cache`)
- `--fast` - Fast mode: skip dependency resolution

Examples:
```sh
# Generate for all workspace binaries
workspace-cache deps

# Generate for a specific binary
workspace-cache deps --bin api

# Generate for multiple binaries
workspace-cache deps --bin api --bin worker

# Custom output directory
workspace-cache deps --bin api -o my-cache
```

### Show Resolved Dependencies

```sh
workspace-cache resolve --bin <binary>
```

Shows which workspace members a binary depends on.

### Build Workspace

```sh
workspace-cache build [OPTIONS]
```

Options:
- `--bin <binary>` - Only build specific binary/binaries
- `--release` - Build in release mode

Examples:
```sh
# Build all binaries
workspace-cache build

# Build specific binary in release mode
workspace-cache build --bin api --release
```

## Fast Mode

Use `--fast` to skip dependency resolution for faster builds (~10-15s faster).
Note: This leads to less optimized caching since any change to a dependency will invalidate the cache.

```sh
workspace-cache deps --bin api --fast
workspace-cache dockerfile --bin api --fast -o Dockerfile
```

## Testing

Run unit tests:

```sh
cargo test
```

Test locally in your workspace:

```sh
# Generate minimal workspace for a binary
workspace-cache deps --bin api

# Build dependencies
cd .workspace-cache
cargo build --release

# Copy real sources and build (deps are cached)
rm -rf crates/api/src crates/common/src
cp -r ../crates/api/src crates/api/src
cp -r ../crates/common/src crates/common/src
cargo build --release --bin api
```

Note: This mirrors how the generated Dockerfile works. The key is building
the final binary from within `.workspace-cache/` after copying real sources.

## License

MIT