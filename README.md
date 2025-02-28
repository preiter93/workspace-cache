# workspace-cache

A Rust CLI tool that optimizes dependency caching for Rust workspaces, primarily for Docker builds.

## Installation

```sh
cargo install --path .
```

## Usage

### Commands

#### `workspace-cache deps`

Generates a minimal workspace that contains only dependency information so Cargo builds dependencies but not the application code.

```sh
# Cache dependencies for entire workspace
workspace-cache deps

# Cache dependencies for specific package(s) only
workspace-cache deps -p api
```

This creates a `.workspace-cache/` directory containing:
- `Cargo.toml` - Minimal manifest with all external dependencies
- `Cargo.lock` - Copy of the original lock file (if present)
- `src/main.rs` - Dummy main file

#### `workspace-cache build`

Builds the real workspace using the already cached dependencies.

```sh
# Build entire workspace
workspace-cache build

# Build specific package(s)
workspace-cache build -p api

# Release build
workspace-cache build --release -p api
```

## Microservice Example

Consider a workspace with multiple microservices:

```
my-platform/
├── Cargo.toml
├── Cargo.lock
└── crates/
    ├── api/           # HTTP API service
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── worker/        # Background worker service
    │   ├── Cargo.toml
    │   └── src/main.rs
    └── common/        # Shared library
        ├── Cargo.toml
        └── src/lib.rs
```

Where `api` depends on `axum`, `common` depends on `serde`, and `worker` depends on `tokio`.

### Building only the API service

```sh
# Generate deps for api + common only (excludes worker's dependencies)
workspace-cache deps -p api -p common

# Build the dependency cache
cd .workspace-cache && cargo build --release && cd ..

# Build only the api binary
workspace-cache build --release -p api
```

This ensures you only compile dependencies actually needed by the `api` service.

## Docker Example

### Dockerfile for API service

```dockerfile
FROM rust:1.76 AS builder

WORKDIR /app

# Install workspace-cache
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Copy only manifest files first
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/common/Cargo.toml ./crates/common/Cargo.toml

# Create dummy source files so cargo can parse manifests
RUN mkdir -p crates/api/src crates/common/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "" > crates/common/src/lib.rs

# Generate dependency cache for api + common only
RUN workspace-cache deps -p api -p common

# Build dependencies only (this layer gets cached!)
RUN cd .workspace-cache && cargo build --release

# Remove dummy sources
RUN rm -rf crates/api/src crates/common/src

# Copy real source code
COPY crates/api/src ./crates/api/src
COPY crates/common/src ./crates/common/src

# Build the api service
RUN workspace-cache build --release -p api

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/api /usr/local/bin/api
CMD ["api"]
```

## How It Works

1. **Dependency Phase**: The `deps` command reads `cargo metadata` to extract all external dependencies from the specified workspace members (or all members if none specified). It generates a minimal single-crate project that depends on all these crates, allowing Cargo to build and cache them.

2. **Build Phase**: The `build` command runs `cargo build` on the real workspace (optionally filtered to specific packages). Since dependencies are already compiled and cached, only the actual application code needs to be built.

This two-phase approach allows Docker to cache the dependency layer separately from the source code layer, dramatically speeding up rebuilds when only source files change.

## Testing

```sh
cargo test
```

## Example Workspace

See `example-workspace/` for a working microservice example:

```
example-workspace/
├── Cargo.toml
└── crates/
    ├── api/
    │   ├── Cargo.toml
    │   ├── Dockerfile
    │   └── src/main.rs
    ├── worker/
    │   ├── Cargo.toml
    │   ├── Dockerfile
    │   └── src/main.rs
    └── common/
        ├── Cargo.toml
        └── src/lib.rs
```

Test it:

```sh
# Build workspace-cache
cargo build --release

# Go to example workspace
cd example-workspace

# Generate deps for worker service only
../target/release/workspace-cache deps -p worker -p common

# Build dependency cache
cd .workspace-cache && cargo build --release && cd ..

# Build the worker
../target/release/workspace-cache build --release -p worker

# Run it
./target/release/worker
```
