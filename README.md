# workspace-cache

Like [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) but focused on **Rust workspaces with multiple binaries** (microservices). Generates optimized Dockerfiles with proper layer caching.

## Installation

```sh
cargo install --git https://github.com/preiter93/workspace-cache workspace-cache
```

## Quick Start

Generate a Dockerfile for your service:

```sh
workspace-cache dockerfile -p api -o Dockerfile
```

This produces an optimized multi-stage Dockerfile:

```dockerfile
FROM rust:latest AS base
WORKDIR /app
RUN cargo install --git https://github.com/preiter93/workspace-cache workspace-cache

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

```sh
# Generate Dockerfile
workspace-cache dockerfile -p <package> [-o <output>] [--base-image <image>] [--runtime-image <image>]

# Generate minimal workspace
workspace-cache deps -p <package>

# Show resolved workspace dependencies
workspace-cache resolve -p <package>

# Build workspace
workspace-cache build [-p <package>] [--release]
```

## Testing

Run unit tests:

```sh
cargo test
```

Test locally in your workspace:

```sh
# Generate minimal workspace for a package
workspace-cache deps -p api

# Build dependencies
cd .workspace-cache
cargo build --release

# Copy real sources and build (deps are cached)
rm -rf crates/api/src crates/common/src
cp -r ../crates/api/src crates/api/src
cp -r ../crates/common/src crates/common/src
cargo build --release -p api
```

Note: This mirrors how the generated Dockerfile works. The key is building
the final binary from within `.workspace-cache/` after copying real sources.

## License

MIT
