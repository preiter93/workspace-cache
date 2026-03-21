# workspace-cache

Like [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) but focused on **Rust workspaces with multiple binaries** (microservices). Generates optimized Dockerfiles with proper layer caching.

## Installation

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
FROM rust:latest AS base
WORKDIR /app
RUN cargo install --git https://github.com/preiter93/workspace-cache workspace-cache

# Prepare minimal workspace
FROM base AS planner
COPY . .
RUN workspace-cache deps --bin api

# Build dependencies
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Build the binary
FROM deps AS builder
RUN rm -rf crates/api/src crates/common/src
COPY crates/api crates/api
COPY crates/common crates/common
RUN cargo build --release --bin api

# Runtime
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
2. **deps** - Builds dependencies (cached until any dependency changes)
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
- `--no-deps` - Skip fetching dependencies for faster generation

Examples:
```sh
# Generate release Dockerfile (default)
workspace-cache dockerfile --bin api -o Dockerfile

# Generate debug Dockerfile
workspace-cache dockerfile --bin api --profile debug -o Dockerfile.debug

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
- `--no-deps` - Skip fetching dependencies from crates.io

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

## No Deps Mode

Use `--no-deps` to skip fetching dependencies from crates.io for faster builds (~10-15s faster).
Note: This leads to less optimized caching since any change to a dependency will invalidate the cache.

```sh
workspace-cache deps --bin api --no-deps
workspace-cache dockerfile --bin api --no-deps -o Dockerfile
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