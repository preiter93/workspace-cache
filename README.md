# workspace-cache

Like [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) but focused on **Rust workspaces with multiple binaries** (microservices). Generates optimized Dockerfiles with proper layer caching.

## Installation

```sh
cargo install --git https://github.com/preiter93/workspace-cache
```

## Quick Start

Generate a Dockerfile for your service:

```sh
workspace-cache dockerfile -p api -o Dockerfile
```

This produces an optimized multi-stage Dockerfile:

```dockerfile
FROM rust:1.87-bookworm AS base
WORKDIR /app
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Prepare minimal workspace
FROM base AS planner
COPY . .
RUN workspace-cache deps -p api

# Build dependencies
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Build the binary
FROM deps AS builder
RUN rm -rf crates/api/src crates/common/src
COPY crates/api crates/api
COPY crates/common crates/common
RUN cargo build --release -p api

# Runtime
FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/api /usr/local/bin/api
ENTRYPOINT ["/usr/local/bin/api"]
```

## Building

```sh
# Build workspace-cache image first
docker build -t workspace-cache .

# Build your service
docker build -f Dockerfile -t api .

# Run it
docker run --rm api
```

## How It Works

1. **planner** - Creates a minimal workspace with stub sources for dependency resolution
2. **deps** - Builds dependencies (cached until Cargo.toml changes)
3. **builder** - Copies real source and builds binary
4. **runtime** - Minimal image with just the binary

When source files change but dependencies don't, Docker skips the `deps` stage entirely.

## Commands

```sh
# Generate Dockerfile
workspace-cache dockerfile -p <package> [-o <output>] [--base-image <image>] [--runtime-image <image>]

# Generate minimal workspace (used internally)
workspace-cache deps -p <package>

# Show resolved workspace dependencies
workspace-cache resolve -p <package>

# Build workspace
workspace-cache build [-p <package>] [--release]
```

## License

MIT
